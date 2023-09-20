use std::{
    ffi::OsStr,
    fmt,
    fs::File,
    io::{self, Read as _},
    path::PathBuf,
};

use clap::{
    builder::{StringValueParser, TypedValueParser, ValueParserFactory},
    Args,
};
use serde::{Deserialize, Serialize};

use common_crypto::Secp256k1RecoverablePrivateKey;
use protocol::{
    codec::{decode_256bits_key, deserialize_address, ProtocolCodec},
    types::{
        Block, ExtraData, HardforkInfoInner, Header, Key256Bits, Metadata, RichBlock,
        SignedTransaction, H160, H256, U256,
    },
};

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
    pub params:   Metadata,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Genesis {
    pub timestamp:        u64,
    pub hardforks:        Vec<HardforkName>,
    pub base_fee_per_gas: U256,
    pub chain_id:         u64,

    #[serde(rename = "transactions")]
    pub txs_file: Option<PathBuf>,
    #[serde(skip)]
    pub txs:      Vec<SignedTransaction>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct InitialAccount {
    #[serde(deserialize_with = "deserialize_address")]
    pub address: H160,
    pub balance: U256,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct PrivateKey {
    #[arg(
        short = 'k',
        long = "key",
        value_name = "PRIVATE_KEY",
        help = "The private key which is used to generate transactions in genesis block,",
        value_parser = PrivateKeyDataValueParser,
    )]
    from_cli:  Option<Key256Bits>,
    #[arg(
        long = "key-file",
        value_name = "PRIVATE_KEY_FILE",
        help = "File path of the private key to generate transactions in genesis block,",
        value_parser = PrivateKeyFileValueParser,
    )]
    from_file: Option<Key256Bits>,
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

impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.from_cli.is_some() {
            write!(
                f,
                "**** a hidden 256 bits secret key (from hex string) ****"
            )
        } else {
            write!(f, "**** a hidden 256 bits secret key (from filepath) ****")
        }
    }
}

impl PrivateKey {
    pub fn data(self) -> Result<Secp256k1RecoverablePrivateKey, String> {
        let Self {
            from_cli,
            from_file,
        } = self;
        match (from_cli, from_file) {
            (Some(data), None) | (None, Some(data)) => {
                Secp256k1RecoverablePrivateKey::try_from(data.as_ref())
                    .map_err(|err| err.to_string())
            }
            _ => {
                let msg = "failed to parse the private key";
                Err(msg.to_owned())
            }
        }
    }
}

#[derive(Clone)]
pub struct PrivateKeyDataValueParser;

impl TypedValueParser for PrivateKeyDataValueParser {
    type Value = Key256Bits;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        StringValueParser::new()
            .parse_ref(cmd, arg, value)
            .and_then(|s| {
                decode_256bits_key(&s).map_err(|err| {
                    let kind = clap::error::ErrorKind::InvalidValue;
                    let msg = format!("failed to parse private key since {err}",);
                    clap::Error::raw(kind, msg)
                })
            })
    }
}

#[derive(Clone)]
pub struct PrivateKeyFileValueParser;

impl TypedValueParser for PrivateKeyFileValueParser {
    type Value = Key256Bits;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        StringValueParser::new()
            .parse_ref(cmd, arg, value)
            .map(PathBuf::from)
            .and_then(|p| {
                File::open(p)
                    .and_then(|mut f| {
                        let mut buffer = Vec::new();
                        f.read_to_end(&mut buffer).map(|_| buffer)
                    })
                    .map_err(|err| {
                        let kind = clap::error::ErrorKind::InvalidValue;
                        let msg = format!("failed to parse private key file since {err}",);
                        clap::Error::raw(kind, msg)
                    })
            })
            .and_then(|bytes| {
                const LEN: usize = 32;
                if bytes.len() == LEN {
                    let mut v = [0u8; 32];
                    v.copy_from_slice(&bytes);
                    Ok(Self::Value::from(v))
                } else {
                    let kind = clap::error::ErrorKind::InvalidValue;
                    let msg = format!(
                        "failed to parse private key file since its length is {} but expect {LEN}.",
                        bytes.len()
                    );
                    Err(clap::Error::raw(kind, msg))
                }
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
            // todo: if Hardforkinput is empty, it must change to latest hardfork info to init
            // genesis
            extra_data: {
                vec![ExtraData {
                    inner: Into::<HardforkInfoInner>::into(HardforkInput {
                        hardforks:    self.hardforks.clone(),
                        block_number: 0,
                    })
                    .encode()
                    .unwrap(),
                }]
            },
            base_fee_per_gas: self.base_fee_per_gas,
            chain_id: self.chain_id,
            ..Default::default()
        }
    }
}

use clap::ValueEnum;
use strum_macros::EnumIter;

#[derive(Clone, Debug, Deserialize, Args)]
pub struct HardforkInput {
    #[arg(
        long = "hardfork-start-number",
        required = false,
        requires = "hardforks"
    )]
    pub block_number: u64,
    #[arg(long = "feature", requires = "block_number")]
    pub hardforks:    Vec<HardforkName>,
}

impl From<HardforkInput> for HardforkInfoInner {
    fn from(value: HardforkInput) -> Self {
        let flags = {
            let r = value.hardforks.into_iter().fold(0, |acc, s| acc | s as u64);

            H256::from_low_u64_be(r.to_be())
        };

        HardforkInfoInner {
            block_number: value.block_number,
            flags,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Copy, ValueEnum, EnumIter, PartialEq, Eq, Hash)]
pub enum HardforkName {
    None = 0b0,
}
