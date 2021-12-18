use crate::{
    error::ErrorKind,
    peer_manager::peer_store::{
        addr_manager::Manager,
        ban_list::BanList,
        types::{ip_to_network, AddrInfo, BannedAddr, PeerInfo},
        Group, Multiaddr, PeerScoreConfig, ReportResult, Status, ADDR_COUNT_LIMIT, ADDR_TIMEOUT_MS,
        ADDR_TRY_TIMEOUT_MS, DIAL_INTERVAL,
    },
};
use ipnetwork::IpNetwork;
use rand::prelude::IteratorRandom;
use std::collections::{hash_map::Entry, HashMap, HashSet};
use tentacle::{
    secio::PeerId,
    service::SessionType,
    utils::{extract_peer_id, multiaddr_to_socketaddr},
};

/// Peer store
///
/// | -- choose to identify --| --- choose to feeler --- | --      delete     --
/// | | 1      | 2     | 3      | 4    | 5    | 6   | 7    | More than seven
/// days  |
#[derive(Default)]
pub struct PeerStore {
    addr_manager: Manager,
    ban_list:     BanList,
    peers:        HashMap<PeerId, PeerInfo>,
    score_config: PeerScoreConfig,
}

impl PeerStore {
    /// New with address list and ban list
    pub fn new(addr_manager: Manager, ban_list: BanList) -> Self {
        PeerStore {
            addr_manager,
            ban_list,
            peers: Default::default(),
            score_config: Default::default(),
        }
    }

    /// this method will assume peer is connected, which implies address is
    /// "verified".
    pub fn add_connected_peer(&mut self, addr: Multiaddr, session_type: SessionType) {
        let now_ms = faketime::unix_time_as_millis();
        match self
            .peers
            .entry(extract_peer_id(&addr).expect("connected addr should have peer id"))
        {
            Entry::Occupied(mut entry) => {
                let mut peer = entry.get_mut();
                peer.connected_addr = addr;
                peer.last_connected_at_ms = now_ms;
                peer.session_type = session_type;
            }
            Entry::Vacant(entry) => {
                let peer = PeerInfo::new(addr, session_type, now_ms);
                entry.insert(peer);
            }
        }
    }

    /// Add discovered peer address
    /// this method will assume peer and addr is untrust since we have not
    /// connected to it.
    pub fn add_addr(&mut self, addr: Multiaddr) -> Result<(), ErrorKind> {
        if self.ban_list.is_addr_banned(&addr) {
            return Ok(());
        }
        self.check_purge()?;
        let score = self.score_config.default_score;
        self.addr_manager.add(AddrInfo::new(addr, 0, score));
        Ok(())
    }

    /// Add outbound peer address
    pub fn add_outbound_addr(&mut self, addr: Multiaddr) {
        if self.ban_list.is_addr_banned(&addr) {
            return;
        }
        let score = self.score_config.default_score;
        self.addr_manager
            .add(AddrInfo::new(addr, faketime::unix_time_as_millis(), score));
    }

    /// Get address manager
    pub fn addr_manager(&self) -> &Manager {
        &self.addr_manager
    }

    /// Get mut address manager
    pub fn mut_addr_manager(&mut self) -> &mut Manager {
        &mut self.addr_manager
    }

    /// Report peer behaviours
    pub fn report(&mut self, addr: &Multiaddr) -> ReportResult {
        if let Some(peer_addr) = self.addr_manager.get_mut(addr) {
            let score = peer_addr.score.saturating_add(0);
            peer_addr.score = score;
            if score < self.score_config.ban_score {
                self.ban_addr(
                    addr,
                    self.score_config.ban_timeout_ms,
                    "report behaviour".to_string(),
                );
                return ReportResult::Banned;
            }
        }
        ReportResult::Ok
    }

    /// Remove peer id
    pub fn remove_disconnected_peer(&mut self, addr: &Multiaddr) -> Option<PeerInfo> {
        extract_peer_id(addr).and_then(|peer_id| self.peers.remove(&peer_id))
    }

    /// Get peer status
    pub fn peer_status(&self, peer_id: &PeerId) -> Status {
        if self.peers.contains_key(peer_id) {
            Status::Connected
        } else {
            Status::Disconnected
        }
    }

    /// Get peers for outbound connection, this method randomly return recently
    /// connected peer addrs
    pub fn fetch_addrs_to_attempt(&mut self, count: usize) -> Vec<AddrInfo> {
        // Get info:
        // 1. Not already connected
        // 2. Connected within 3 days

        let now_ms = faketime::unix_time_as_millis();
        let peers = &self.peers;
        let addr_expired_ms = now_ms.saturating_sub(ADDR_TRY_TIMEOUT_MS);
        // get addrs that can attempt.
        self.addr_manager
            .fetch_random(count, |peer_addr: &AddrInfo| {
                extract_peer_id(&peer_addr.addr)
                    .map(|peer_id| !peers.contains_key(&peer_id))
                    .unwrap_or_default()
                    && peer_addr.connected(|t| {
                        t > addr_expired_ms && t <= now_ms.saturating_sub(DIAL_INTERVAL)
                    })
            })
    }

    /// Get peers for feeler connection, this method randomly return peer addrs
    /// that we never connected to.
    pub fn fetch_addrs_to_feeler(&mut self, count: usize) -> Vec<AddrInfo> {
        // Get info:
        // 1. Not already connected
        // 2. Not already tried in a minute
        // 3. Not connected within 3 days

        let now_ms = faketime::unix_time_as_millis();
        let addr_expired_ms = now_ms.saturating_sub(ADDR_TRY_TIMEOUT_MS);
        let peers = &self.peers;
        self.addr_manager
            .fetch_random(count, |peer_addr: &AddrInfo| {
                extract_peer_id(&peer_addr.addr)
                    .map(|peer_id| !peers.contains_key(&peer_id))
                    .unwrap_or_default()
                    && !peer_addr.tried_in_last_minute(now_ms)
                    && !peer_addr.connected(|t| t > addr_expired_ms)
            })
    }

    /// Return valid addrs that success connected, used for discovery.
    pub fn fetch_random_addrs(&mut self, count: usize) -> Vec<AddrInfo> {
        // Get info:
        // 1. Already connected or Connected within 7 days

        let now_ms = faketime::unix_time_as_millis();
        let addr_expired_ms = now_ms.saturating_sub(ADDR_TIMEOUT_MS);
        let peers = &self.peers;
        // get success connected addrs.
        self.addr_manager
            .fetch_random(count, |peer_addr: &AddrInfo| {
                extract_peer_id(&peer_addr.addr)
                    .map(|peer_id| peers.contains_key(&peer_id))
                    .unwrap_or_default()
                    || peer_addr.connected(|t| t > addr_expired_ms)
            })
    }

    #[allow(clippy::mutable_key_type)]
    pub fn fetch_addr_by_peer_id(&self, peer_id: &PeerId) -> HashSet<Multiaddr> {
        let now_ms = faketime::unix_time_as_millis();
        let addr_expired_ms = now_ms.saturating_sub(ADDR_TRY_TIMEOUT_MS);
        self.addr_manager.get_by_peer_id(peer_id, |peer_addr| {
            peer_addr
                .connected(|t| t > addr_expired_ms && t <= now_ms.saturating_sub(DIAL_INTERVAL))
        })
    }

    /// Ban an addr
    pub(crate) fn ban_addr(&mut self, addr: &Multiaddr, timeout_ms: u64, ban_reason: String) {
        if let Some(addr) = multiaddr_to_socketaddr(addr) {
            let network = ip_to_network(addr.ip());
            self.ban_network(network, timeout_ms, ban_reason)
        }
        self.addr_manager.remove(addr);
    }

    pub(crate) fn ban_network(&mut self, network: IpNetwork, timeout_ms: u64, ban_reason: String) {
        let now_ms = faketime::unix_time_as_millis();
        let ban_addr = BannedAddr {
            address: network,
            ban_until: now_ms + timeout_ms,
            created_at: now_ms,
            ban_reason,
        };
        self.mut_ban_list().ban(ban_addr);
    }

    /// Whether the address is banned
    pub fn is_addr_banned(&self, addr: &Multiaddr) -> bool {
        self.ban_list().is_addr_banned(addr)
    }

    /// Get ban list
    pub fn ban_list(&self) -> &BanList {
        &self.ban_list
    }

    /// Get mut ban list
    pub fn mut_ban_list(&mut self) -> &mut BanList {
        &mut self.ban_list
    }

    /// Clear ban list
    pub fn clear_ban_list(&mut self) {
        std::mem::take(&mut self.ban_list);
    }

    /// Check and try delete addrs if reach limit
    /// return Err if peer_store is full and can't be purge
    fn check_purge(&mut self) -> Result<(), ErrorKind> {
        if self.addr_manager.count() < ADDR_COUNT_LIMIT {
            return Ok(());
        }

        // Evicting invalid data in the peer store is a relatively rare operation
        // There are certain cleanup strategies here:
        // 1. First evict the nodes that have reached the eviction condition
        // 2. If the first step is unsuccessful, enter the network segment grouping mode
        //  2.1. Group current data according to network segment
        //  2.2. Sort according to the amount of data in the same network segment
        //  2.3. In the network segment with more than 4 peer, randomly evict 2 peer

        let now_ms = faketime::unix_time_as_millis();
        let candidate_peers: Vec<_> = self
            .addr_manager
            .addrs_iter()
            .filter_map(|addr| {
                if !addr.is_connectable(now_ms) {
                    Some(addr.addr.clone())
                } else {
                    None
                }
            })
            .collect();

        for key in candidate_peers.iter() {
            self.addr_manager.remove(key);
        }

        if candidate_peers.is_empty() {
            let candidate_peers: Vec<_> = {
                let mut peers_by_network_group: HashMap<Group, Vec<_>> = HashMap::default();
                for addr in self.addr_manager.addrs_iter() {
                    peers_by_network_group
                        .entry((&addr.addr).into())
                        .or_default()
                        .push(addr);
                }
                let len = peers_by_network_group.len();
                let mut peers = peers_by_network_group
                    .drain()
                    .map(|(_, v)| v)
                    .collect::<Vec<Vec<_>>>();

                peers.sort_unstable_by_key(|k| std::cmp::Reverse(k.len()));

                peers
                    .into_iter()
                    .take(len / 2)
                    .flat_map(move |addrs| {
                        if addrs.len() > 4 {
                            Some(
                                addrs
                                    .iter()
                                    .choose_multiple(&mut rand::thread_rng(), 2)
                                    .into_iter()
                                    .map(|addr| addr.addr.clone())
                                    .collect::<Vec<Multiaddr>>(),
                            )
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect()
            };

            for key in candidate_peers.iter() {
                self.addr_manager.remove(key);
            }

            if candidate_peers.is_empty() {
                return Err(ErrorKind::PeerStoreEvictionFailed);
            }
        }
        Ok(())
    }
}
