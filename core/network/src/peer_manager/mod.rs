use parking_lot::{Mutex, RwLock};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tentacle::{
    multiaddr::Multiaddr, secio::PeerId, utils::extract_peer_id, ProtocolId, SessionId,
};

pub use self::{
    peer_store::{AddrInfo, PeerStore},
    registry::{Online, PeerInfo},
};
use crate::config::NetworkConfig;

mod peer_store;
mod registry;

pub struct PeerManager {
    online:           RwLock<Online>,
    peer_store:       RwLock<PeerStore>,
    bootstraps:       HashMap<PeerId, Multiaddr>,
    chain_id:         Mutex<String>,
    pub public_addrs: RwLock<HashSet<Multiaddr>>,
    config:           Arc<NetworkConfig>,
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
            b
        };
        PeerManager {
            online: RwLock::new(Online::default()),
            peer_store: RwLock::new(PeerStore::load_from_dir_or_default(
                config.peer_store_path.clone(),
            )),
            chain_id: Mutex::new(String::new()),
            bootstraps,
            public_addrs: RwLock::new(HashSet::new()),
            config,
        }
    }

    #[allow(clippy::mutable_key_type)]
    pub fn peers(&self, pid: Vec<PeerId>) -> (Vec<SessionId>, Vec<Multiaddr>) {
        let mut connected = Vec::new();
        let mut unconnected = Vec::new();
        let online = self.online.read();
        let mut peer_store = self.peer_store.write();

        for id in pid {
            if let Some(info) = online.peers.get(&id) {
                connected.push(info.session_id);
                continue;
            }
            let list = peer_store.fetch_addr_by_peer_id(&id);

            if !list.is_empty() {
                let now_ms = faketime::unix_time_as_millis();
                for addr in list.iter() {
                    if let Some(paddr) = peer_store.mut_addr_manager().get_mut(addr) {
                        paddr.mark_tried(now_ms);
                    }
                }
                unconnected.extend(list.into_iter())
            }
        }

        (connected, unconnected)
    }

    pub fn register(&self, peer: PeerInfo) {
        let (addr, ty) = (peer.addr.clone(), peer.session_type);
        self.with_registry_mut(|online| {
            online
                .peers
                .insert(extract_peer_id(&peer.addr).unwrap(), peer)
        });
        self.with_peer_store_mut(|peer_store| peer_store.add_connected_peer(addr, ty));
    }

    pub fn unregister(&self, id: &PeerId) {
        if let Some(peer) = self.with_registry_mut(|online| online.peers.remove(id)) {
            self.with_peer_store_mut(|peer_store| peer_store.remove_disconnected_peer(&peer.addr));
        }
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

    pub fn local_peer_id(&self) -> PeerId {
        self.config.secio_keypair.peer_id()
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

    pub fn with_registry<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Online) -> T,
    {
        f(&self.online.read())
    }

    pub fn with_registry_mut<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut Online) -> T,
    {
        f(&mut self.online.write())
    }

    pub fn with_peer_store<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&PeerStore) -> T,
    {
        f(&self.peer_store.read())
    }

    pub fn with_peer_store_mut<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut PeerStore) -> T,
    {
        f(&mut self.peer_store.write())
    }

    pub fn always_allow(&self, peer_id: &PeerId) -> bool {
        self.bootstraps.contains_key(peer_id)
    }

    pub fn local_listen_addrs(&self) -> Vec<Multiaddr> {
        self.public_addrs.read().iter().cloned().collect()
    }

    pub(crate) fn public_addrs(&self, count: usize) -> Vec<Multiaddr> {
        self.public_addrs
            .read()
            .iter()
            .take(count)
            .cloned()
            .collect()
    }
}
