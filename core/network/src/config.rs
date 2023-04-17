use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    time::Duration,
};

use tentacle::{
    multiaddr::{Multiaddr, Protocol},
    secio::{PeerId, SecioKeyPair},
};

use common_config_parser::types::Config;
use protocol::{codec::hex_decode, ProtocolResult};

use crate::error::NetworkError;

// TODO: 0.0.0.0 expose? 127.0.0.1 doesn't work because of tentacle-discovery.
// Default listen address: 0.0.0.0:2337
pub const DEFAULT_LISTEN_IP_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
pub const DEFAULT_LISTEN_PORT: u16 = 2337;
// Default max connections
pub const DEFAULT_MAX_CONNECTIONS: usize = 40;
// Default connection stream frame window lenght
pub const DEFAULT_MAX_FRAME_LENGTH: usize = 4 * 1024 * 1024; // 4 Mib
pub const DEFAULT_BUFFER_SIZE: usize = 24 * 1024 * 1024; // same as tentacle

// Default max wait streams for accept
pub const DEFAULT_MAX_WAIT_STREAMS: usize = 256;
// Default write timeout
pub const DEFAULT_WRITE_TIMEOUT: u64 = 10; // seconds

pub const DEFAULT_SAME_IP_CONN_LIMIT: usize = 1;
pub const DEFAULT_INBOUND_CONN_LIMIT: usize = 20;

// Default peer trust metric
pub const DEFAULT_PEER_TRUST_INTERVAL_DURATION: Duration = Duration::from_secs(60);
pub const DEFAULT_PEER_TRUST_MAX_HISTORY_DURATION: Duration =
    Duration::from_secs(24 * 60 * 60 * 10); // 10 day
const DEFAULT_PEER_FATAL_BAN_DURATION: Duration = Duration::from_secs(60 * 60); // 1 hour
const DEFAULT_PEER_SOFT_BAN_DURATION: Duration = Duration::from_secs(60 * 10); // 10 minutes

// Default peer store persistent path
pub const DEFAULT_PEER_DAT_FILE: &str = "./";

pub const DEFAULT_PING_INTERVAL: u64 = 15;
pub const DEFAULT_PING_TIMEOUT: u64 = 30;
pub const DEFAULT_DISCOVERY_SYNC_INTERVAL: u64 = 60 * 60; // 1 hour

pub const DEFAULT_PEER_MANAGER_HEART_BEAT_INTERVAL: u64 = 30;
pub const DEFAULT_SELF_HEART_BEAT_INTERVAL: u64 = 35;

pub const DEFAULT_RPC_TIMEOUT: u64 = 10;

#[derive(Debug)]
pub struct NetworkConfig {
    // connection
    pub default_listen:   Multiaddr,
    pub max_connections:  usize,
    pub max_frame_length: usize,
    pub send_buffer_size: usize,
    pub recv_buffer_size: usize,
    pub max_wait_streams: usize,
    pub write_timeout:    u64,

    // peer manager
    pub bootstraps:             Vec<Multiaddr>,
    pub allowlist:              Vec<PeerId>,
    pub allowlist_only:         bool,
    pub enable_save_restore:    bool,
    pub peer_store_path:        PathBuf,
    pub peer_trust_interval:    Duration,
    pub peer_trust_max_history: Duration,
    pub peer_fatal_ban:         Duration,
    pub peer_soft_ban:          Duration,
    pub same_ip_conn_limit:     usize,
    pub inbound_conn_limit:     usize,

    // identity and encryption
    pub secio_keypair: SecioKeyPair,

    // protocol
    pub ping_interval:           Duration,
    pub ping_timeout:            Duration,
    pub discovery_sync_interval: Duration,

    // routine
    pub peer_manager_heart_beat_interval: Duration,
    pub heart_beat_interval:              Duration,

    // rpc
    pub rpc_timeout: Duration,
}

impl NetworkConfig {
    pub fn new() -> Self {
        let mut listen_addr = Multiaddr::from(DEFAULT_LISTEN_IP_ADDR);
        listen_addr.push(Protocol::Tcp(DEFAULT_LISTEN_PORT));

        let peer_manager_hb_interval =
            Duration::from_secs(DEFAULT_PEER_MANAGER_HEART_BEAT_INTERVAL);

        NetworkConfig {
            default_listen:   listen_addr,
            max_connections:  DEFAULT_MAX_CONNECTIONS,
            max_frame_length: DEFAULT_MAX_FRAME_LENGTH,
            send_buffer_size: DEFAULT_BUFFER_SIZE,
            recv_buffer_size: DEFAULT_BUFFER_SIZE,
            max_wait_streams: DEFAULT_MAX_WAIT_STREAMS,
            write_timeout:    DEFAULT_WRITE_TIMEOUT,

            bootstraps:             Default::default(),
            allowlist:              Default::default(),
            allowlist_only:         false,
            enable_save_restore:    false,
            peer_store_path:        PathBuf::from(DEFAULT_PEER_DAT_FILE.to_owned()),
            peer_trust_interval:    DEFAULT_PEER_TRUST_INTERVAL_DURATION,
            peer_trust_max_history: DEFAULT_PEER_TRUST_MAX_HISTORY_DURATION,
            peer_fatal_ban:         DEFAULT_PEER_FATAL_BAN_DURATION,
            peer_soft_ban:          DEFAULT_PEER_SOFT_BAN_DURATION,
            same_ip_conn_limit:     DEFAULT_SAME_IP_CONN_LIMIT,
            inbound_conn_limit:     DEFAULT_INBOUND_CONN_LIMIT,

            secio_keypair: SecioKeyPair::secp256k1_generated(),

            ping_interval:           Duration::from_secs(DEFAULT_PING_INTERVAL),
            ping_timeout:            Duration::from_secs(DEFAULT_PING_TIMEOUT),
            discovery_sync_interval: Duration::from_secs(DEFAULT_DISCOVERY_SYNC_INTERVAL),

            peer_manager_heart_beat_interval: peer_manager_hb_interval,
            heart_beat_interval:              Duration::from_secs(DEFAULT_SELF_HEART_BEAT_INTERVAL),

            rpc_timeout: Duration::from_secs(DEFAULT_RPC_TIMEOUT),
        }
    }

    pub fn from_config(config: &Config) -> ProtocolResult<Self> {
        let mut network_config = Self::new()
            .peer_store_dir(config.data_path.clone().join("peer_store"))
            .ping_interval(config.network.ping_interval)
            .max_frame_length(config.network.max_frame_length)
            .send_buffer_size(config.network.send_buffer_size)
            .recv_buffer_size(config.network.recv_buffer_size)
            .bootstraps(config.network.bootstraps.clone().unwrap_or_default().iter().map(|addr| addr.multi_address.clone()).collect())
            // .allowlist(allowlist)?
            .listen_addr(config.network.listening_address.clone())
            .secio_keypair(&config.privkey.as_string_trim0x());

        network_config = network_config.max_connections(config.network.max_connected_peers)?;

        Ok(network_config)
    }

    pub fn max_connections(mut self, max: Option<usize>) -> ProtocolResult<Self> {
        if let Some(max) = max {
            if max <= self.inbound_conn_limit {
                return Err(NetworkError::InboundLimitEqualOrSmallerThanMaxConn.into());
            }
            self.max_connections = max;
        }
        Ok(self)
    }

    pub fn listen_addr(mut self, addr: Multiaddr) -> Self {
        self.default_listen = addr;
        self
    }

    pub fn max_frame_length(mut self, max: Option<usize>) -> Self {
        if let Some(max) = max {
            self.max_frame_length = max;
        }

        self
    }

    pub fn send_buffer_size(mut self, size: Option<usize>) -> Self {
        if let Some(size) = size {
            self.send_buffer_size = size;
        }

        self
    }

    pub fn recv_buffer_size(mut self, size: Option<usize>) -> Self {
        if let Some(size) = size {
            self.recv_buffer_size = size;
        }

        self
    }

    pub fn bootstraps(mut self, addrs: Vec<Multiaddr>) -> Self {
        self.bootstraps = addrs;
        self
    }

    pub fn secio_keypair(mut self, sk_hex: &str) -> Self {
        let skp = hex_decode(sk_hex)
            .map(SecioKeyPair::secp256k1_raw_key)
            .unwrap()
            .unwrap();
        self.secio_keypair = skp;

        self
    }

    pub fn ping_interval(mut self, interval: Option<u64>) -> Self {
        if let Some(interval) = interval {
            self.ping_interval = Duration::from_secs(interval);
        }

        self
    }

    pub fn ping_timeout(mut self, timeout: u64) -> Self {
        self.ping_timeout = Duration::from_secs(timeout);

        self
    }

    pub fn discovery_sync_interval(mut self, interval: u64) -> Self {
        self.discovery_sync_interval = Duration::from_secs(interval);

        self
    }

    pub fn peer_store_dir(mut self, path: PathBuf) -> Self {
        self.peer_store_path = path;
        self
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig::new()
    }
}
