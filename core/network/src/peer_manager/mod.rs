use parking_lot::{Mutex, RwLock};
use std::borrow::Borrow;
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
use crate::PeerIdExt;

mod peer_store;
mod registry;

pub struct PeerManager {
    online:           RwLock<Online>,
    peer_store:       RwLock<PeerStore>,
    bootstraps:       HashMap<PeerId, Multiaddr>,
    chain_id:         Mutex<String>,
    pub public_addrs: RwLock<HashSet<Multiaddr>>,
    config:           Arc<NetworkConfig>,

    pub consensus_list: RwLock<HashSet<PeerId>>,
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
            consensus_list: RwLock::new(HashSet::new()),
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

    #[allow(clippy::mutable_key_type)]
    pub fn unconnected_consensus_peer(&self) -> Vec<Multiaddr> {
        let mut unconnected = Vec::new();
        let online = self.online.read();
        let mut peer_store = self.peer_store.write();

        for id in self.consensus_list.read().iter() {
            if online.peers.contains_key(id) {
                continue;
            }
            let list = peer_store.fetch_addr_by_peer_id(id);
            let now_ms = faketime::unix_time_as_millis();
            for addr in list.iter() {
                if let Some(paddr) = peer_store.mut_addr_manager().get_mut(addr) {
                    paddr.mark_tried(now_ms);
                }
            }
            unconnected.extend(list.into_iter())
        }
        unconnected
    }

    pub fn connected_consensus_peer(&self) -> Vec<Multiaddr> {
        let online = self.online.read();
        let mut list = Vec::new();

        for id in self.consensus_list.read().iter() {
            if let Some(info) = online.peers.get(id) {
                if info.session_type.is_outbound() || info.reuse {
                    list.push(info.addr.clone());
                }

                list.extend(info.listens.iter().cloned());
            }
        }

        list
    }

    pub fn register(&self, peer: PeerInfo) {
        let (addr, ty) = (peer.addr.clone(), peer.session_type);
        let addr_for_apm_use = peer.addr.clone();
        self.with_registry_mut(|online| {
            online
                .peers
                .insert(extract_peer_id(&peer.addr).unwrap(), peer)
        });
        self.with_peer_store_mut(|peer_store| peer_store.add_connected_peer(addr, ty));

        if self
            .consensus_list
            .read()
            .contains(&extract_peer_id(&addr_for_apm_use).unwrap())
        {
            common_apm::metrics::network::NETWORK_CONNECTED_CONSENSUS_PEERS.inc()
        }

        common_apm::metrics::network::NETWORK_CONNECTED_PEERS.inc();
        common_apm::metrics::network::NETWORK_SAVED_PEER_COUNT
            .set(self.with_peer_store_mut(|peer_store| peer_store.peer_count()) as i64);
    }

    pub fn unregister(&self, addr: &Multiaddr) {
        if let Some(peer) = self.with_registry_mut(|online| {
            online.remove_feeler(addr);
            online.peers.remove(&extract_peer_id(addr).unwrap())
        }) {
            self.with_peer_store_mut(|peer_store| peer_store.remove_disconnected_peer(&peer.addr));

            let peer_id = extract_peer_id(&peer.addr).unwrap();
            common_apm::metrics::network::NETWORK_PEER_ID_DISCONNECTED_COUNT_VEC
                .with_label_values(&[&peer_id.to_string()])
                .inc();

            if self.consensus_list.read().contains(&peer_id) {
                common_apm::metrics::network::NETWORK_CONNECTED_CONSENSUS_PEERS.dec()
            }

            common_apm::metrics::network::NETWORK_CONNECTED_PEERS.dec();
            common_apm::metrics::network::NETWORK_SAVED_PEER_COUNT
                .set(self.with_peer_store_mut(|peer_store| peer_store.peer_count()) as i64);
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

    pub fn always_allow(&self, addr: &Multiaddr) -> bool {
        let peer_id = extract_peer_id(addr).unwrap();
        self.bootstraps.contains_key(&peer_id)
            || self.consensus_list.read().contains(&peer_id)
            || self.with_peer_store(|peer_store| !peer_store.is_addr_banned(addr))
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

    pub fn ban_id(&self, peer_id: &PeerId, timeout: u64, ban_reason: String) -> Option<SessionId> {
        if let Some(info) = self.online.read().peers.get(peer_id) {
            self.peer_store
                .write()
                .ban_addr(&info.addr, timeout, ban_reason)
        }
        None
    }

    pub fn ban_session_id(&self, session_id: SessionId, timeout: u64, ban_reason: String) {
        let addr = self.online.read().peers.values().find_map(|info| {
            if info.session_id == session_id {
                Some(info.addr.clone())
            } else {
                None
            }
        });

        if let Some(addr) = addr {
            self.peer_store.write().ban_addr(&addr, timeout, ban_reason)
        }
    }
}
