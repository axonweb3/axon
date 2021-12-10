use async_trait::async_trait;
use bytes::Bytes;
use protocol::traits::{Context, Gossip, MessageCodec, Priority};
use protocol::ProtocolResult;
use tentacle::secio::PeerId;
use tentacle::service::{ServiceAsyncControl, TargetProtocol, TargetSession};

use crate::endpoint::Endpoint;
use crate::error::NetworkError;
use crate::message::{Headers, NetworkMessage};
use crate::peer_manager::PeerManager;
use crate::protocols::{IDENTIFY_PROTOCOL_ID, TRANSMITTER_PROTOCOL_ID};
use crate::traits::NetworkContext;

#[derive(Clone)]
pub struct NetworkGossip {
    transmitter:  ServiceAsyncControl,
    peer_manager: PeerManager,
}

impl NetworkGossip {
    pub fn new(transmitter: ServiceAsyncControl, peer_manager: PeerManager) -> Self {
        NetworkGossip {
            transmitter,
            peer_manager,
        }
    }

    async fn package_message<M>(
        &self,
        _ctx: Context,
        endpoint: &str,
        mut msg: M,
    ) -> ProtocolResult<Bytes>
    where
        M: MessageCodec,
    {
        let endpoint = endpoint.parse::<Endpoint>()?;
        let data = msg.encode_msg()?;
        let headers = Headers::default();
        // if let Some(state) = common_apm::muta_apm::MutaTracer::span_state(&ctx) {
        //     headers.set_trace_id(state.trace_id());
        //     headers.set_span_id(state.span_id());
        //     log::info!("no trace id found for gossip {}", endpoint.full_url());
        // }
        let msg = NetworkMessage::new(endpoint, data, headers).encode()?;

        Ok(msg)
    }

    async fn send_to_sessions(
        &self,
        _ctx: Context,
        target_session: TargetSession,
        data: Bytes,
        priority: Priority,
    ) -> Result<(), NetworkError> {
        match priority {
            Priority::Normal => self
                .transmitter
                .clone()
                .filter_broadcast(target_session, TRANSMITTER_PROTOCOL_ID.into(), data)
                .await
                .unwrap(),
            Priority::High => self
                .transmitter
                .clone()
                .quick_filter_broadcast(target_session, TRANSMITTER_PROTOCOL_ID.into(), data)
                .await
                .unwrap(),
        }
        Ok(())
    }

    async fn send_to_peers<'a, P: AsRef<[Bytes]> + 'a>(
        &self,
        ctx: Context,
        peer_ids: P,
        data: Bytes,
        priority: Priority,
    ) -> Result<(), NetworkError> {
        let peer_ids: Vec<PeerId> = {
            let byteses = peer_ids.as_ref().iter();
            let maybe_ids = byteses.map(|bytes| {
                PeerId::from_bytes(bytes.as_ref().to_vec()).map_err(|_| NetworkError::InvalidPeerId)
            });

            maybe_ids.collect::<Result<Vec<_>, _>>()?
        };
        let (connected, unconnected) = self.peer_manager.peers(peer_ids);
        if !unconnected.is_empty() {
            let mut control = self.transmitter.clone();
            tokio::spawn(async move {
                for addr in unconnected {
                    control
                        .dial(addr, TargetProtocol::Single(IDENTIFY_PROTOCOL_ID.into()))
                        .await
                        .unwrap()
                }
            });
        }

        self.send_to_sessions(
            ctx,
            TargetSession::Filter(Box::new(move |id| connected.contains(id))),
            data,
            priority,
        )
        .await
    }
}

#[async_trait]
impl Gossip for NetworkGossip {
    async fn broadcast<M>(
        &self,
        mut cx: Context,
        endpoint: &str,
        msg: M,
        priority: Priority,
    ) -> ProtocolResult<()>
    where
        M: MessageCodec,
    {
        let msg = self.package_message(cx.clone(), endpoint, msg).await?;
        let ctx = cx.set_url(endpoint.to_owned());
        self.send_to_sessions(ctx, TargetSession::All, msg, priority)
            .await?;
        // common_apm::metrics::network::on_network_message_sent_all_target(endpoint);
        Ok(())
    }

    async fn multicast<'a, M, P>(
        &self,
        mut cx: Context,
        endpoint: &str,
        peer_ids: P,
        msg: M,
        priority: Priority,
    ) -> ProtocolResult<()>
    where
        M: MessageCodec,
        P: AsRef<[Bytes]> + Send + 'a,
    {
        let msg = self.package_message(cx.clone(), endpoint, msg).await?;
        // let multicast_count = peer_ids.as_ref().len();

        let ctx = cx.set_url(endpoint.to_owned());
        self.send_to_peers(ctx, peer_ids, msg, priority).await?;
        // common_apm::metrics::network::on_network_message_sent_multi_target(
        //     endpoint,
        //     multicast_count as i64,
        // );
        Ok(())
    }
}
