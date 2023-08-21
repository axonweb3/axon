use std::{ffi::OsStr, io, path::PathBuf};

use clap::builder::{StringValueParser, TypedValueParser, ValueParserFactory};
use serde::Deserialize;

use protocol::types::{Block, Bytes, Header, RichBlock, SignedTransaction, H160, U256};

use crate::parse_file;

/// The chain specification.
#[derive(Clone, Debug, Deserialize)]
pub struct ChainSpec {
    /// The data of the genesis block.
    pub genesis:  Genesis,
    /// Accounts since the genesis block.
    pub accounts: Vec<InitialAccount>,
    /// Parameters which make the chain to be unique.
    ///
    /// All parameters are not allowed to be modified after the chain
    /// initialized.
    pub params:   Params,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Genesis {
    pub timestamp:        u64,
    pub extra_data:       Bytes,
    pub base_fee_per_gas: U256,
    pub chain_id:         u64,

    #[serde(rename = "transactions")]
    pub txs_file: Option<PathBuf>,
    #[serde(skip)]
    pub txs:      Vec<SignedTransaction>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Params {}

#[derive(Clone, Debug, Deserialize)]
pub struct InitialAccount {
    pub address: H160,
    pub balance: U256,
}

impl ValueParserFactory for ChainSpec {
    type Parser = ChainSpecValueParser;

    fn value_parser() -> Self::Parser {
        ChainSpecValueParser
    }
}

#[derive(Clone, Debug)]
pub struct ChainSpecValueParser;

impl TypedValueParser for ChainSpecValueParser {
    type Value = ChainSpec;

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
            .map_err(|err| {
                let kind = clap::error::ErrorKind::InvalidValue;
                let msg = format!(
                    "failed to parse chain spec file {} since {err}",
                    file_path.display()
                );
                clap::Error::raw(kind, msg)
            })
            .and_then(|mut spec: Self::Value| {
                if let Some(ref mut f) = spec.genesis.txs_file {
                    let txs_file = dir_path.join(&f);
                    let txs: Vec<SignedTransaction> =
                        parse_file(&txs_file, true).map_err(|err| {
                            let kind = clap::error::ErrorKind::InvalidValue;
                            let msg = format!(
                                "failed to parse transactions json file {} since {err}",
                                txs_file.display()
                            );
                            clap::Error::raw(kind, msg)
                        })?;
                    *f = txs_file;
                    spec.genesis.txs = txs;
                }
                Ok(spec)
            })
    }
}

impl Genesis {
    /// Build a `RichBlock` of the genesis block from the user provided
    /// parameters.
    pub fn build_rich_block(&self) -> RichBlock {
        let block = self.build_block();
        let txs = self.txs.clone();
        RichBlock { block, txs }
    }

    /// Build a `Block` of the genesis block from the user provided parameters.
    pub fn build_block(&self) -> Block {
        let header = self.build_header();
        let tx_hashes = self.txs.iter().map(|tx| tx.transaction.hash).collect();
        Block { header, tx_hashes }
    }

    /// Build a `Header` of the genesis block from the user provided parameters.
    pub fn build_header(&self) -> Header {
        Header {
            timestamp: self.timestamp,
            extra_data: self.extra_data.clone(),
            base_fee_per_gas: self.base_fee_per_gas,
            chain_id: self.chain_id,
            ..Default::default()
        }
    }
}
