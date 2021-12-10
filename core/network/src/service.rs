use crate::{
    compress::{compress, decompress},
    config::NetworkConfig,
    endpoint::{Endpoint, EndpointScheme},
    error::NetworkError,
    outbound::{NetworkGossip, NetworkRpc},
    peer_manager::{PeerInfo, PeerManager},
    protocols::{
        DiscoveryAddressManager, DiscoveryProtocol, IdentifyProtocol, PingHandler,
        TransmitterProtocol, DISCOVERY_PROTOCOL_ID, IDENTIFY_PROTOCOL_ID, PING_PROTOCOL_ID,
        TRANSMITTER_PROTOCOL_ID,
    },
    reactor::MessageRouter,
};

use async_trait::async_trait;
use bytes::Bytes;
use futures::StreamExt;
use protocol::{
    traits::{
        Context, Gossip, MessageCodec, MessageHandler, PeerTrust, Priority, Rpc, TrustFeedback,
    },
    ProtocolResult,
};
use std::{sync::Arc, time::Duration};
use tentacle::service::TargetProtocol;
use tentacle::{
    builder::{MetaBuilder, ServiceBuilder},
    context::ServiceContext,
    secio::PeerId,
    service::{
        BlockingFlag, ProtocolHandle, Service, ServiceAsyncControl, ServiceError, ServiceEvent,
    },
    traits::ServiceHandle,
    utils::extract_peer_id,
    yamux::Config as YamuxConfig,
};
use tokio::time::Instant;
use tokio_util::codec::length_delimited;

#[derive(Clone)]
pub struct NetworkServiceHandle {
    gossip:     NetworkGossip,
    rpc:        NetworkRpc,
    peer_state: PeerManager,
}

#[async_trait]
impl Gossip for NetworkServiceHandle {
    async fn broadcast<M>(&self, cx: Context, end: &str, msg: M, p: Priority) -> ProtocolResult<()>
    where
        M: MessageCodec,
    {
        self.gossip.broadcast(cx, end, msg, p).await
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

pub struct NetworkService {
    // Config backup
    config: Arc<NetworkConfig>,

    // Public service components
    gossip: NetworkGossip,
    rpc:    NetworkRpc,

    // Core service
    peer_mgr_handle: PeerManager,
    net:             Service<ServiceHandler>,
}

impl NetworkService {
    pub fn new(config: NetworkConfig) -> Self {
        let config = Arc::new(config);
        let peer_manager = PeerManager::new(Arc::clone(&config));
        let service_handle = ServiceHandler {
            peer_store: peer_manager.clone(),
        };
        let message_router = MessageRouter::new();

        let block_flag = {
            let mut f = BlockingFlag::default();
            f.disable_all();
            f
        };

        let mut protocol_meta = Vec::new();
        let max_frame_length = config.max_frame_length;

        let ping_peer_manager = peer_manager.clone();
        let ping_handle =
            PingHandler::new(config.ping_interval, config.ping_timeout, ping_peer_manager);
        let ping = MetaBuilder::new()
            .flag(block_flag)
            .id(PING_PROTOCOL_ID.into())
            .name(move |_| "/axon/ping".to_string())
            .codec(move || {
                Box::new(
                    length_delimited::Builder::new()
                        .max_frame_length(max_frame_length)
                        .new_codec(),
                )
            })
            .service_handle(move || ProtocolHandle::Callback(Box::new(ping_handle)))
            .build();
        protocol_meta.push(ping);

        let identify_peer_manager = peer_manager.clone();
        let identify = MetaBuilder::new()
            .flag(block_flag)
            .id(IDENTIFY_PROTOCOL_ID.into())
            .name(move |_| "/axon/identify".to_string())
            .codec(move || {
                Box::new(
                    length_delimited::Builder::new()
                        .max_frame_length(max_frame_length)
                        .new_codec(),
                )
            })
            .service_handle(move || {
                ProtocolHandle::Callback(Box::new(IdentifyProtocol::new(identify_peer_manager)))
            })
            .build();
        protocol_meta.push(identify);

        let discovery_peer_manager = DiscoveryAddressManager::new(peer_manager.clone());
        let discovery = MetaBuilder::new()
            .flag(block_flag)
            .id(DISCOVERY_PROTOCOL_ID.into())
            .name(move |_| "/axon/discovery".to_string())
            .codec(move || {
                Box::new(
                    length_delimited::Builder::new()
                        .max_frame_length(max_frame_length)
                        .new_codec(),
                )
            })
            .service_handle(move || {
                ProtocolHandle::Callback(Box::new(DiscoveryProtocol::new(
                    discovery_peer_manager,
                    None,
                )))
            })
            .build();
        protocol_meta.push(discovery);

        let transmitter_peer_manager = peer_manager.clone();
        let transmitter_router = message_router.clone();
        let transmitter = MetaBuilder::new()
            .flag(block_flag)
            .id(TRANSMITTER_PROTOCOL_ID.into())
            .name(move |_| "/axon/transmitter".to_string())
            .before_send(compress)
            .before_receive(|| Some(Box::new(decompress)))
            .codec(move || {
                Box::new(
                    length_delimited::Builder::new()
                        .max_frame_length(max_frame_length)
                        .new_codec(),
                )
            })
            .service_handle(move || {
                ProtocolHandle::Callback(Box::new(TransmitterProtocol::new(
                    transmitter_router,
                    transmitter_peer_manager,
                )))
            })
            .build();
        protocol_meta.push(transmitter);

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
            .timeout(Duration::from_secs(5));

        let service = service_builder.build(service_handle);

        let control: ServiceAsyncControl = service.control().clone().into();

        let gossip = NetworkGossip::new(control.clone(), peer_manager.clone());
        let rpc = NetworkRpc::new(control, message_router);

        NetworkService {
            config,
            gossip,
            rpc,
            peer_mgr_handle: peer_manager,
            net: service,
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
            gossip:     self.gossip.clone(),
            rpc:        self.rpc.clone(),
            peer_state: self.peer_mgr_handle.clone(),
        }
    }

    pub fn peer_id(&self) -> PeerId {
        self.config.secio_keypair.peer_id()
    }

    pub fn set_chain_id(&self, chain_id: String) {
        self.peer_mgr_handle.set_chain_id(chain_id);
    }

    pub async fn run(mut self) {
        self.net
            .listen(self.config.default_listen.clone())
            .await
            .unwrap();
        for addr in self.config.bootstraps.iter() {
            self.net
                .dial(
                    addr.clone(),
                    TargetProtocol::Single(IDENTIFY_PROTOCOL_ID.into()),
                )
                .await
                .unwrap();
        }
        let mut control: ServiceAsyncControl = self.net.control().clone().into();

        let mut interval = tokio::time::interval_at(Instant::now(), Duration::from_secs(10));
        loop {
            tokio::select! {
                Some(_) = self.net.next() => {},
                _ = interval.tick() => {
                    for addr in self.peer_mgr_handle.unconnected_bootstraps() {
                        control
                            .dial(addr, TargetProtocol::Single(IDENTIFY_PROTOCOL_ID.into()))
                            .await
                            .unwrap();
                    }
                }
                else => {
                    let _ = control.shutdown().await;
                    break
                }
            }
        }
    }
}

struct ServiceHandler {
    peer_store: PeerManager,
}

impl ServiceHandle for ServiceHandler {
    fn handle_error(&mut self, _control: &mut ServiceContext, error: ServiceError) {
        log::info!("p2p error: {:?}", error)
    }

    fn handle_event(&mut self, _control: &mut ServiceContext, event: ServiceEvent) {
        match event {
            ServiceEvent::SessionOpen { session_context } => self.peer_store.register(
                PeerInfo::new(session_context.address.clone(), session_context.id),
            ),
            ServiceEvent::SessionClose { session_context } => self
                .peer_store
                .unregister(&extract_peer_id(&session_context.address).unwrap()),
            ServiceEvent::ListenClose { address } => {
                log::info!("listen stop at: {}", address)
            }
            ServiceEvent::ListenStarted { address } => {
                log::info!("listen start at: {}", address)
            }
        }
    }
}
