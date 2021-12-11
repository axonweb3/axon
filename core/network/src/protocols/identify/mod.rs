mod protocol;

use self::protocol::{AddressInfo, Identity};
use log::{debug, trace, warn};
use prost::Message;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tentacle::{
    bytes::Bytes,
    context::{ProtocolContext, ProtocolContextMutRef, SessionContext},
    multiaddr::Multiaddr,
    secio::PeerId,
    service::TargetProtocol,
    traits::ServiceProtocol,
    utils::{extract_peer_id, is_reachable, multiaddr_to_socketaddr},
    SessionId,
};

use crate::peer_manager::PeerManager;

const MAX_RETURN_LISTEN_ADDRS: usize = 10;
const BAN_ON_NOT_SAME_NET: Duration = Duration::from_secs(5 * 60);
const CHECK_TIMEOUT_TOKEN: u64 = 100;
// Check timeout interval (seconds)
const CHECK_TIMEOUT_INTERVAL: u64 = 1;
const DEFAULT_TIMEOUT: u64 = 8;
const MAX_ADDRS: usize = 10;

/// The misbehavior to report to underlying peer storage
pub enum Misbehavior {
    /// Repeat received message
    DuplicateReceived,
    /// Timeout reached
    Timeout,
    /// Remote peer send invalid data
    InvalidData,
    /// Send too many addresses in listen addresses
    TooManyAddresses(usize),
}

/// Misbehavior report result
pub enum MisbehaveResult {
    /// Continue to run
    Continue,
    /// Disconnect this peer
    Disconnect,
}

impl MisbehaveResult {
    pub fn is_disconnect(&self) -> bool {
        matches!(self, MisbehaveResult::Disconnect)
    }
}

pub(crate) struct RemoteInfo {
    peer_id:      PeerId,
    session:      SessionContext,
    connected_at: Instant,
    timeout:      Duration,
    has_received: bool,
}

impl RemoteInfo {
    fn new(session: SessionContext, timeout: Duration) -> RemoteInfo {
        let peer_id = session
            .remote_pubkey
            .as_ref()
            .map(PeerId::from_public_key)
            .expect("secio must enabled!");
        RemoteInfo {
            peer_id,
            session,
            connected_at: Instant::now(),
            timeout,
            has_received: false,
        }
    }
}

pub struct IdentifyProtocol {
    remote_infos:   HashMap<SessionId, RemoteInfo>,
    global_ip_only: bool,
    peer_manager:   PeerManager,
    identify:       Option<Bytes>,
}

impl IdentifyProtocol {
    pub fn new(peer_manager: PeerManager) -> IdentifyProtocol {
        IdentifyProtocol {
            remote_infos: HashMap::default(),
            global_ip_only: true,
            peer_manager,
            identify: None,
        }
    }

    fn check_duplicate(&mut self, context: &mut ProtocolContextMutRef) -> MisbehaveResult {
        let session = context.session;
        let info = self
            .remote_infos
            .get_mut(&session.id)
            .expect("RemoteInfo must exists");

        if info.has_received {
            debug!("remote({:?}) repeat send identify", info.peer_id);
            let _error = Misbehavior::DuplicateReceived;
            MisbehaveResult::Disconnect
        } else {
            info.has_received = true;
            MisbehaveResult::Continue
        }
    }

    fn received_identify(
        &self,
        context: &mut ProtocolContextMutRef,
        chain_id: &str,
    ) -> MisbehaveResult {
        if self.peer_manager.chain_id() == chain_id {
            if context.session.ty.is_outbound() {
                let _ = context.open_protocols(context.session.id, TargetProtocol::All);
            }
            MisbehaveResult::Continue
        } else {
            MisbehaveResult::Disconnect
        }
    }

    fn process_listens(
        &mut self,
        context: &mut ProtocolContextMutRef,
        listens: Vec<Multiaddr>,
    ) -> MisbehaveResult {
        let session = context.session;
        let _info = self
            .remote_infos
            .get_mut(&session.id)
            .expect("RemoteInfo must exists");

        if listens.len() > MAX_ADDRS {
            let _error = Misbehavior::TooManyAddresses(listens.len());
            MisbehaveResult::Disconnect
        } else {
            trace!("received listen addresses: {:?}", listens);
            let global_ip_only = self.global_ip_only;
            let _reachable_addrs = listens
                .into_iter()
                .filter(|addr| {
                    multiaddr_to_socketaddr(addr)
                        .map(|socket_addr| !global_ip_only || is_reachable(socket_addr.ip()))
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();
            // self.callback
            //     .add_remote_listen_addrs(session.id, reachable_addrs);
            MisbehaveResult::Continue
        }
    }

    fn process_observed(
        &mut self,
        context: &mut ProtocolContextMutRef,
        observed: Option<Multiaddr>,
    ) -> MisbehaveResult {
        if observed.is_none() {
            return MisbehaveResult::Continue;
        }
        let observed = observed.unwrap();
        let session = context.session;
        let _info = self
            .remote_infos
            .get_mut(&session.id)
            .expect("RemoteInfo must exists");

        trace!("received observed address: {}", observed);

        // let global_ip_only = self.global_ip_only;
        // if multiaddr_to_socketaddr(&observed)
        //     .map(|socket_addr| socket_addr.ip())
        //     .filter(|ip_addr| !global_ip_only || is_reachable(*ip_addr))
        //     .is_some()
        //     && self
        //         .callback
        //         .add_observed_addr(&info.peer_id, observed, info.session.ty)
        //         .is_disconnect()
        // {
        //     return MisbehaveResult::Disconnect;
        // }
        MisbehaveResult::Continue
    }
}

impl ServiceProtocol for IdentifyProtocol {
    fn init(&mut self, context: &mut ProtocolContext) {
        let proto_id = context.proto_id;
        if context
            .set_service_notify(
                proto_id,
                Duration::from_secs(CHECK_TIMEOUT_INTERVAL),
                CHECK_TIMEOUT_TOKEN,
            )
            .is_err()
        {
            warn!("identify start fail")
        }
    }

    fn connected(&mut self, context: ProtocolContextMutRef, _version: &str) {
        let session = context.session;

        self.peer_manager.open_protocol(
            &extract_peer_id(&session.address).unwrap(),
            crate::protocols::SupportProtocols::Identify.protocol_id(),
        );

        let remote_info = RemoteInfo::new(session.clone(), Duration::from_secs(DEFAULT_TIMEOUT));
        trace!("IdentifyProtocol connected from {:?}", remote_info.peer_id);
        self.remote_infos.insert(session.id, remote_info);

        // let listen_addrs: Vec<Multiaddr> = self
        //     .callback
        //     .local_listen_addrs()
        //     .iter()
        //     .filter(|addr| {
        //         multiaddr_to_socketaddr(addr)
        //             .map(|socket_addr| !self.global_ip_only ||
        // is_reachable(socket_addr.ip()))             .unwrap_or(false)
        //     })
        //     .take(MAX_ADDRS)
        //     .cloned()
        //     .collect();

        let data = match self.identify.clone() {
            Some(d) => d,
            None => {
                let d = Identity::new(
                    self.peer_manager.chain_id(),
                    AddressInfo::new(Vec::new(), session.address.clone()),
                )
                .into_bytes();
                self.identify = Some(d.clone());
                d
            }
        };

        let _ = context.quick_send_message(data);
    }

    fn disconnected(&mut self, context: ProtocolContextMutRef) {
        let info = self
            .remote_infos
            .remove(&context.session.id)
            .expect("RemoteInfo must exists");
        trace!("IdentifyProtocol disconnected from {:?}", info.peer_id);
        self.peer_manager.close_protocol(
            &extract_peer_id(&context.session.address).unwrap(),
            &crate::protocols::SupportProtocols::Identify.protocol_id(),
        );
    }

    fn received(&mut self, mut context: ProtocolContextMutRef, data: Bytes) {
        let session = context.session;

        match Identity::decode(data).ok() {
            Some(message) => match message.addr_info {
                Some(addr_info) => {
                    if self.check_duplicate(&mut context).is_disconnect()
                        || self
                            .received_identify(&mut context, &message.chain_id)
                            .is_disconnect()
                        || self
                            .process_listens(&mut context, addr_info.listen_addrs())
                            .is_disconnect()
                        || self
                            .process_observed(&mut context, addr_info.observed_addr())
                            .is_disconnect()
                    {
                        let _ = context.disconnect(session.id);
                    }
                }
                None => {
                    let _ = context.disconnect(session.id);
                }
            },
            None => {
                let info = self
                    .remote_infos
                    .get(&session.id)
                    .expect("RemoteInfo must exists");
                debug!(
                    "IdentifyProtocol received invalid data from {:?}",
                    info.peer_id
                );
                let _ = context.disconnect(session.id);
            }
        }
    }

    fn notify(&mut self, context: &mut ProtocolContext, _token: u64) {
        for (session_id, info) in &self.remote_infos {
            if !info.has_received && (info.connected_at + info.timeout) <= Instant::now() {
                debug!("{:?} receive identify message timeout", info.peer_id);
                let _ = context.disconnect(*session_id);
            }
        }
    }
}
