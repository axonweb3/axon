use std::{collections::HashSet, sync::Arc, time::Duration};

use rand::prelude::IteratorRandom;
use tentacle::{
    builder::ServiceBuilder,
    context::ServiceContext,
    error::{DialerErrorKind, HandshakeErrorKind, ProtocolHandleErrorKind},
    multiaddr::Multiaddr,
    secio::{error::SecioError, PeerId},
    service::{
        ProtocolHandle, Service, ServiceAsyncControl, ServiceError, ServiceEvent, SessionType,
        TargetProtocol, TcpSocket,
    },
    traits::ServiceHandle,
    utils::{extract_peer_id, is_reachable, multiaddr_to_socketaddr},
    yamux::Config as YamuxConfig,
};

use protocol::tokio::time::{Instant, MissedTickBehavior};
use protocol::{
    async_trait, tokio,
    traits::{
        Context, Gossip, MessageCodec, MessageHandler, Network, PeerTag, PeerTrust, Priority, Rpc,
        TrustFeedback,
    },
    types::Bytes,
    ProtocolResult,
};

use crate::{
    config::NetworkConfig,
    endpoint::{Endpoint, EndpointScheme},
    error::NetworkError,
    outbound::{NetworkGossip, NetworkRpc},
    peer_manager::{AddrInfo, PeerInfo, PeerManager, PeerStore},
    protocols::{
        DiscoveryAddressManager, DiscoveryProtocol, Feeler, IdentifyProtocol, PingHandler,
        SupportProtocols, TransmitterProtocol,
    },
    reactor::MessageRouter,
};

#[derive(Clone)]
pub struct NetworkServiceHandle {
    gossip: NetworkGossip,
    rpc:    NetworkRpc,
}

#[async_trait]
impl Gossip for NetworkServiceHandle {
    async fn broadcast<M>(&self, cx: Context, end: &str, msg: M, p: Priority) -> ProtocolResult<()>
    where
        M: MessageCodec,
    {
        self.gossip.broadcast(cx, end, msg, p).await
    }

    async fn gossip<M>(
        &self,
        cx: Context,
        origin: Option<usize>,
        end: &str,
        msg: M,
        p: Priority,
    ) -> ProtocolResult<()>
    where
        M: MessageCodec,
    {
        self.gossip.gossip(cx, origin, end, msg, p).await
    }

    async fn multicast<'a, M, P>(
        &self,
        cx: Context,
        end: &str,
        peer_ids: P,
        msg: M,
        p: Priority,
    ) -> ProtocolResult<()>
    where
        M: MessageCodec,
        P: AsRef<[Bytes]> + Send + 'a,
    {
        self.gossip.multicast(cx, end, peer_ids, msg, p).await
    }
}

#[async_trait]
impl Rpc for NetworkServiceHandle {
    async fn call<M, R>(&self, cx: Context, end: &str, msg: M, p: Priority) -> ProtocolResult<R>
    where
        M: MessageCodec,
        R: MessageCodec,
    {
        self.rpc.call(cx, end, msg, p).await
    }

    async fn response<M>(
        &self,
        cx: Context,
        end: &str,
        msg: ProtocolResult<M>,
        p: Priority,
    ) -> ProtocolResult<()>
    where
        M: MessageCodec,
    {
        self.rpc.response(cx, end, msg, p).await
    }
}

impl PeerTrust for NetworkServiceHandle {
    fn report(&self, _ctx: Context, _feedback: TrustFeedback) {}
}

impl Network for NetworkServiceHandle {
    fn tag(&self, _ctx: Context, peer_id: Bytes, tag: PeerTag) -> ProtocolResult<()> {
        let peer_id = PeerId::from_bytes(peer_id.as_ref().to_vec())
            .map_err(|_| NetworkError::InvalidPeerId)?;

        match tag {
            PeerTag::Consensus => {
                self.gossip
                    .peer_manager
                    .consensus_list
                    .write()
                    .insert(peer_id);
            }
            PeerTag::Ban { until } => {
                if let Some(id) = self.gossip.peer_manager.ban_id(
                    &peer_id,
                    until,
                    "ban from other module".to_string(),
                ) {
                    let sender = self.gossip.transmitter.clone();
                    tokio::spawn(async move {
                        let _ignore = sender.disconnect(id).await;
                    });
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn untag(&self, _ctx: Context, peer_id: Bytes, tag: &PeerTag) -> ProtocolResult<()> {
        let peer_id =
            PeerId::from_bytes(peer_id.to_vec()).map_err(|_| NetworkError::InvalidPeerId)?;

        if let PeerTag::Consensus = tag {
            self.gossip
                .peer_manager
                .consensus_list
                .write()
                .remove(&peer_id);
        }

        Ok(())
    }

    fn tag_consensus(&self, _ctx: Context, peer_ids: Vec<Bytes>) -> ProtocolResult<()> {
        let mut peer_ids: HashSet<PeerId> = {
            let byteses = peer_ids.iter();
            let maybe_ids = byteses.map(|bytes| {
                PeerId::from_bytes(bytes.as_ref().to_vec()).map_err(|_| NetworkError::InvalidPeerId)
            });

            maybe_ids.collect::<Result<HashSet<_>, _>>()?
        };

        std::mem::swap(
            &mut *self.gossip.peer_manager.consensus_list.write(),
            &mut peer_ids,
        );

        Ok(())
    }

    fn peer_count(&self, _ctx: Context) -> ProtocolResult<usize> {
        Ok(self
            .gossip
            .peer_manager
            .with_registry(|reg| reg.peers.len()))
    }
}

pub struct NetworkService {
    // Config backup
    config: Arc<NetworkConfig>,

    // Public service components
    gossip: NetworkGossip,
    rpc:    NetworkRpc,

    // Core service
    peer_mgr_handle: Arc<PeerManager>,
    net:             Option<Service<ServiceHandler>>,

    control:            ServiceAsyncControl,
    try_identify_count: u8,
}

impl NetworkService {
    pub fn new(config: NetworkConfig) -> Self {
        let config = Arc::new(config);
        let peer_manager = Arc::new(PeerManager::new(Arc::clone(&config)));
        let service_handle = ServiceHandler {
            peer_store: Arc::clone(&peer_manager),
            config:     Arc::clone(&config),
        };
        let message_router = MessageRouter::new();

        let mut protocol_meta = Vec::new();

        let ping_peer_manager = Arc::clone(&peer_manager);
        let ping_handle =
            PingHandler::new(config.ping_interval, config.ping_timeout, ping_peer_manager);
        let ping = SupportProtocols::Ping
            .build_meta_with_service_handle(|| ProtocolHandle::Callback(Box::new(ping_handle)));
        protocol_meta.push(ping);

        let identify_peer_manager = Arc::clone(&peer_manager);
        let identify = SupportProtocols::Identify.build_meta_with_service_handle(move || {
            ProtocolHandle::Callback(Box::new(IdentifyProtocol::new(identify_peer_manager)))
        });
        protocol_meta.push(identify);

        let discovery_peer_manager = DiscoveryAddressManager::new(Arc::clone(&peer_manager));
        let discovery = SupportProtocols::Discovery.build_meta_with_service_handle(move || {
            ProtocolHandle::Callback(Box::new(DiscoveryProtocol::new(
                discovery_peer_manager,
                None,
            )))
        });
        protocol_meta.push(discovery);

        let transmitter_peer_manager = Arc::clone(&peer_manager);
        let transmitter_router = message_router.clone();
        let transmitter = SupportProtocols::Transmitter.build_meta_with_service_handle(move || {
            ProtocolHandle::Callback(Box::new(TransmitterProtocol::new(
                transmitter_router,
                transmitter_peer_manager,
            )))
        });
        protocol_meta.push(transmitter);

        let feeler_peer_manager = Arc::clone(&peer_manager);
        let feeler = SupportProtocols::Feeler.build_meta_with_service_handle(move || {
            ProtocolHandle::Callback(Box::new(Feeler::new(feeler_peer_manager)))
        });
        protocol_meta.push(feeler);

        let mut service_builder = ServiceBuilder::new();
        let yamux_config = YamuxConfig {
            max_stream_count: protocol_meta.len(),
            max_stream_window_size: 1024 * 1024,
            ..Default::default()
        };

        for meta in protocol_meta {
            service_builder = service_builder.insert_protocol(meta);
        }

        service_builder = service_builder
            .key_pair(config.secio_keypair.clone())
            .yamux_config(yamux_config)
            .forever(true)
            .max_connection_number(config.max_connections)
            .set_send_buffer_size(config.send_buffer_size)
            .set_recv_buffer_size(config.recv_buffer_size)
            .set_channel_size(1024)
            .timeout(Duration::from_secs(5));
        #[cfg(target_os = "linux")]
        let service_builder = {
            let addr = multiaddr_to_socketaddr(&config.default_listen).unwrap();
            service_builder.tcp_config(move |socket: TcpSocket| {
                let socket_ref = socket2::SockRef::from(&socket);

                #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
                socket_ref.set_reuse_port(true)?;

                socket_ref.set_reuse_address(true)?;
                socket_ref.bind(&addr.into())?;
                Ok(socket)
            })
        };

        let service = service_builder.build(service_handle);

        let control: ServiceAsyncControl = service.control().clone();

        let gossip = NetworkGossip::new(control.clone(), Arc::clone(&peer_manager));
        let rpc = NetworkRpc::new(control, message_router);

        NetworkService {
            config,
            gossip,
            rpc,
            peer_mgr_handle: peer_manager,
            control: service.control().clone(),
            net: Some(service),
            try_identify_count: 0,
        }
    }

    pub fn register_endpoint_handler<M>(
        &mut self,
        end: &str,
        handler: impl MessageHandler<Message = M>,
    ) -> ProtocolResult<()>
    where
        M: MessageCodec,
    {
        let endpoint = end.parse::<Endpoint>()?;
        if endpoint.scheme() == EndpointScheme::RpcResponse {
            let err = "use register_rpc_response() instead".to_owned();

            return Err(NetworkError::UnexpectedScheme(err).into());
        }

        self.rpc.router.register_reactor(endpoint, handler);
        Ok(())
    }

    // Currently rpc response dont invoke message handler, so we create a dummy
    // for it.
    pub fn register_rpc_response(&mut self, end: &str) -> ProtocolResult<()> {
        let endpoint = end.parse::<Endpoint>()?;
        if endpoint.scheme() != EndpointScheme::RpcResponse {
            return Err(NetworkError::UnexpectedScheme(end.to_owned()).into());
        }

        self.rpc.router.register_rpc_response(endpoint);
        Ok(())
    }

    pub fn handle(&self) -> NetworkServiceHandle {
        NetworkServiceHandle {
            gossip: self.gossip.clone(),
            rpc:    self.rpc.clone(),
        }
    }

    pub fn peer_id(&self) -> PeerId {
        self.config.secio_keypair.peer_id()
    }

    pub fn set_chain_id(&self, chain_id: String) {
        self.peer_mgr_handle.set_chain_id(chain_id);
    }

    /// Dial just feeler protocol
    pub async fn dial_feeler(&mut self, addr: Multiaddr) {
        let peer_id = extract_peer_id(&addr).unwrap();
        let can_dial = self.peer_mgr_handle.with_registry_mut(|reg| {
            !reg.peers.contains_key(&peer_id)
                && !reg.dialing.contains(&addr)
                && reg.add_feeler(addr.clone())
        });
        if can_dial {
            let _ignore = self
                .control
                .dial(
                    addr.clone(),
                    TargetProtocol::Single(SupportProtocols::Identify.protocol_id()),
                )
                .await;
        }
    }

    /// Dial just identify protocol
    pub async fn dial_identify(&mut self, addr: Multiaddr) {
        let peer_id = extract_peer_id(&addr).unwrap();
        let can_dial = self.peer_mgr_handle.with_registry_mut(|reg| {
            !reg.peers.contains_key(&peer_id)
                && !reg.is_feeler(&addr)
                && reg.dialing.insert(addr.clone())
        });
        if can_dial {
            let _ignore = self
                .control
                .dial(
                    addr.clone(),
                    TargetProtocol::Single(SupportProtocols::Identify.protocol_id()),
                )
                .await;
        }
    }

    async fn try_dial_observed_addr(&mut self) {
        let addr = {
            let addrs = self.peer_mgr_handle.public_addrs.read();
            if addrs.is_empty() {
                return;
            }
            // random get addr
            addrs.iter().choose(&mut rand::thread_rng()).cloned()
        };

        if let Some(addr) = addr {
            self.dial_identify(addr).await;
        }
    }

    async fn try_dial_feeler(&mut self) {
        let now_ms = faketime::unix_time_as_millis();
        let attempt_peers = self.peer_mgr_handle.with_peer_store_mut(|peer_store| {
            let paddrs = peer_store.fetch_addrs_to_feeler(10);
            for paddr in paddrs.iter() {
                // mark addr as tried
                if let Some(paddr) = peer_store.mut_addr_manager().get_mut(&paddr.addr) {
                    paddr.mark_tried(now_ms);
                }
            }
            paddrs
        });

        log::trace!(
            "feeler dial count={}, attempt_peers: {:?}",
            attempt_peers.len(),
            attempt_peers,
        );

        for addr in attempt_peers.into_iter().map(|info| info.addr) {
            self.dial_feeler(addr).await;
        }
    }

    async fn try_dial_peers(&mut self) {
        let status = self
            .peer_mgr_handle
            .with_registry(|reg| reg.connection_status());
        let count = (self.config.max_connections - self.config.inbound_conn_limit)
            .saturating_sub(status.inbound) as usize;
        if count == 0 {
            self.try_identify_count = 0;
            return;
        }
        self.try_identify_count += 1;

        let f = |peer_store: &mut PeerStore, number: usize, now_ms: u64| -> Vec<AddrInfo> {
            let paddrs = peer_store.fetch_addrs_to_attempt(number);
            for paddr in paddrs.iter() {
                // mark addr as tried
                if let Some(paddr) = peer_store.mut_addr_manager().get_mut(&paddr.addr) {
                    paddr.mark_tried(now_ms);
                }
            }
            paddrs
        };

        let peers: Box<dyn Iterator<Item = Multiaddr> + Send> = if self.try_identify_count > 3 {
            self.try_identify_count = 0;
            let bootnodes = self.peer_mgr_handle.unconnected_bootstraps();
            let len = bootnodes.len();
            if len < count {
                let now_ms = faketime::unix_time_as_millis();
                let attempt_peers = self
                    .peer_mgr_handle
                    .with_peer_store_mut(|peer_store| f(peer_store, count - len, now_ms));

                Box::new(
                    attempt_peers
                        .into_iter()
                        .map(|info| info.addr)
                        .chain(bootnodes.into_iter()),
                )
            } else {
                Box::new(
                    bootnodes
                        .into_iter()
                        .choose_multiple(&mut rand::thread_rng(), count)
                        .into_iter(),
                )
            }
        } else {
            let now_ms = faketime::unix_time_as_millis();
            let attempt_peers = self
                .peer_mgr_handle
                .with_peer_store_mut(|peer_store| f(peer_store, count, now_ms));

            log::trace!(
                "identify dial count={}, attempt_peers: {:?}",
                attempt_peers.len(),
                attempt_peers,
            );

            Box::new(attempt_peers.into_iter().map(|info| info.addr))
        };

        for addr in peers {
            self.dial_identify(addr).await;
        }
    }

    async fn try_dial_consensus(&mut self) {
        let addrs = self.peer_mgr_handle.unconnected_consensus_peer();

        for addr in addrs {
            self.dial_identify(addr).await;
        }
    }

    #[allow(clippy::unnecessary_to_owned)]
    pub async fn run(mut self) {
        if let Some(mut net) = self.net.take() {
            net.listen(self.config.default_listen.clone())
                .await
                .unwrap();

            for addr in self.config.bootstraps.to_vec() {
                self.dial_identify(addr).await;
            }

            tokio::spawn(async move { net.run().await });
        }

        let mut interval = tokio::time::interval_at(Instant::now(), Duration::from_secs(10));
        let mut dump_interval =
            tokio::time::interval_at(Instant::now(), Duration::from_secs(3600 * 24));

        dump_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.try_dial_consensus().await;
                    self.try_dial_peers().await;
                    self.try_dial_feeler().await;
                    self.try_dial_observed_addr().await;
                }
                _ = dump_interval.tick() => {
                    self.peer_mgr_handle.with_peer_store(|store|{
                        let _ignore = store.dump_to_dir(self.config.peer_store_path.clone())
                            .map_err(|e| log::info!("dump peer store error: {:?}", e));
                    })
                }
                else => {
                    let _ = self.control.shutdown().await;
                    break
                }
            }
        }
    }
}

struct ServiceHandler {
    peer_store: Arc<PeerManager>,
    config:     Arc<NetworkConfig>,
}

#[async_trait]
impl ServiceHandle for ServiceHandler {
    async fn handle_error(&mut self, control: &mut ServiceContext, error: ServiceError) {
        match error {
            ServiceError::DialerError { address, error } => {
                self.peer_store.with_registry_mut(|reg| {
                    reg.remove_feeler(&address);
                    reg.dialing.remove(&address)
                });
                let mut public_addrs = self.peer_store.public_addrs.write();
                match error {
                    DialerErrorKind::HandshakeError(HandshakeErrorKind::SecioError(
                        SecioError::ConnectSelf,
                    )) => {
                        log::debug!("dial observed address success: {:?}", address);
                        if let Some(ip) = multiaddr_to_socketaddr(&address) {
                            if is_reachable(ip.ip()) {
                                public_addrs.insert(address);
                            }
                        }
                    }
                    DialerErrorKind::IoError(e)
                        if e.kind() == std::io::ErrorKind::AddrNotAvailable =>
                    {
                        log::warn!("DialerError({}) {}", address, e);
                    }
                    _ => {
                        log::debug!("DialerError({:?}) {:?}", address, error);
                    }
                }
            }
            ServiceError::ProtocolError {
                id,
                proto_id,
                error,
            } => {
                log::debug!("ProtocolError({}, {}) {}", id, proto_id, error);
                let message = format!("ProtocolError id={}", proto_id);
                // Ban because misbehave of remote peer
                self.peer_store.ban_session_id(
                    id,
                    Duration::from_secs(300).as_millis() as u64,
                    message,
                );
                let _ignore = control.disconnect(id).await;
            }
            ServiceError::SessionTimeout { session_context } => {
                log::warn!(
                    "SessionTimeout({}, {})",
                    session_context.id,
                    session_context.address,
                );
            }
            ServiceError::MuxerError {
                session_context,
                error,
            } => {
                log::debug!(
                    "MuxerError({}, {}), substream error {}, disconnect it",
                    session_context.id,
                    session_context.address,
                    error,
                );
            }
            ServiceError::ListenError { address, error } => {
                log::warn!("ListenError: address={:?}, error={:?}", address, error);
            }
            ServiceError::ProtocolSelectError {
                proto_name,
                session_context,
            } => {
                log::debug!(
                    "ProtocolSelectError: proto_name={:?}, session_id={}",
                    proto_name,
                    session_context.id,
                );
            }
            ServiceError::SessionBlocked { session_context } => {
                log::debug!("SessionBlocked: {}", session_context.id);
            }
            ServiceError::ProtocolHandleError { proto_id, error } => {
                log::debug!("ProtocolHandleError: {:?}, proto_id: {}", error, proto_id);

                let ProtocolHandleErrorKind::AbnormallyClosed(opt_session_id) = error;
                if let Some(id) = opt_session_id {
                    self.peer_store.ban_session_id(
                        id,
                        Duration::from_secs(300).as_millis() as u64,
                        format!("protocol {} panic when process peer message", proto_id),
                    );
                }
                log::warn!("ProtocolHandleError: {:?}, proto_id: {}", error, proto_id);
                std::process::exit(-1)
            }
        }
    }

    async fn handle_event(&mut self, control: &mut ServiceContext, event: ServiceEvent) {
        match event {
            ServiceEvent::SessionOpen { session_context } => {
                let (feeler, status) = self.peer_store.with_registry_mut(|reg| {
                    reg.dialing.remove(&session_context.address);
                    (
                        reg.is_feeler(&session_context.address),
                        reg.connection_status(),
                    )
                });
                if feeler {
                    return;
                }
                let disable = status.total + 1 > self.config.max_connections
                    || match session_context.ty {
                        SessionType::Inbound => status.inbound + 1 > self.config.inbound_conn_limit,
                        SessionType::Outbound => {
                            status.outbound + 1
                                > self.config.max_connections - self.config.inbound_conn_limit
                        }
                    };

                if disable && !self.peer_store.always_allow(&session_context.address) {
                    let _ignore = control.disconnect(session_context.id).await;
                } else {
                    self.peer_store.register(PeerInfo::new(session_context))
                }
            }
            ServiceEvent::SessionClose { session_context } => {
                self.peer_store.unregister(&session_context.address)
            }
            ServiceEvent::ListenClose { address } => {
                log::info!("listen stop at: {}", address)
            }
            ServiceEvent::ListenStarted { address } => {
                log::info!("listen start at: {}", address)
            }
        }
    }
}
