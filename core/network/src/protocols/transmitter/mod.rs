use tentacle::{
    async_trait,
    bytes::Bytes,
    context::{ProtocolContext, ProtocolContextMutRef},
    runtime::spawn,
    traits::ServiceProtocol,
    utils::extract_peer_id,
};

use self::protocol::ReceivedMessage;
use crate::{
    peer_manager::PeerManager,
    reactor::{MessageRouter, RemotePeer},
    PeerIdExt,
};
use std::sync::Arc;

pub mod protocol;

pub struct TransmitterProtocol {
    router:       MessageRouter,
    peer_manager: Arc<PeerManager>,
}

impl TransmitterProtocol {
    pub fn new(router: MessageRouter, peer_manager: Arc<PeerManager>) -> Self {
        TransmitterProtocol {
            router,
            peer_manager,
        }
    }
}

#[async_trait]
impl ServiceProtocol for TransmitterProtocol {
    async fn init(&mut self, _context: &mut ProtocolContext) {}

    async fn connected(&mut self, context: ProtocolContextMutRef<'_>, _version: &str) {
        log::info!(
            "{} open on {}, addr: {}",
            context.proto_id,
            context.session.id,
            context.session.address
        );
        self.peer_manager.open_protocol(
            &extract_peer_id(&context.session.address).unwrap(),
            crate::protocols::SupportProtocols::Transmitter.protocol_id(),
        )
    }

    async fn disconnected(&mut self, context: ProtocolContextMutRef<'_>) {
        log::info!("{} close on {}", context.proto_id, context.session.id);
        self.peer_manager.close_protocol(
            &extract_peer_id(&context.session.address).unwrap(),
            &crate::protocols::SupportProtocols::Transmitter.protocol_id(),
        )
    }

    async fn received(&mut self, context: ProtocolContextMutRef<'_>, data: Bytes) {
        let session = context.session;
        let recv_msg = ReceivedMessage {
            session_id: session.id,
            peer_id: session.remote_pubkey.as_ref().unwrap().peer_id(),
            data,
        };

        let remote_peer = RemotePeer::from_proto_context(&context);
        let peer_id = remote_peer.peer_id.to_string();

        // let host = remote_peer.connected_addr.host.to_owned();
        let route_fut = self.router.route_message(remote_peer.clone(), recv_msg);
        spawn(async move {
            common_apm::metrics::network::NETWORK_RECEIVED_MESSAGE_IN_PROCESSING_GUAGE.inc();
            common_apm::metrics::network::NETWORK_RECEIVED_PEER_ID_MESSAGE_IN_PROCESSING_GUAGE_VEC
                .with_label_values(&[&peer_id])
                .inc();

            if let Err(err) = route_fut.await {
                log::warn!("route message from {:?} failed: {:?}", remote_peer, err);
            }

            common_apm::metrics::network::NETWORK_RECEIVED_MESSAGE_IN_PROCESSING_GUAGE.dec();
            common_apm::metrics::network::NETWORK_RECEIVED_PEER_ID_MESSAGE_IN_PROCESSING_GUAGE_VEC
                .with_label_values(&[&peer_id])
                .dec();
        });
    }
}
