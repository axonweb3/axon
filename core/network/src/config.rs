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

pub const DEFAULT_SAME_IP_CONN_LIMIT: usize = 1;
pub const DEFAULT_INBOUND_CONN_LIMIT: usize = 20;

// Default peer store persistent path
pub const DEFAULT_PEER_DAT_FILE: &str = "./";

pub const DEFAULT_PING_INTERVAL: u64 = 15;
pub const DEFAULT_PING_TIMEOUT: u64 = 30;

pub const DEFAULT_RPC_TIMEOUT: u64 = 10;

#[derive(Debug)]
pub struct NetworkConfig {
    // connection
    pub default_listen:   Multiaddr,
    pub max_connections:  usize,
    pub max_frame_length: usize,
    pub send_buffer_size: usize,
    pub recv_buffer_size: usize,

    // peer manager
    pub bootstraps:          Vec<Multiaddr>,
    pub allowlist:           Vec<PeerId>,
    pub allowlist_only:      bool,
    pub enable_save_restore: bool,
    pub peer_store_path:     PathBuf,
    pub inbound_conn_limit:  usize,

    // identity and encryption
    pub secio_keypair: SecioKeyPair,

    // protocol
    pub ping_interval: Duration,
    pub ping_timeout:  Duration,

    // rpc
    pub rpc_timeout: Duration,
}

impl NetworkConfig {
    pub fn new() -> Self {
        let mut listen_addr = Multiaddr::from(DEFAULT_LISTEN_IP_ADDR);
        listen_addr.push(Protocol::Tcp(DEFAULT_LISTEN_PORT));

        NetworkConfig {
            default_listen:   listen_addr,
            max_connections:  DEFAULT_MAX_CONNECTIONS,
            max_frame_length: DEFAULT_MAX_FRAME_LENGTH,
            send_buffer_size: DEFAULT_BUFFER_SIZE,
            recv_buffer_size: DEFAULT_BUFFER_SIZE,

            bootstraps:          Default::default(),
            allowlist:           Default::default(),
            allowlist_only:      false,
            enable_save_restore: false,
            peer_store_path:     PathBuf::from(DEFAULT_PEER_DAT_FILE.to_owned()),
            inbound_conn_limit:  DEFAULT_INBOUND_CONN_LIMIT,

            secio_keypair: SecioKeyPair::secp256k1_generated(),

            ping_interval: Duration::from_secs(DEFAULT_PING_INTERVAL),
            ping_timeout:  Duration::from_secs(DEFAULT_PING_TIMEOUT),

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
