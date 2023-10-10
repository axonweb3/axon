use std::{ffi::OsStr, fmt, fs::File, io::Read as _, path::PathBuf};

use clap::{
    builder::{StringValueParser, TypedValueParser, ValueParserFactory},
    Args, ValueEnum,
};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use common_crypto::Secp256k1RecoverablePrivateKey;
use protocol::{
    codec::{decode_256bits_key, deserialize_address},
    types::{
        HardforkInfoInner, Header, Key256Bits, Metadata, H160, H256, RLP_EMPTY_LIST, RLP_NULL, U256,
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
        parse_file(&file_path, false).map_err(|err| {
            let kind = clap::error::ErrorKind::InvalidValue;
            let msg = format!(
                "failed to parse chain spec file {} since {err}",
                file_path.display()
            );
            clap::Error::raw(kind, msg)
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
    /// Build a `Header` of the genesis block from the user provided parameters.
    pub fn build_header(&self) -> Header {
        Header {
            transactions_root: RLP_NULL,
            signed_txs_hash: RLP_EMPTY_LIST,
            timestamp: self.timestamp,
            base_fee_per_gas: self.base_fee_per_gas,
            chain_id: self.chain_id,
            ..Default::default()
        }
    }

    pub fn generate_hardfork_info(&self) -> HardforkInfoInner {
        Into::<HardforkInfoInner>::into(HardforkInput {
            hardforks:    self.hardforks.clone(),
            block_number: 0,
        })
    }
}

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
        let convert_fn = |hardforks: Vec<HardforkName>| -> H256 {
            let r = hardforks.into_iter().fold(0, |acc, s| acc | s as u64);

            H256::from_low_u64_be(r.to_be())
        };

        let flags = if value.hardforks.is_empty() {
            H256::from_low_u64_be(HardforkName::all().to_be())
        } else if value.hardforks.len() == 1 {
            if value.hardforks[0] == HardforkName::None {
                H256::zero()
            } else {
                convert_fn(value.hardforks)
            }
        } else {
            convert_fn(value.hardforks)
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
    Andromeda = 0b1,
}

impl HardforkName {
    pub fn all() -> u64 {
        HardforkName::Andromeda as u64
    }
}
