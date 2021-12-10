use bytes::Bytes;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use derive_more::Display;
use parking_lot::RwLock;
use protocol::traits::{MessageCodec, MessageHandler};
use protocol::ProtocolResult;
use tentacle::context::ProtocolContextMutRef;
use tentacle::secio::PeerId;
use tentacle::SessionId;

use crate::common::ConnectedAddr;
use crate::endpoint::Endpoint;
use crate::error::{ErrorKind, NetworkError};
use crate::message::NetworkMessage;
use crate::protocols::ReceivedMessage;

use super::rpc_map::RpcMap;
use super::Reactor;

#[derive(Debug, Display)]
#[display(fmt = "connection isnt encrypted, no peer id")]
pub struct NoEncryption {}

#[derive(Debug, Display, Clone)]
#[display(fmt = "remote peer {:?} addr {}", peer_id, connected_addr)]
pub struct RemotePeer {
    pub session_id:     SessionId,
    pub peer_id:        PeerId,
    pub connected_addr: ConnectedAddr,
}

impl RemotePeer {
    pub fn from_proto_context(protocol_context: &ProtocolContextMutRef) -> Self {
        let session = protocol_context.session;
        let pubkey = session.remote_pubkey.as_ref().unwrap();

        RemotePeer {
            session_id:     session.id,
            peer_id:        pubkey.peer_id(),
            connected_addr: ConnectedAddr::from(&session.address),
        }
    }
}

pub struct RouterContext {
    pub(crate) remote_peer: RemotePeer,
    pub(crate) rpc_map:     Arc<RpcMap>,
}

impl RouterContext {
    fn new(remote_peer: RemotePeer, rpc_map: Arc<RpcMap>) -> Self {
        RouterContext {
            remote_peer,
            rpc_map,
        }
    }
}

type ReactorMap = HashMap<Endpoint, Arc<Box<dyn Reactor>>>;

#[derive(Clone)]
pub struct MessageRouter {
    // Endpoint to reactor channel map
    reactor_map: Arc<RwLock<ReactorMap>>,

    // Rpc map
    pub(crate) rpc_map: Arc<RpcMap>,
}

impl MessageRouter {
    pub fn new() -> Self {
        MessageRouter {
            reactor_map: Default::default(),
            rpc_map:     Arc::new(RpcMap::new()),
        }
    }

    pub fn register_reactor<M: MessageCodec>(
        &self,
        endpoint: Endpoint,
        message_handler: impl MessageHandler<Message = M>,
    ) {
        let reactor = super::generate(message_handler);
        self.reactor_map
            .write()
            .insert(endpoint, Arc::new(Box::new(reactor)));
    }

    pub fn register_rpc_response(&self, endpoint: Endpoint) {
        let nop_reactor = super::rpc_resp::<Bytes>();
        self.reactor_map
            .write()
            .insert(endpoint, Arc::new(Box::new(nop_reactor)));
    }

    pub fn route_message(
        &self,
        remote_peer: RemotePeer,
        recv_msg: ReceivedMessage,
    ) -> impl Future<Output = ProtocolResult<()>> {
        let reactor_map = Arc::clone(&self.reactor_map);
        let router_context = RouterContext::new(remote_peer, Arc::clone(&self.rpc_map));
        // let raw_data_size = recv_msg.data.len();

        async move {
            let network_message = { NetworkMessage::decode(recv_msg.data)? };
            // common_apm::metrics::network::on_network_message_received(&network_message.
            // url);

            let endpoint = network_message.url.parse::<Endpoint>()?;
            // common_apm::metrics::network::NETWORK_MESSAGE_SIZE_COUNT_VEC
            //     .with_label_values(&["received", &endpoint.root()])
            //     .inc_by(raw_data_size as i64);

            let reactor = {
                let opt_reactor = reactor_map.read().get(&endpoint).cloned();
                opt_reactor
                    .ok_or_else(|| NetworkError::from(ErrorKind::NoReactor(endpoint.root())))?
            };

            let ret = reactor
                .react(router_context, endpoint.clone(), network_message)
                .await;
            if let Err(err) = ret.as_ref() {
                log::error!("process {} message failed: {}", endpoint, err);
            }
            ret
        }
    }
}
