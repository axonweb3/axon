use log::{debug, warn};
use prost::{Message, Oneof};
use std::{
    collections::{HashMap, HashSet},
    str,
    sync::Arc,
    time::{Duration, Instant},
};
use tentacle::{
    async_trait,
    bytes::{Bytes, BytesMut},
    context::{ProtocolContext, ProtocolContextMutRef},
    service::TargetSession,
    traits::ServiceProtocol,
    utils::extract_peer_id,
    SessionId,
};

use crate::peer_manager::PeerManager;

const SEND_PING_TOKEN: u64 = 0;
const CHECK_TIMEOUT_TOKEN: u64 = 1;

/// Ping protocol handler.
///
/// The interval means that we send ping to peers.
/// The timeout means that consider peer is timeout if during a timeout we still
/// have not received pong from a peer
pub struct PingHandler {
    interval:              Duration,
    timeout:               Duration,
    connected_session_ids: HashMap<SessionId, PingStatus>,
    start_time:            Instant,
    peer_manager:          Arc<PeerManager>,
}

impl PingHandler {
    pub fn new(
        interval: Duration,
        timeout: Duration,
        peer_manager: Arc<PeerManager>,
    ) -> PingHandler {
        let now = Instant::now();
        PingHandler {
            interval,
            timeout,
            connected_session_ids: Default::default(),
            start_time: now,
            peer_manager,
        }
    }

    fn ping_received(&mut self, _id: SessionId) {}

    fn pong_received(&mut self, _id: SessionId, _last_ping: Instant) {}

    async fn ping_peers(&mut self, context: &ProtocolContext) {
        let now = Instant::now();
        let send_nonce = nonce(&now, self.start_time);
        let peers: HashSet<SessionId> = self
            .connected_session_ids
            .iter_mut()
            .filter_map(|(session_id, ps)| {
                if ps.processing {
                    None
                } else {
                    ps.processing = true;
                    ps.last_ping_sent_at = now;
                    ps.nonce = send_nonce;
                    Some(*session_id)
                }
            })
            .collect();
        if !peers.is_empty() {
            debug!("start ping peers: {:?}", peers);
            let ping_msg = PingMessage::new_ping(send_nonce).into_bytes();
            let proto_id = context.proto_id;
            if context
                .filter_broadcast(
                    TargetSession::Filter(Box::new(move |id| peers.contains(id))),
                    proto_id,
                    ping_msg,
                )
                .await
                .is_err()
            {
                debug!("send message fail");
            }
        }
    }
}

fn nonce(t: &Instant, start_time: Instant) -> u32 {
    t.duration_since(start_time).as_secs() as u32
}

/// PingStatus of a peer
#[derive(Clone, Debug)]
struct PingStatus {
    /// Are we currently pinging this peer?
    processing:        bool,
    /// The time we last send ping to this peer.
    last_ping_sent_at: Instant,
    nonce:             u32,
}

impl PingStatus {
    /// A meaningless value, peer must send a pong has same nonce to respond a
    /// ping.
    fn nonce(&self) -> u32 {
        self.nonce
    }

    /// Time duration since we last send ping.
    fn elapsed(&self) -> Duration {
        (self.last_ping_sent_at).saturating_duration_since(Instant::now())
    }
}

#[async_trait]
impl ServiceProtocol for PingHandler {
    async fn init(&mut self, context: &mut ProtocolContext) {
        // periodicly send ping to peers
        let proto_id = context.proto_id;
        if context
            .set_service_notify(proto_id, self.interval, SEND_PING_TOKEN)
            .await
            .is_err()
        {
            warn!("start ping fail");
        }
        if context
            .set_service_notify(proto_id, self.timeout, CHECK_TIMEOUT_TOKEN)
            .await
            .is_err()
        {
            warn!("start ping fail");
        }
    }

    async fn connected(&mut self, context: ProtocolContextMutRef<'_>, version: &str) {
        let session = context.session;
        self.connected_session_ids
            .entry(session.id)
            .or_insert_with(|| PingStatus {
                last_ping_sent_at: Instant::now(),
                processing:        false,
                nonce:             0,
            });
        debug!(
            "proto id [{}] open on session [{}], address: [{}], type: [{:?}], version: {}",
            context.proto_id, session.id, session.address, session.ty, version
        );
        debug!("connected sessions are: {:?}", self.connected_session_ids);
        self.peer_manager.open_protocol(
            &extract_peer_id(&session.address).unwrap(),
            crate::protocols::SupportProtocols::Ping.protocol_id(),
        )
    }

    async fn disconnected(&mut self, context: ProtocolContextMutRef<'_>) {
        let session = context.session;
        self.connected_session_ids.remove(&session.id);

        debug!(
            "proto id [{}] close on session [{}]",
            context.proto_id, session.id
        );
        self.peer_manager.close_protocol(
            &extract_peer_id(&session.address).unwrap(),
            &crate::protocols::SupportProtocols::Ping.protocol_id(),
        )
    }

    async fn received(&mut self, context: ProtocolContextMutRef<'_>, data: Bytes) {
        let session = context.session;
        match PingMessage::decode(data).ok() {
            None => {}
            Some(PingMessage { payload: None }) => {}
            Some(PingMessage { payload: Some(pld) }) => match pld {
                PingPayload::Ping(nonce) => {
                    self.ping_received(session.id);
                    let pong = PingMessage::new_pong(nonce).into_bytes();
                    if let Err(err) = context.send_message(pong).await {
                        debug!("send message {}", err);
                    }
                }
                PingPayload::Pong(nonce) => {
                    // check pong
                    if let Some(status) = self.connected_session_ids.get_mut(&session.id) {
                        if (true, nonce) == (status.processing, status.nonce()) {
                            status.processing = false;
                            let last_ping_sent_at = status.last_ping_sent_at;
                            self.pong_received(session.id, last_ping_sent_at);
                            return;
                        }
                        if let Err(err) = context.disconnect(session.id).await {
                            debug!("send message {}", err);
                        }
                    }
                }
            },
        }
    }

    async fn notify(&mut self, context: &mut ProtocolContext, token: u64) {
        match token {
            SEND_PING_TOKEN => self.ping_peers(context).await,
            CHECK_TIMEOUT_TOKEN => {
                let timeout = self.timeout;
                for (id, _ps) in self
                    .connected_session_ids
                    .iter()
                    .filter(|(_id, ps)| ps.processing && ps.elapsed() >= timeout)
                {
                    debug!("ping timeout, {:?}", id);
                    let _ = context.disconnect(*id).await;
                }
            }
            _ => panic!("unknown token {}", token),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Oneof)]
pub enum PingPayload {
    #[prost(uint32, tag = "1")]
    Ping(u32),
    #[prost(uint32, tag = "2")]
    Pong(u32),
}

#[derive(Clone, PartialEq, Message)]
pub struct PingMessage {
    #[prost(oneof = "PingPayload", tags = "1, 2")]
    pub payload: Option<PingPayload>,
}

impl PingMessage {
    pub fn new_pong(nonce: u32) -> Self {
        PingMessage {
            payload: Some(PingPayload::Pong(nonce)),
        }
    }

    pub fn new_ping(nonce: u32) -> Self {
        PingMessage {
            payload: Some(PingPayload::Ping(nonce)),
        }
    }

    pub fn into_bytes(self) -> Bytes {
        let mut buf = BytesMut::with_capacity(self.encoded_len());
        self.encode(&mut buf).unwrap();

        buf.freeze()
    }
}
