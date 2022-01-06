use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tentacle::{
    context::SessionContext, multiaddr::Multiaddr, secio::PeerId, service::SessionType, ProtocolId,
    SessionId,
};

pub struct PeerInfo {
    pub addr:         Multiaddr,
    pub session_id:   SessionId,
    opened_protocols: HashSet<ProtocolId>,
    pub session_type: SessionType,
    pub listens:      Vec<Multiaddr>,
    pub reuse:        bool,
}

impl PeerInfo {
    pub fn new(ctx: Arc<SessionContext>) -> Self {
        PeerInfo {
            addr:             ctx.address.clone(),
            session_id:       ctx.id,
            opened_protocols: Default::default(),
            session_type:     ctx.ty,
            listens:          Vec::new(),
            reuse:            false,
        }
    }

    pub fn insert_protocol(&mut self, id: ProtocolId) {
        self.opened_protocols.insert(id);
    }

    pub fn remove_protocol(&mut self, id: &ProtocolId) {
        self.opened_protocols.remove(id);
    }
}

#[derive(Default)]
pub struct Online {
    pub peers:   HashMap<PeerId, PeerInfo>,
    pub dialing: HashSet<Multiaddr>,
    feeler:      HashSet<Multiaddr>,
}

impl Online {
    pub fn remove_feeler(&mut self, addr: &Multiaddr) {
        self.feeler.remove(addr);
    }

    pub fn add_feeler(&mut self, addr: Multiaddr) -> bool {
        self.feeler.insert(addr)
    }

    pub fn is_feeler(&self, addr: &Multiaddr) -> bool {
        self.feeler.contains(addr)
    }
}

pub struct ConnectionStatus {
    /// Total session number
    pub total:    usize,
    /// inbound number
    pub inbound:  usize,
    /// outbound number
    pub outbound: usize,
}

impl Online {
    pub fn connection_status(&self) -> ConnectionStatus {
        let total = self.peers.len();
        let mut inbound = 0;
        let mut outbound = 0;
        for peer in self.peers.values() {
            if peer.session_type.is_outbound() {
                outbound += 1;
            } else {
                inbound += 1;
            }
        }
        ConnectionStatus {
            total,
            inbound,
            outbound,
        }
    }
}
