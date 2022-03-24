use std::sync::Arc;

use common_apm::tracing::AxonTracer;
use tentacle::secio::PeerId;
use tentacle::service::{ServiceAsyncControl, TargetProtocol, TargetSession};

use protocol::traits::{Context, Gossip, MessageCodec, Priority};
use protocol::{async_trait, tokio, types::Bytes, ProtocolResult};

use crate::endpoint::Endpoint;
use crate::error::NetworkError;
use crate::message::{Headers, NetworkMessage};
use crate::peer_manager::PeerManager;
use crate::traits::NetworkContext;

#[derive(Clone)]
pub struct NetworkGossip {
    pub(crate) transmitter:  ServiceAsyncControl,
    pub(crate) peer_manager: Arc<PeerManager>,
}

impl NetworkGossip {
    pub fn new(transmitter: ServiceAsyncControl, peer_manager: Arc<PeerManager>) -> Self {
        NetworkGossip {
            transmitter,
            peer_manager,
        }
    }

    async fn package_message<M>(
        &self,
        ctx: Context,
        endpoint: &str,
        mut msg: M,
    ) -> ProtocolResult<Bytes>
    where
        M: MessageCodec,
    {
        let endpoint = endpoint.parse::<Endpoint>()?;
        let data = msg.encode_msg()?;
        let mut headers = Headers::default();
        if let Some(state) = AxonTracer::span_state(&ctx) {
            headers.set_trace_id(state.trace_id());
            headers.set_span_id(state.span_id());
            log::info!("no trace id found for gossip {}", endpoint.full_url());
        }
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
                .filter_broadcast(
                    target_session,
                    crate::protocols::SupportProtocols::Transmitter.protocol_id(),
                    data,
                )
                .await
                .unwrap(),
            Priority::High => self
                .transmitter
                .clone()
                .quick_filter_broadcast(
                    target_session,
                    crate::protocols::SupportProtocols::Transmitter.protocol_id(),
                    data,
                )
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
            let control = self.transmitter.clone();
            tokio::spawn(async move {
                for addr in unconnected {
                    control
                        .dial(
                            addr,
                            TargetProtocol::Single(
                                crate::protocols::SupportProtocols::Identify.protocol_id(),
                            ),
                        )
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
        common_apm::metrics::network::on_network_message_sent_all_target(endpoint);
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
        let multicast_count = peer_ids.as_ref().len();

        let ctx = cx.set_url(endpoint.to_owned());
        self.send_to_peers(ctx, peer_ids, msg, priority).await?;
        common_apm::metrics::network::on_network_message_sent_multi_target(
            endpoint,
            multicast_count as f64,
        );
        Ok(())
    }
}
