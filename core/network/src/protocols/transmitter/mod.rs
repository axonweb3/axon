use tentacle::{
    bytes::Bytes,
    context::{ProtocolContext, ProtocolContextMutRef},
    traits::ServiceProtocol,
    utils::extract_peer_id,
};

use self::protocol::ReceivedMessage;
use crate::{
    peer_manager::PeerManager,
    reactor::{MessageRouter, RemotePeer},
};

pub mod protocol;

pub struct TransmitterProtocol {
    router:       MessageRouter,
    peer_manager: PeerManager,
}

impl TransmitterProtocol {
    pub fn new(router: MessageRouter, peer_manager: PeerManager) -> Self {
        TransmitterProtocol {
            router,
            peer_manager,
        }
    }
}

impl ServiceProtocol for TransmitterProtocol {
    fn init(&mut self, _context: &mut ProtocolContext) {}

    fn connected(&mut self, context: ProtocolContextMutRef, _version: &str) {
        self.peer_manager.open_protocol(
            &extract_peer_id(&context.session.address).unwrap(),
            crate::protocols::TRANSMITTER_PROTOCOL_ID.into(),
        )
    }

    fn disconnected(&mut self, context: ProtocolContextMutRef) {
        self.peer_manager.close_protocol(
            &extract_peer_id(&context.session.address).unwrap(),
            &crate::protocols::TRANSMITTER_PROTOCOL_ID.into(),
        )
    }

    fn received(&mut self, context: ProtocolContextMutRef, data: Bytes) {
        let session = context.session;
        let recv_msg = ReceivedMessage {
            session_id: session.id,
            peer_id: session.remote_pubkey.as_ref().unwrap().peer_id(),
            data,
        };

        let remote_peer = RemotePeer::from_proto_context(&context);

        // let host = remote_peer.connected_addr.host.to_owned();
        let route_fut = self.router.route_message(remote_peer.clone(), recv_msg);
        tokio::spawn(async move {
            // common_apm::metrics::network::NETWORK_RECEIVED_MESSAGE_IN_PROCESSING_GUAGE.
            // inc(); common_apm::metrics::network::
            // NETWORK_RECEIVED_IP_MESSAGE_IN_PROCESSING_GUAGE_VEC
            //     .with_label_values(&[&host])
            //     .inc();

            if let Err(err) = route_fut.await {
                log::warn!("route message from {} failed: {}", remote_peer, err);
            }

            // common_apm::metrics::network::
            // NETWORK_RECEIVED_MESSAGE_IN_PROCESSING_GUAGE.dec();
            // common_apm::metrics::network::
            // NETWORK_RECEIVED_IP_MESSAGE_IN_PROCESSING_GUAGE_VEC
            //     .with_label_values(&[&host])
            //     .dec();
        });
    }
}
