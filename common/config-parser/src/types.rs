use std::{
    collections::HashMap, ffi::OsStr, io, marker::PhantomData, net::SocketAddr, path::PathBuf,
};

use clap::builder::{StringValueParser, TypedValueParser, ValueParserFactory};
use serde::{de, Deserialize};
use tentacle_multiaddr::MultiAddr;

use protocol::types::{Hex, H160, U256};

use crate::parse_file;

pub const DEFAULT_BROADCAST_TXS_SIZE: usize = 200;
pub const DEFAULT_BROADCAST_TXS_INTERVAL: u64 = 200; // milliseconds
pub const DEFAULT_SYNC_TXS_CHUNK_SIZE: usize = 5000;
pub const DEFAULT_CACHE_SIZE: usize = 100;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    // crypto
    pub privkey:   Privkey,
    // db config
    pub data_path: PathBuf,

    pub accounts: Vec<InitialAccount>,

    pub rpc:        ConfigApi,
    pub web3:       ConfigWeb3,
    pub network:    ConfigNetwork,
    pub mempool:    ConfigMempool,
    pub executor:   ConfigExecutor,
    pub consensus:  ConfigConsensus,
    #[serde(default)]
    pub logger:     ConfigLogger,
    #[serde(default)]
    pub rocksdb:    ConfigRocksDB,
    pub jaeger:     Option<ConfigJaeger>,
    pub prometheus: Option<ConfigPrometheus>,

    #[serde(default)]
    pub ibc_contract_address:  H160,
    pub wckb_contract_address: H160,
}

impl Config {
    pub fn data_path_for_system_contract(&self) -> PathBuf {
        let mut path_state = self.data_path.clone();
        path_state.push("rocksdb");
        path_state.push("system_contract");
        path_state
    }

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

    pub fn data_path_for_version(&self) -> PathBuf {
        let mut path_state = self.data_path.clone();
        path_state.push("axon.ver");
        path_state
    }
}

impl ValueParserFactory for Config {
    type Parser = ConfigValueParser;

    fn value_parser() -> Self::Parser {
        ConfigValueParser
    }
}

#[derive(Clone, Debug)]
pub struct ConfigValueParser;

impl TypedValueParser for ConfigValueParser {
    type Value = Config;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let file_path = StringValueParser::new()
            .parse_ref(cmd, arg, value)
            .map(PathBuf::from)?;
        let dir_path = file_path.parent().ok_or_else(|| {
            let err = {
                let kind = io::ErrorKind::Other;
                let msg = format!("no parent directory of {}", file_path.display());
                io::Error::new(kind, msg)
            };
            let kind = clap::error::ErrorKind::InvalidValue;
            clap::Error::raw(kind, err)
        })?;
        parse_file(&file_path, false)
            .map(|mut config: Self::Value| {
                if let Some(ref mut f) = config.rocksdb.options_file {
                    *f = dir_path.join(&f)
                }
                config
            })
            .map_err(|err| {
                let kind = clap::error::ErrorKind::InvalidValue;
                let msg = format!("failed to parse file {} since {err}", file_path.display());
                clap::Error::raw(kind, msg)
            })
    }
}

#[derive(Clone, Debug)]
pub struct JsonValueParser<T: de::DeserializeOwned + 'static + Clone + Send + Sync>(PhantomData<T>);

impl<T> Default for JsonValueParser<T>
where
    T: de::DeserializeOwned + 'static + Clone + Send + Sync,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> TypedValueParser for JsonValueParser<T>
where
    T: de::DeserializeOwned + 'static + Clone + Send + Sync,
{
    type Value = T;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let file_path = StringValueParser::new()
            .parse_ref(cmd, arg, value)
            .map(PathBuf::from)?;
        parse_file(&file_path, true).map_err(|err| {
            let kind = clap::error::ErrorKind::InvalidValue;
            let msg = format!(
                "failed to parse JSON file {} since {err}",
                file_path.display()
            );
            clap::Error::raw(kind, msg)
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigApi {
    pub http_listening_address: Option<SocketAddr>,
    pub ws_listening_address:   Option<SocketAddr>,
    pub maxconn:                u32,
    pub max_payload_size:       u32,
    pub enable_dump_profile:    Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigWeb3 {
    #[serde(default = "default_log_filter_max_block_range")]
    pub log_filter_max_block_range: u64,
    #[serde(default = "default_max_gas_cap")]
    pub max_gas_cap:                u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigNetwork {
    pub bootstraps:          Option<Vec<ConfigNetworkBootstrap>>,
    pub allowlist:           Option<Vec<String>>,
    pub allowlist_only:      Option<bool>,
    pub max_connected_peers: Option<usize>,
    pub inbound_conn_limit:  Option<usize>,
    pub listening_address:   MultiAddr,
    pub rpc_timeout:         Option<u64>,
    pub send_buffer_size:    Option<usize>,
    pub recv_buffer_size:    Option<usize>,
    pub max_frame_length:    Option<usize>,
    pub ping_interval:       Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigNetworkBootstrap {
    pub multi_address: MultiAddr,
}

fn default_sync_txs_chunk_size() -> usize {
    DEFAULT_SYNC_TXS_CHUNK_SIZE
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigConsensus {
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
pub struct InitialAccount {
    pub address: H160,
    pub balance: U256,
}

fn default_max_gas_cap() -> u64 {
    25_000_000
}

fn default_log_filter_max_block_range() -> u64 {
    10_000
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Privkey(Hex);

impl Privkey {
    pub fn as_string_trim0x(&self) -> String {
        self.0.as_string_trim0x()
    }

    pub fn inner(self) -> Hex {
        self.0
    }
}

impl serde::Serialize for Privkey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Privkey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct PVisitor;

        impl<'de> serde::de::Visitor<'de> for PVisitor {
            type Value = Privkey;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Expect a hex string")
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match Hex::from_string(v.clone()) {
                    Ok(v) => Ok(Privkey(v)),
                    Err(_) => {
                        let key = std::env::var(v)
                            .map_err(|e| serde::de::Error::custom(e.to_string()))?;
                        Hex::from_string(key)
                            .map(Privkey)
                            .map_err(|e| serde::de::Error::custom(e.to_string()))
                    }
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match Hex::from_string(v.to_string()) {
                    Ok(v) => Ok(Privkey(v)),
                    Err(_) => {
                        let key = std::env::var(v)
                            .map_err(|e| serde::de::Error::custom(e.to_string()))?;
                        Hex::from_string(key)
                            .map(Privkey)
                            .map_err(|e| serde::de::Error::custom(e.to_string()))
                    }
                }
            }
        }

        deserializer.deserialize_string(PVisitor)
    }
}
