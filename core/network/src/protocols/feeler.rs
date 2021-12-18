use crate::peer_manager::PeerManager;
use std::sync::Arc;
use tentacle::{
    context::{ProtocolContext, ProtocolContextMutRef},
    traits::ServiceProtocol,
};

/// Feeler
/// Currently do nothing, CKBProtocol auto refresh peer_store after connected.
pub struct Feeler {
    network_state: Arc<PeerManager>,
}

impl Feeler {
    pub fn new(network_state: Arc<PeerManager>) -> Self {
        Feeler { network_state }
    }
}

impl ServiceProtocol for Feeler {
    fn init(&mut self, _context: &mut ProtocolContext) {}

    fn connected(&mut self, context: ProtocolContextMutRef, _: &str) {
        let session = context.session;
        if context.session.ty.is_outbound() {
            self.network_state.with_peer_store_mut(|peer_store| {
                peer_store.add_outbound_addr(session.address.clone());
            });
        }

        log::debug!("peer={} FeelerProtocol.connected", session.address);
        if let Err(err) = context.control().disconnect(session.id) {
            log::debug!("Disconnect failed {:?}, error: {:?}", session.id, err);
        }
    }

    fn disconnected(&mut self, context: ProtocolContextMutRef) {
        let session = context.session;
        self.network_state.with_registry_mut(|reg| {
            reg.remove_feeler(&session.address);
        });
        log::debug!("peer={} FeelerProtocol.disconnected", session.address);
    }
}
