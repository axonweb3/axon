use std::fmt;
use std::str::FromStr;
use protocol::codec::ProtocolCodec;
use protocol::types::{
    AccessList, Block, Bloom, Bytes, Hash, Hasher, Log, Public, Receipt, SignatureComponents,
    SignedTransaction, Transaction, TransactionAction, UnverifiedTransaction, H160, H256, H64,
    U256, U64,
};
use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[allow(clippy::large_enum_variant)]
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RichTransactionOrHash {
    Hash(Hash),
    Rich(SignedTransaction),
}

impl Serialize for RichTransactionOrHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RichTransactionOrHash::Hash(h) => h.serialize(serializer),
            RichTransactionOrHash::Rich(stx) => stx.serialize(serializer),
        }
    }
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
#[serde(rename_all = "camelCase")]
pub struct Web3SendTrancationRequest {
    pub data:     Option<Bytes>,
    pub from:     Option<H160>,
    pub to:       Option<H160>,
    pub gas:      U256,
    pub gasprice: U256,
    pub value:    Option<U256>,
}

impl Web3SendTrancationRequest {
    pub fn create_signedtransaction_by_web3sendtrancationrequest(&self) -> SignedTransaction {
        SignedTransaction {
            transaction: UnverifiedTransaction {
                unsigned:  Transaction {
                    nonce:                    U256::default(),
                    max_priority_fee_per_gas: U256::default(),
                    gas_price:                self.gasprice,
                    gas_limit:                U256::from_str("0x1000000000").unwrap(),
                    action:                   TransactionAction::Create,
                    value:                    U256::default(),
                    data:                     if let Some(dx) = &self.data {
                        dx.clone()
                    } else {
                        Bytes::default()
                    },
                    access_list:              Vec::new(),
                },
                signature: Some(SignatureComponents {
                    standard_v: 0,
                    r:          H256::default(),
                    s:          H256::default(),
                }),
                chain_id:  0x1389,
                hash:      H256::default(),
            },
            sender:      if let Some(addr) = self.from {
                addr
            } else {
                H160::default()
            },
            public:      Public::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Web3CallRequest {
    pub address: Option<H160>,
    pub to:      H160,
    pub data:    Option<Bytes>,
}

impl Web3CallRequest {
    pub fn create_signedtransaction_by_web3allrequest(&self) -> SignedTransaction {
        SignedTransaction {
            transaction: UnverifiedTransaction {
                unsigned:  Transaction {
                    nonce:                    U256::default(),
                    max_priority_fee_per_gas: U256::default(),
                    gas_price:                U256::default(),
                    gas_limit:                U256::from_str("0x1000000000").unwrap(),
                    action:                   TransactionAction::Call(self.to),
                    value:                    U256::default(),
                    data:                     if let Some(dx) = &self.data {
                        dx.clone()
                    } else {
                        Bytes::default()
                    },
                    access_list:              Vec::new(),
                },
                signature: Some(SignatureComponents {
                    standard_v: 0,
                    r:          H256::default(),
                    s:          H256::default(),
                }),
                chain_id:  0u64,
                hash:      H256::default(),
            },
            sender:      if let Some(addr) = self.address {
                addr
            } else {
                H160::default()
            },
            public:      Public::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Web3TransactionReceipt {
    pub block_number:        u64,
    pub block_hash:          H256,
    pub contract_address:    Option<H160>,
    pub cumulative_gas_used: U256,
    pub effective_gas_price: U256,
    pub from:                H160,
    pub gas_used:            U256,
    pub logs:                Vec<Log>,
    pub logs_bloom:          Bloom,
    pub status:              U256,
    pub to:                  Option<H160>,
    pub transaction_hash:    Hash,
    pub transaction_index:   Option<U256>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub transaction_type:    Option<U64>,
}

impl Web3TransactionReceipt {
    pub fn create_new(receipt: Receipt, tx: SignedTransaction) -> Web3TransactionReceipt {
        Web3TransactionReceipt {
            block_number:        receipt.block_number,
            block_hash:          receipt.block_hash,
            contract_address:    None,
            cumulative_gas_used: receipt.used_gas,
            effective_gas_price: receipt.used_gas,
            from:                tx.sender,
            gas_used:            receipt.used_gas,
            logs:                receipt.logs,
            logs_bloom:          receipt.logs_bloom,
            status:              U256::zero(),
            to:                  if let TransactionAction::Call(to) = tx.transaction.unsigned.action
            {
                Some(to)
            } else {
                None
            },
            transaction_hash:    tx.transaction.hash,
            transaction_index:   Some(receipt.tx_index.into()),
            transaction_type:    None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Web3Block {
    pub number:            U256,
    pub hash:              Hash,
    pub parent_hash:       Hash,
    pub nonce:             H64,
    #[serde(rename = "sha3Uncles")]
    pub sha3_uncles:       Hash,
    pub logs_bloom:        Bloom,
    pub transactions_root: Hash,
    pub state_root:        Hash,
    pub receipts_root:     Hash,
    pub miner:             H160,
    pub difficury:         U256,
    pub total_difficulty:  U256,
    pub extra_data:        Bytes,
    pub size:              u64,
    pub gas_limit:         U256,
    pub gas_used:          U256,
    pub timestamp:         U256,
    pub transactions:      Vec<RichTransactionOrHash>,
    pub uncles:            Vec<Hash>,
}

impl From<Block> for Web3Block {
    fn from(b: Block) -> Self {
        let encode = b.header.encode().unwrap();
        Web3Block {
            number:            b.header.number.into(),
            hash:              Hasher::digest(&encode),
            parent_hash:       b.header.prev_hash,
            nonce:             b.header.nonce,
            sha3_uncles:       Default::default(),
            logs_bloom:        b.header.log_bloom,
            transactions_root: b.header.transactions_root,
            state_root:        b.header.state_root,
            receipts_root:     b.header.receipts_root,
            miner:             b.header.proposer,
            difficury:         b.header.difficulty,
            total_difficulty:  b.header.difficulty,
            extra_data:        b.header.extra_data,
            size:              encode.len() as u64,
            gas_limit:         b.header.gas_limit,
            gas_used:          b.header.gas_used,
            timestamp:         b.header.timestamp.into(),
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
#[serde(rename_all = "camelCase")]
pub struct Web3Transaction {
    pub hash:                     Hash,
    pub nonce:                    U256,
    pub block_hash:               Option<Hash>,
    pub block_number:             Option<U256>,
    pub transaction_index:        Option<U256>,
    pub from:                     H160,
    pub to:                       Option<H160>,
    pub value:                    U256,
    pub gas_price:                U256,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas:          Option<U256>,
    pub gas:                      U256,
    pub input:                    Bytes,
    pub v:                        U256,
    pub r:                        U256,
    pub s:                        U256,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list:              Option<AccessList>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>,
}

impl Web3Transaction {
    pub fn _new(stx: SignedTransaction, receipt: Receipt) -> Web3Transaction {
        let signature = stx.transaction.signature.clone().unwrap();
        Web3Transaction {
            hash:                     stx.transaction.hash,
            nonce:                    stx.transaction.unsigned.nonce,
            block_hash:               Some(receipt.block_hash),
            block_number:             Some(receipt.block_number.into()),
            transaction_index:        Some(receipt.tx_index.into()),
            from:                     stx.sender,
            to:                       if let TransactionAction::Call(to) =
                stx.transaction.unsigned.action
            {
                Some(to)
            } else {
                None
            },
            value:                    stx.transaction.unsigned.value,
            gas_price:                stx.transaction.unsigned.gas_price,
            max_fee_per_gas:          None,
            gas:                      receipt.used_gas,
            input:                    stx.transaction.unsigned.data,
            v:                        signature.standard_v.into(),
            r:                        signature.r.as_ref().into(),
            s:                        signature.s.as_ref().into(),
            access_list:              Some(stx.transaction.unsigned.access_list.clone()),
            max_priority_fee_per_gas: Some(stx.transaction.unsigned.max_priority_fee_per_gas),
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
