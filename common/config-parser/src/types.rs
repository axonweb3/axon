use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;

use serde::Deserialize;
use tentacle_multiaddr::MultiAddr;

use protocol::types::{Hex, H160, H256, U256};

pub const DEFAULT_BROADCAST_TXS_SIZE: usize = 200;
pub const DEFAULT_BROADCAST_TXS_INTERVAL: u64 = 200; // milliseconds
pub const DEFAULT_OVERLORD_GAP: usize = 5;
pub const DEFAULT_SYNC_TXS_CHUNK_SIZE: usize = 5000;
pub const DEFAULT_CACHE_SIZE: usize = 128 << 20;

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigApi {
    pub http_listening_address: Option<SocketAddr>,
    pub ws_listening_address:   Option<SocketAddr>,
    #[serde(default)]
    pub maxconn:                usize,
    #[serde(default)]
    pub max_payload_size:       usize,
    pub enable_dump_profile:    Option<bool>,
    #[serde(default)]
    pub client_version:         String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigGraphQLTLS {
    pub private_key_file_path:       PathBuf,
    pub certificate_chain_file_path: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigNetwork {
    pub bootstraps:                 Option<Vec<ConfigNetworkBootstrap>>,
    pub allowlist:                  Option<Vec<String>>,
    pub allowlist_only:             Option<bool>,
    pub trust_interval_duration:    Option<u64>,
    pub trust_max_history_duration: Option<u64>,
    pub fatal_ban_duration:         Option<u64>,
    pub soft_ban_duration:          Option<u64>,
    pub max_connected_peers:        Option<usize>,
    pub same_ip_conn_limit:         Option<usize>,
    pub inbound_conn_limit:         Option<usize>,
    pub listening_address:          MultiAddr,
    pub rpc_timeout:                Option<u64>,
    pub selfcheck_interval:         Option<u64>,
    pub send_buffer_size:           Option<usize>,
    pub write_timeout:              Option<u64>,
    pub recv_buffer_size:           Option<usize>,
    pub max_frame_length:           Option<usize>,
    pub max_wait_streams:           Option<usize>,
    pub ping_interval:              Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigNetworkBootstrap {
    pub multi_address: MultiAddr,
}

fn default_overlord_gap() -> usize {
    DEFAULT_OVERLORD_GAP
}

fn default_sync_txs_chunk_size() -> usize {
    DEFAULT_SYNC_TXS_CHUNK_SIZE
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigConsensus {
    #[serde(default = "default_overlord_gap")]
    pub overlord_gap:        usize,
    #[serde(default = "default_sync_txs_chunk_size")]
    pub sync_txs_chunk_size: usize,
}

fn default_broadcast_txs_size() -> usize {
    DEFAULT_BROADCAST_TXS_SIZE
}

fn default_broadcast_txs_interval() -> u64 {
    DEFAULT_BROADCAST_TXS_INTERVAL
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigMempool {
    pub pool_size:   u64,
    pub timeout_gap: u64,

    #[serde(default = "default_broadcast_txs_size")]
    pub broadcast_txs_size:     usize,
    #[serde(default = "default_broadcast_txs_interval")]
    pub broadcast_txs_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigExecutor {
    pub light:             bool,
    pub triedb_cache_size: usize,
}

fn default_cache_size() -> usize {
    DEFAULT_CACHE_SIZE
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigRocksDB {
    pub max_open_files: i32,
    #[serde(default = "default_cache_size")]
    pub cache_size:     usize,
    pub options_file:   Option<PathBuf>,
}

impl Default for ConfigRocksDB {
    fn default() -> Self {
        Self {
            max_open_files: 64,
            cache_size:     default_cache_size(),
            options_file:   None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigLogger {
    pub filter:                     String,
    pub log_to_console:             bool,
    pub console_show_file_and_line: bool,
    pub log_to_file:                bool,
    pub metrics:                    bool,
    pub log_path:                   PathBuf,
    pub file_size_limit:            u64,
    #[serde(default)]
    pub modules_level:              HashMap<String, String>,
}

impl Default for ConfigLogger {
    fn default() -> Self {
        Self {
            filter:                     "info".into(),
            log_to_console:             true,
            console_show_file_and_line: false,
            log_to_file:                true,
            metrics:                    true,
            log_path:                   "logs/".into(),
            file_size_limit:            1024 * 1024 * 1024, // GiB
            modules_level:              HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigJaeger {
    pub service_name:       Option<String>,
    pub tracing_address:    Option<SocketAddr>,
    pub tracing_batch_size: Option<usize>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigPrometheus {
    pub listening_address: Option<SocketAddr>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigCrossClient {
    pub axon_udt_hash:       H256,
    pub ckb_uri:             String,
    pub mercury_uri:         String,
    pub start_block_number:  u64,
    pub pk:                  Hex,
    pub enable:              bool,
    pub checkpoint_interval: u64,

    pub admin_address:        H160,
    pub node_address:         H160,
    pub selection_lock_hash:  H256,
    pub checkpoint_type_hash: H256,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BlockchainConfig {
    pub name:    String,
    pub id:      u8,
    pub tx_hash: H256,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigInteroperabilityExtension {
    pub blockchain_extension_transaction_hashes: Vec<BlockchainConfig>,
}

impl From<ConfigInteroperabilityExtension> for HashMap<u8, H256> {
    fn from(ie: ConfigInteroperabilityExtension) -> Self {
        ie.blockchain_extension_transaction_hashes
            .into_iter()
            .map(|v| (v.id, v.tx_hash))
            .collect()
    }
}

impl ConfigInteroperabilityExtension {
    pub fn get_hashes(&self) -> Vec<H256> {
        self.blockchain_extension_transaction_hashes
            .iter()
            .map(|v| v.tx_hash)
            .collect()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MnemonicAccountsConfig {
    pub mnemonic:      String,
    pub initial_index: Option<u32>,
    pub path:          Option<String>,
    pub count:         u32,
    pub balance:       U256,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    // crypto
    pub privkey:   Hex,
    // db config
    pub data_path: PathBuf,

    pub accounts: MnemonicAccountsConfig,

    pub rpc:                         ConfigApi,
    pub network:                     ConfigNetwork,
    pub mempool:                     ConfigMempool,
    pub executor:                    ConfigExecutor,
    pub consensus:                   ConfigConsensus,
    #[serde(default)]
    pub logger:                      ConfigLogger,
    #[serde(default)]
    pub rocksdb:                     ConfigRocksDB,
    pub jaeger:                      Option<ConfigJaeger>,
    pub prometheus:                  Option<ConfigPrometheus>,
    pub cross_client:                ConfigCrossClient,
    pub epoch_len:                   u64,
    pub metadata_contract_address:   H256,
    pub crosschain_contract_address: H256,
    pub interoperability_extension:  ConfigInteroperabilityExtension,
}

impl Config {
    pub fn data_path_for_state(&self) -> PathBuf {
        let mut path_state = self.data_path.clone();
        path_state.push("rocksdb");
        path_state.push("state_data");
        path_state
    }

    pub fn data_path_for_block(&self) -> PathBuf {
        let mut path_state = self.data_path.clone();
        path_state.push("rocksdb");
        path_state.push("block_data");
        path_state
    }

    pub fn data_path_for_txs_wal(&self) -> PathBuf {
        let mut path_state = self.data_path.clone();
        path_state.push("txs_wal");
        path_state
    }

    pub fn data_path_for_consensus_wal(&self) -> PathBuf {
        let mut path_state = self.data_path.clone();
        path_state.push("consensus_wal");
        path_state
    }
}
