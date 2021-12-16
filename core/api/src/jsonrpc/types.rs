use std::fmt;

use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use protocol::codec::ProtocolCodec;
use protocol::types::{
    AccessList, Block, Bloom, Bytes, Hash, Hasher, SignedTransaction, H160, H64, U256, U64,
};

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RichTransactionOrHash {
    Hash(Hash),
    Rich(SignedTransaction),
}

impl RichTransactionOrHash {
    pub fn get_hash(&self) -> Hash {
        match self {
            RichTransactionOrHash::Hash(hash) => *hash,
            RichTransactionOrHash::Rich(stx) => stx.transaction.hash,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Web3Block {
    pub number:            u64,
    pub hash:              Hash,
    pub parent_hash:       Hash,
    pub nonce:             H64,
    pub sha3_uncles:       Hash,
    pub logs_bloom:        Bloom,
    pub transactions_root: Hash,
    pub state_root:        Hash,
    pub receipts_root:     Hash,
    pub miner:             H160,
    pub difficury:         u64,
    pub total_difficulty:  u64,
    pub extra_data:        Bytes,
    pub size:              u64,
    pub gas_limit:         U256,
    pub gas_used:          U256,
    pub timestamp:         u64,
    pub transactions:      Vec<RichTransactionOrHash>,
    pub uncles:            Vec<Hash>,
}

impl From<Block> for Web3Block {
    fn from(b: Block) -> Self {
        let encode = b.header.encode().unwrap();
        Web3Block {
            number:            b.header.number,
            hash:              Hasher::digest(&encode),
            parent_hash:       b.header.prev_hash,
            nonce:             b.header.nonce,
            sha3_uncles:       Default::default(),
            logs_bloom:        b.header.log_bloom,
            transactions_root: b.header.transactions_root,
            state_root:        b.header.state_root,
            receipts_root:     b.header.receipts_root,
            miner:             b.header.proposer,
            difficury:         b.header.difficulty.as_u64(),
            total_difficulty:  b.header.difficulty.as_u64(),
            extra_data:        b.header.extra_data,
            size:              encode.len() as u64,
            gas_limit:         b.header.gas_limit,
            gas_used:          b.header.gas_used,
            timestamp:         b.header.timestamp,
            transactions:      b
                .tx_hashes
                .iter()
                .map(|hash| RichTransactionOrHash::Hash(*hash))
                .collect(),
            uncles:            vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CallRequest {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub transaction_type:         Option<U64>,
    pub from:                     Option<H160>,
    pub to:                       Option<H160>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price:                Option<U256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas:          Option<U256>,
    pub gas:                      Option<U256>,
    pub value:                    Option<U256>,
    pub data:                     Option<Bytes>,
    pub nonce:                    Option<U256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list:              Option<AccessList>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockId {
    Num(u64),
    Latest,
}

impl Default for BlockId {
    fn default() -> Self {
        BlockId::Latest
    }
}

impl<'a> Deserialize<'a> for BlockId {
    fn deserialize<D>(deserializer: D) -> Result<BlockId, D::Error>
    where
        D: Deserializer<'a>,
    {
        deserializer.deserialize_any(BlockIdVisitor)
    }
}

impl Serialize for BlockId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            BlockId::Num(ref x) => serializer.serialize_str(&format!("0x{:x}", x)),
            BlockId::Latest => serializer.serialize_str("latest"),
        }
    }
}

struct BlockIdVisitor;

impl<'a> Visitor<'a> for BlockIdVisitor {
    type Value = BlockId;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a block number or 'latest' ")
    }

    #[allow(clippy::never_loop)]
    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'a>,
    {
        let mut block_number = None;

        loop {
            let key_str: Option<String> = visitor.next_key()?;

            match key_str {
                Some(key) => match key.as_str() {
                    "blockNumber" => {
                        let value: String = visitor.next_value()?;
                        if let Some(stripper) = value.strip_prefix("0x") {
                            let number = u64::from_str_radix(stripper, 16).map_err(|e| {
                                Error::custom(format!("Invalid block number: {}", e))
                            })?;

                            block_number = Some(number);
                            break;
                        } else {
                            return Err(Error::custom(
                                "Invalid block number: missing 0x prefix".to_string(),
                            ));
                        }
                    }
                    key => return Err(Error::custom(format!("Unknown key: {}", key))),
                },
                None => break,
            };
        }

        if let Some(number) = block_number {
            return Ok(BlockId::Num(number));
        }

        Err(Error::custom("Invalid input"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match value {
            "latest" => Ok(BlockId::Latest),
            _ if value.starts_with("0x") => u64::from_str_radix(&value[2..], 16)
                .map(BlockId::Num)
                .map_err(|e| Error::custom(format!("Invalid block number: {}", e))),
            _ => Err(Error::custom(
                "Invalid block number: missing 0x prefix".to_string(),
            )),
        }
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_str(value.as_ref())
    }
}

#[derive(Debug, PartialEq)]
pub struct Index(usize);

impl Index {
    pub fn _value(&self) -> usize {
        self.0
    }
}

impl<'a> Deserialize<'a> for Index {
    fn deserialize<D>(deserializer: D) -> Result<Index, D::Error>
    where
        D: Deserializer<'a>,
    {
        deserializer.deserialize_any(IndexVisitor)
    }
}

struct IndexVisitor;

impl<'a> Visitor<'a> for IndexVisitor {
    type Value = Index;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a hex-encoded or decimal index")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match value {
            _ if value.starts_with("0x") => usize::from_str_radix(&value[2..], 16)
                .map(Index)
                .map_err(|e| Error::custom(format!("Invalid index: {}", e))),
            _ => value
                .parse::<usize>()
                .map(Index)
                .map_err(|e| Error::custom(format!("Invalid index: {}", e))),
        }
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_str(value.as_ref())
    }
}
