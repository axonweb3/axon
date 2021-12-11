use parking_lot::{Mutex, RwLock};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tentacle::{
    multiaddr::Multiaddr, secio::PeerId, utils::extract_peer_id, ProtocolId, SessionId,
};

use crate::config::NetworkConfig;

pub struct PeerInfo {
    addr:             Multiaddr,
    session_id:       SessionId,
    opened_protocols: HashSet<ProtocolId>,
}

impl PeerInfo {
    pub fn new(addr: Multiaddr, id: SessionId) -> Self {
        PeerInfo {
            addr,
            session_id: id,
            opened_protocols: Default::default(),
        }
    }

    fn insert_protocol(&mut self, id: ProtocolId) {
        self.opened_protocols.insert(id);
    }

    fn remove_protocol(&mut self, id: &ProtocolId) {
        self.opened_protocols.remove(id);
    }
}

#[derive(Default)]
struct Online {
    peers: HashMap<PeerId, PeerInfo>,
}

#[derive(Default)]
struct StorePeer {
    addrs: HashSet<Multiaddr>,
}

#[derive(Default)]
struct PeerStore {
    list: HashMap<PeerId, StorePeer>,
}

#[derive(Clone)]
pub struct PeerManager {
    online:     Arc<RwLock<Online>>,
    peer_store: Arc<RwLock<PeerStore>>,
    bootstraps: Arc<HashMap<PeerId, Multiaddr>>,
    chain_id:   Arc<Mutex<String>>,
}

impl PeerManager {
    pub fn new(config: Arc<NetworkConfig>) -> Self {
        let bootstraps = {
            let mut b = HashMap::with_capacity(config.bootstraps.len().saturating_sub(1));
            let id = config.secio_keypair.peer_id();
            for addr in config.bootstraps.iter() {
                let o_id = extract_peer_id(addr).unwrap();
                if id != o_id {
                    b.insert(o_id, addr.clone());
                }
            }
            Arc::new(b)
        };
        PeerManager {
            online: Arc::new(RwLock::new(Online::default())),
            peer_store: Arc::new(RwLock::new(PeerStore::default())),
            chain_id: Arc::new(Mutex::new(String::new())),
            bootstraps,
        }
    }

    pub fn peers(&self, pid: Vec<PeerId>) -> (Vec<SessionId>, Vec<Multiaddr>) {
        let mut connected = Vec::new();
        let mut unconnected = Vec::new();
        let online = self.online.read();
        let peer_store = self.peer_store.read();

        for id in pid {
            if let Some(info) = online.peers.get(&id) {
                connected.push(info.session_id);
                continue;
            }
            if let Some(info) = peer_store.list.get(&id) {
                if !info.addrs.is_empty() {
                    unconnected.extend(info.addrs.clone().into_iter())
                }
            }
        }

        (connected, unconnected)
    }

    pub fn register(&self, peer: PeerInfo) {
        let mut online = self.online.write();
        online
            .peers
            .insert(extract_peer_id(&peer.addr).unwrap(), peer);
    }

    pub fn unregister(&self, id: &PeerId) {
        let mut online = self.online.write();
        online.peers.remove(id);
    }

    pub fn open_protocol(&self, id: &PeerId, pid: ProtocolId) {
        let mut online = self.online.write();
        if let Some(info) = online.peers.get_mut(id) {
            info.insert_protocol(pid)
        }
    }

    pub fn close_protocol(&self, id: &PeerId, pid: &ProtocolId) {
        let mut online = self.online.write();
        if let Some(info) = online.peers.get_mut(id) {
            info.remove_protocol(pid)
        }
    }

    pub fn set_chain_id(&self, chain_id: String) {
        *self.chain_id.lock() = chain_id;
    }

    pub fn chain_id(&self) -> String {
        self.chain_id.lock().clone()
    }

    pub fn unconnected_bootstraps(&self) -> Vec<Multiaddr> {
        let online = self.online.read();
        let mut res = Vec::new();

        for (id, addr) in self.bootstraps.iter() {
            if !online.peers.contains_key(id) {
                res.push(addr.clone())
            }
        }
        res
    }
}
