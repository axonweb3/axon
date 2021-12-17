//! TODO: We should periodic forwarding of consensus node addresses
//! Mark and switch consensus node list

use std::net::IpAddr;
use tentacle::{multiaddr::Multiaddr, utils::multiaddr_to_socketaddr};

mod addr_manager;
mod ban_list;
mod peer_store_db;
mod peer_store_impl;
mod types;

/// peer store evict peers after reach this limitation
pub(crate) const ADDR_COUNT_LIMIT: usize = 16384;
/// Consider we never seen a peer if peer's last_connected_at beyond this
/// timeout
const ADDR_TIMEOUT_MS: u64 = 7 * 24 * 3600 * 1000;
/// The timeout that peer's address should be added to the feeler list again
pub(crate) const ADDR_TRY_TIMEOUT_MS: u64 = 3 * 24 * 3600 * 1000;
/// When obtaining the list of selectable nodes for identify,
/// the node that has just been disconnected needs to be excluded
pub(crate) const DIAL_INTERVAL: u64 = 15 * 1000;
const ADDR_MAX_RETRIES: u32 = 3;
const ADDR_MAX_FAILURES: u32 = 10;

pub use self::{peer_store_impl::PeerStore, types::AddrInfo};

/// Alias score
pub type Score = i32;

/// PeerStore Scoring configuration
#[derive(Copy, Clone, Debug)]
pub struct PeerScoreConfig {
    /// Default score
    pub default_score:  Score,
    /// Ban score
    pub ban_score:      Score,
    /// Ban time
    pub ban_timeout_ms: u64,
}

impl Default for PeerScoreConfig {
    fn default() -> Self {
        PeerScoreConfig {
            default_score:  100,
            ban_score:      40,
            ban_timeout_ms: 24 * 3600 * 1000, // 1 day
        }
    }
}

/// Peer Status
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Status {
    /// Connected
    Connected,
    /// The peer is disconnected
    Disconnected,
}

/// Report result
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ReportResult {
    /// Ok
    Ok,
    /// The peer is banned
    Banned,
}

impl ReportResult {
    /// Whether ban
    pub fn is_banned(self) -> bool {
        self == ReportResult::Banned
    }

    /// Whether ok
    pub fn is_ok(self) -> bool {
        self == ReportResult::Ok
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum Group {
    None,
    LocalNetwork,
    IP4([u8; 2]),
    IP6([u8; 4]),
}

impl From<&Multiaddr> for Group {
    fn from(multiaddr: &Multiaddr) -> Group {
        if let Some(socket_addr) = multiaddr_to_socketaddr(multiaddr) {
            let ip_addr = socket_addr.ip();
            if ip_addr.is_loopback() {
                return Group::LocalNetwork;
            }
            // TODO uncomment after ip feature stable
            // if !ip_addr.is_global() {
            //     // Global NetworkGroup
            //     return Group::GlobalNetwork
            // }

            // IPv4 NetworkGroup
            if let IpAddr::V4(ipv4) = ip_addr {
                let bits = ipv4.octets();
                return Group::IP4([bits[0], bits[1]]);
            }
            // IPv6 NetworkGroup
            if let IpAddr::V6(ipv6) = ip_addr {
                if let Some(ipv4) = ipv6.to_ipv4() {
                    let bits = ipv4.octets();
                    return Group::IP4([bits[0], bits[1]]);
                }
                let bits = ipv6.octets();
                return Group::IP6([bits[0], bits[1], bits[2], bits[3]]);
            }
        }
        // Can't group addr
        Group::None
    }
}
