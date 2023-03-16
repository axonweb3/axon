use std::sync::Arc;

use tentacle::secio::PeerId;
use tentacle::{
    service::{ServiceAsyncControl, TargetProtocol, TargetSession},
    SessionId,
};

use common_apm::tracing::AxonTracer;

use protocol::rand::{self, Rng};
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
            log::debug!("no trace id found for gossip {}", endpoint.full_url());
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
                .filter_broadcast(
                    target_session,
                    crate::protocols::SupportProtocols::Transmitter.protocol_id(),
                    data,
                )
                .await
                .unwrap(),
            Priority::High => self
                .transmitter
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
            TargetSession::Multi(Box::new(connected.into_iter())),
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

    async fn gossip<M>(
        &self,
        mut cx: Context,
        origin: Option<usize>,
        endpoint: &str,
        msg: M,
        priority: Priority,
    ) -> ProtocolResult<()>
    where
        M: MessageCodec,
    {
        let msg = self.package_message(cx.clone(), endpoint, msg).await?;
        let ctx = cx.set_url(endpoint.to_owned());
        let mut r = RandomGossip::random();
        let target = match origin {
            Some(id) => TargetSession::Filter(Box::new(move |i| {
                if &Into::<SessionId>::into(id) == i {
                    return false;
                }
                r.next_inner()
            })),
            None => TargetSession::Filter(Box::new(move |_| r.next_inner())),
        };
        self.send_to_sessions(ctx, target, msg, priority).await?;
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

struct RandomGossip {
    index: u8,
}

impl RandomGossip {
    fn random() -> Self {
        Self {
            index: rand::thread_rng().gen_range(0, 3),
        }
    }

    #[cfg(test)]
    fn new(index: u8) -> Self {
        Self { index }
    }

    fn next_inner(&mut self) -> bool {
        if self.index < 2 {
            self.index += 1;
            true
        } else {
            self.index = 0;
            false
        }
    }
}

impl Iterator for RandomGossip {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_inner())
    }
}

#[cfg(test)]
mod test {
    use super::RandomGossip;

    #[test]
    fn test_random_gossip() {
        let a = RandomGossip::new(0);

        assert_eq!(
            vec![true, true, false, true, true, false],
            a.take(6).collect::<Vec<bool>>()
        );

        let a = RandomGossip::new(1);

        assert_eq!(
            vec![true, false, true, true, false, true],
            a.take(6).collect::<Vec<bool>>()
        );

        let a = RandomGossip::new(2);

        assert_eq!(
            vec![false, true, true, false, true, true],
            a.take(6).collect::<Vec<bool>>()
        );
    }
}
