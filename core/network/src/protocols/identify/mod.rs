mod protocol;

use self::protocol::{AddressInfo, Identity};
use log::{debug, trace, warn};
use prost::Message;
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tentacle::{
    async_trait,
    bytes::Bytes,
    context::{ProtocolContext, ProtocolContextMutRef, SessionContext},
    multiaddr::{Multiaddr, Protocol},
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
    peer_manager:   Arc<PeerManager>,
}

impl IdentifyProtocol {
    pub fn new(peer_manager: Arc<PeerManager>) -> IdentifyProtocol {
        IdentifyProtocol {
            remote_infos: HashMap::default(),
            global_ip_only: true,
            peer_manager,
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

    async fn received_identify(
        &self,
        context: &mut ProtocolContextMutRef<'_>,
        chain_id: &str,
    ) -> MisbehaveResult {
        if self.peer_manager.chain_id() == chain_id {
            if context.session.ty.is_outbound() {
                if self
                    .peer_manager
                    .with_registry(|reg| reg.is_feeler(&context.session.address))
                {
                    let _ = context
                        .open_protocols(
                            context.session.id,
                            TargetProtocol::Single(
                                crate::protocols::SupportProtocols::Feeler.protocol_id(),
                            ),
                        )
                        .await;
                } else {
                    let _ = context
                        .open_protocols(
                            context.session.id,
                            TargetProtocol::Filter(Box::new(|id| {
                                id != &crate::protocols::SupportProtocols::Feeler.protocol_id()
                            })),
                        )
                        .await;
                }
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
        if listens.len() > MAX_ADDRS {
            let _error = Misbehavior::TooManyAddresses(listens.len());
            MisbehaveResult::Disconnect
        } else {
            trace!("received listen addresses: {:?}", listens);
            let global_ip_only = self.global_ip_only;
            let reachable_addrs = listens
                .into_iter()
                .filter(|addr| {
                    multiaddr_to_socketaddr(addr)
                        .map(|socket_addr| !global_ip_only || is_reachable(socket_addr.ip()))
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();
            self.peer_manager.with_peer_store_mut(|peer_store| {
                    for addr in reachable_addrs.iter() {
                        if let Err(err) = peer_store.add_addr(addr.clone()) {
                            log::error!("IdentifyProtocol failed to add address to peer store, address: {}, error: {:?}", addr, err);
                        }
                    }
                });
            let peer_id = extract_peer_id(&context.session.address).unwrap();
            self.peer_manager.with_registry_mut(|reg| {
                if let Some(info) = reg.peers.get_mut(&peer_id) {
                    info.listens = reachable_addrs;
                }
            });
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
        let mut observed = observed.unwrap();
        let session = context.session;
        let _info = self
            .remote_infos
            .get_mut(&session.id)
            .expect("RemoteInfo must exists");

        trace!("received observed address: {}", observed);

        let global_ip_only = self.global_ip_only;
        if multiaddr_to_socketaddr(&observed)
            .map(|socket_addr| socket_addr.ip())
            .filter(|ip_addr| !global_ip_only || is_reachable(*ip_addr))
            .is_none()
        {
            return MisbehaveResult::Continue;
        }

        if session.ty.is_inbound() {
            // The address already been discovered by other peer
            return MisbehaveResult::Continue;
        }

        if extract_peer_id(&observed).is_none() {
            observed.push(Protocol::P2P(Cow::Borrowed(
                self.peer_manager.local_peer_id().as_bytes(),
            )))
        }

        let source_addr = observed.clone();
        let observed_addrs_iter = self
            .peer_manager
            .public_addrs(MAX_ADDRS)
            .into_iter()
            .filter_map(|listen_addr| multiaddr_to_socketaddr(&listen_addr))
            .map(|socket_addr| {
                observed
                    .iter()
                    .map(|proto| match proto {
                        Protocol::Tcp(_) => Protocol::Tcp(socket_addr.port()),
                        value => value,
                    })
                    .collect::<Multiaddr>()
            })
            .chain(::std::iter::once(source_addr));

        for addr in observed_addrs_iter {
            let _ignore = context.dial(
                addr,
                TargetProtocol::Single(crate::protocols::SupportProtocols::Identify.protocol_id()),
            );
        }

        MisbehaveResult::Continue
    }
}

#[async_trait]
impl ServiceProtocol for IdentifyProtocol {
    async fn init(&mut self, context: &mut ProtocolContext) {
        let proto_id = context.proto_id;
        if context
            .set_service_notify(
                proto_id,
                Duration::from_secs(CHECK_TIMEOUT_INTERVAL),
                CHECK_TIMEOUT_TOKEN,
            )
            .await
            .is_err()
        {
            warn!("identify start fail")
        }
    }

    async fn connected(&mut self, context: ProtocolContextMutRef<'_>, _version: &str) {
        let session = context.session;

        self.peer_manager.open_protocol(
            &extract_peer_id(&session.address).unwrap(),
            crate::protocols::SupportProtocols::Identify.protocol_id(),
        );

        if context.session.ty.is_outbound() {
            // why don't set inbound here?
            // because inbound address can't feeler during staying connected
            // and if set it to peer store, it will be broadcast to the entire network,
            // but this is an unverified address
            self.peer_manager.with_peer_store_mut(|peer_store| {
                peer_store.add_outbound_addr(context.session.address.clone());
            });
        }

        let remote_info = RemoteInfo::new(session.clone(), Duration::from_secs(DEFAULT_TIMEOUT));
        trace!("IdentifyProtocol connected from {:?}", remote_info.peer_id);
        self.remote_infos.insert(session.id, remote_info);

        let listen_addrs: Vec<Multiaddr> = self
            .peer_manager
            .local_listen_addrs()
            .iter()
            .filter(|addr| {
                multiaddr_to_socketaddr(addr)
                    .map(|socket_addr| !self.global_ip_only || is_reachable(socket_addr.ip()))
                    .unwrap_or(false)
            })
            .take(MAX_ADDRS)
            .cloned()
            .collect();

        let data = Identity::new(
            self.peer_manager.chain_id(),
            AddressInfo::new(listen_addrs, session.address.clone()),
        )
        .into_bytes();

        let _ = context.quick_send_message(data).await;
    }

    async fn disconnected(&mut self, context: ProtocolContextMutRef<'_>) {
        self.remote_infos
            .remove(&context.session.id)
            .expect("RemoteInfo must exists");

        self.peer_manager.close_protocol(
            &extract_peer_id(&context.session.address).unwrap(),
            &crate::protocols::SupportProtocols::Identify.protocol_id(),
        );

        if context.session.ty.is_outbound() {
            // Due to the filtering strategy of the peer store, if the node is
            // disconnected after a long connection is maintained for more than seven days,
            // it is possible that the node will be accidentally evicted, so it is necessary
            // to reset the information of the node when disconnected.
            self.peer_manager.with_peer_store_mut(|peer_store| {
                if !peer_store.is_addr_banned(&context.session.address) {
                    peer_store.add_outbound_addr(context.session.address.clone());
                }
            });
        }
    }

    async fn received(&mut self, mut context: ProtocolContextMutRef<'_>, data: Bytes) {
        let session = context.session;

        match Identity::decode(data).ok() {
            Some(message) => match message.addr_info {
                Some(addr_info) => {
                    if self.check_duplicate(&mut context).is_disconnect()
                        || self
                            .received_identify(&mut context, &message.chain_id)
                            .await
                            .is_disconnect()
                        || self
                            .process_listens(&mut context, addr_info.listen_addrs())
                            .is_disconnect()
                        || self
                            .process_observed(&mut context, addr_info.observed_addr())
                            .is_disconnect()
                    {
                        let _ = context.disconnect(session.id).await;
                    }
                }
                None => {
                    let _ = context.disconnect(session.id).await;
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
                let _ = context.disconnect(session.id).await;
            }
        }
    }

    async fn notify(&mut self, context: &mut ProtocolContext, _token: u64) {
        for (session_id, info) in &self.remote_infos {
            if !info.has_received && (info.connected_at + info.timeout) <= Instant::now() {
                debug!("{:?} receive identify message timeout", info.peer_id);
                let _ = context.disconnect(*session_id).await;
            }
        }
    }
}
