use std::fmt;

use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use core_consensus::SyncStatus as InnerSyncStatus;
use protocol::codec::ProtocolCodec;
use protocol::types::{
    AccessList, Block, Bloom, Bytes, Hash, Header, Hex, Public, Receipt, SignedTransaction, H160,
    H256, U256, U64,
};

const EIP1559_TX_TYPE: u64 = 0x02;

#[allow(clippy::large_enum_variant)]
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RichTransactionOrHash {
    Hash(Hash),
    Rich(Web3Transaction),
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
            RichTransactionOrHash::Rich(tx) => tx.hash,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Web3Transaction {
    #[serde(rename = "type")]
    pub type_:                    Option<U64>,
    pub block_number:             Option<U256>,
    pub block_hash:               Option<H256>,
    pub hash:                     Hash,
    pub nonce:                    U256,
    pub transaction_index:        Option<U256>,
    pub from:                     H160,
    pub to:                       Option<H160>,
    pub value:                    U256,
    pub gas:                      U256,
    pub gas_price:                U256,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas:          Option<U256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>,
    pub raw:                      Hex,
    pub input:                    Hex,
    pub public_key:               Option<Public>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list:              Option<AccessList>,
    pub chain_id:                 Option<U256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard_v:               Option<U256>,
    pub v:                        U256,
    pub r:                        U256,
    pub s:                        U256,
}

impl From<SignedTransaction> for Web3Transaction {
    fn from(stx: SignedTransaction) -> Web3Transaction {
        let signature = stx.transaction.signature.clone().unwrap_or_default();
        Web3Transaction {
            type_:                    Some(EIP1559_TX_TYPE.into()),
            block_number:             None,
            block_hash:               None,
            raw:                      Hex::encode(stx.transaction.encode().unwrap()),
            public_key:               stx.public,
            gas:                      U256::zero(),
            gas_price:                stx.transaction.unsigned.gas_price(),
            max_fee_per_gas:          Some(U256::from(1337u64)),
            max_priority_fee_per_gas: stx.transaction.unsigned.max_priority_fee_per_gas(),
            hash:                     stx.transaction.hash,
            from:                     stx.sender,
            to:                       stx.get_to(),
            input:                    Hex::encode(stx.transaction.unsigned.data()),
            nonce:                    stx.transaction.unsigned.nonce(),
            transaction_index:        None,
            value:                    stx.transaction.unsigned.value(),
            access_list:              Some(stx.transaction.unsigned.access_list()),
            chain_id:                 Some(stx.transaction.chain_id.into()),
            standard_v:               None,
            v:                        signature.standard_v.into(),
            r:                        signature.r.as_ref().into(),
            s:                        signature.s.as_ref().into(),
        }
    }
}

impl From<(SignedTransaction, Receipt)> for Web3Transaction {
    fn from(stx_receipt: (SignedTransaction, Receipt)) -> Self {
        let (stx, receipt) = stx_receipt;
        let signature = stx.transaction.signature.clone().unwrap_or_default();
        Web3Transaction {
            type_:                    Some(EIP1559_TX_TYPE.into()),
            block_number:             Some(receipt.block_number.into()),
            block_hash:               Some(receipt.block_hash),
            raw:                      Hex::encode(stx.transaction.encode().unwrap()),
            public_key:               stx.public,
            gas:                      receipt.used_gas,
            gas_price:                stx.transaction.unsigned.gas_price(),
            max_fee_per_gas:          Some(U256::from(1337u64)),
            max_priority_fee_per_gas: stx.transaction.unsigned.max_priority_fee_per_gas(),
            hash:                     receipt.tx_hash,
            from:                     stx.sender,
            to:                       stx.get_to(),
            input:                    Hex::encode(stx.transaction.unsigned.data()),
            nonce:                    stx.transaction.unsigned.nonce(),
            transaction_index:        Some(receipt.tx_index.into()),
            value:                    stx.transaction.unsigned.value(),
            access_list:              Some(stx.transaction.unsigned.access_list()),
            chain_id:                 Some(stx.transaction.chain_id.into()),
            standard_v:               None,
            v:                        signature.standard_v.into(),
            r:                        signature.r.as_ref().into(),
            s:                        signature.s.as_ref().into(),
        }
    }
}

impl Web3Transaction {
    pub fn add_block_number(mut self, block_number: u64) -> Self {
        self.block_number = Some(block_number.into());
        self
    }

    pub fn add_block_hash(mut self, block_hash: H256) -> Self {
        self.block_hash = Some(block_hash);
        self
    }

    pub fn add_tx_index(mut self, index: usize) -> Self {
        self.transaction_index = Some(index.into());
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Web3Receipt {
    pub block_number:        U256,
    pub block_hash:          H256,
    pub contract_address:    Option<H160>,
    pub cumulative_gas_used: U256,
    pub effective_gas_price: U256,
    pub from:                H160,
    pub gas_used:            U256,
    pub logs:                Vec<Web3ReceiptLog>,
    pub logs_bloom:          Bloom,
    #[serde(rename = "root")]
    pub state_root:          Hash,
    pub status:              U256,
    pub to:                  Option<H160>,
    pub transaction_hash:    Hash,
    pub transaction_index:   Option<U256>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub transaction_type:    Option<U64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Web3ReceiptLog {
    pub address:           H160,
    pub topics:            Vec<H256>,
    pub data:              Hex,
    pub block_number:      U256,
    pub block_hash:        Hash,
    pub transaction_hash:  Hash,
    pub transaction_index: Option<U256>,
    pub log_index:         U256,
    pub removed:           bool,
}

impl Web3Receipt {
    pub fn new(receipt: Receipt, stx: SignedTransaction) -> Web3Receipt {
        let logs_list = receipt
            .logs
            .iter()
            .map(|log| Web3ReceiptLog {
                address:           log.address,
                topics:            log.topics.clone(),
                data:              Hex::encode(&log.data),
                block_number:      receipt.block_number.into(),
                block_hash:        receipt.block_hash,
                transaction_hash:  receipt.tx_hash,
                transaction_index: Some(receipt.tx_index.into()),
                log_index:         U256::zero(),
                removed:           false,
            })
            .collect::<Vec<_>>();

        let mut web3_receipt = Web3Receipt {
            block_number:        receipt.block_number.into(),
            block_hash:          receipt.block_hash,
            contract_address:    receipt.code_address.map(Into::into),
            cumulative_gas_used: receipt.used_gas,
            effective_gas_price: receipt.used_gas,
            from:                receipt.sender,
            status:              receipt.status(),
            gas_used:            receipt.used_gas,
            logs:                logs_list,
            logs_bloom:          receipt.logs_bloom,
            state_root:          receipt.state_root,
            to:                  stx.get_to(),
            transaction_hash:    receipt.tx_hash,
            transaction_index:   Some(receipt.tx_index.into()),
            transaction_type:    Some(EIP1559_TX_TYPE.into()),
        };
        for item in receipt.logs.into_iter() {
            web3_receipt.logs.push(Web3ReceiptLog {
                address:           item.address,
                topics:            item.topics,
                data:              Hex::encode(item.data),
                block_number:      receipt.block_number.into(),
                transaction_hash:  receipt.tx_hash,
                transaction_index: Some(receipt.tx_index.into()),
                block_hash:        receipt.block_hash,
                log_index:         receipt.log_index.into(),
                // Todo: FIXME
                removed:           false,
            });
        }
        web3_receipt
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Web3Block {
    pub hash:              H256,
    pub parent_hash:       H256,
    #[serde(rename = "sha3Uncles")]
    pub sha3_uncles:       H256,
    pub author:            H160,
    pub miner:             H160,
    pub state_root:        H256,
    pub transactions_root: H256,
    pub receipts_root:     H256,
    pub number:            U256,
    pub gas_used:          U256,
    pub gas_limit:         U256,
    pub extra_data:        Hex,
    pub logs_bloom:        Option<Bloom>,
    pub timestamp:         U256,
    pub difficulty:        U256,
    pub total_difficulty:  Option<U256>,
    pub seal_fields:       Vec<Bytes>,
    pub base_fee_per_gas:  U256,
    pub uncles:            Vec<H256>,
    pub transactions:      Vec<RichTransactionOrHash>,
    pub size:              Option<U256>,
    pub mix_hash:          H256,
    pub nonce:             U256,
}

impl From<Block> for Web3Block {
    fn from(b: Block) -> Self {
        Web3Block {
            hash:              b.header_hash(),
            number:            b.header.number.into(),
            author:            b.header.proposer,
            parent_hash:       b.header.prev_hash,
            sha3_uncles:       Default::default(),
            logs_bloom:        Some(b.header.log_bloom),
            transactions_root: b.header.transactions_root,
            state_root:        b.header.state_root,
            receipts_root:     b.header.receipts_root,
            miner:             b.header.proposer,
            difficulty:        b.header.difficulty,
            total_difficulty:  None,
            seal_fields:       vec![],
            base_fee_per_gas:  b.header.base_fee_per_gas,
            extra_data:        Hex::encode(&b.header.extra_data),
            size:              Some(b.header.size().into()),
            gas_limit:         b.header.gas_limit,
            gas_used:          b.header.gas_used,
            timestamp:         b.header.timestamp.into(),
            transactions:      b
                .tx_hashes
                .into_iter()
                .map(RichTransactionOrHash::Hash)
                .collect(),
            uncles:            vec![],
            mix_hash:          H256::default(),
            nonce:             U256::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TransactionCondition {
    #[serde(rename = "block")]
    Number(u64),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Web3CallRequest {
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
    pub data:                     Hex,
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

impl From<BlockId> for Option<u64> {
    fn from(id: BlockId) -> Self {
        match id {
            BlockId::Num(num) => Some(num),
            BlockId::Latest => None,
        }
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockIdWithPending {
    BlockId(BlockId),
    Pending,
}

impl<'a> Deserialize<'a> for BlockIdWithPending {
    fn deserialize<D>(deserializer: D) -> Result<BlockIdWithPending, D::Error>
    where
        D: Deserializer<'a>,
    {
        pub struct InnerVisitor;

        impl<'a> Visitor<'a> for InnerVisitor {
            type Value = BlockIdWithPending;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a block number or 'latest' or 'pending' ")
            }

            fn visit_map<V>(self, visitor: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'a>,
            {
                BlockIdVisitor
                    .visit_map(visitor)
                    .map(BlockIdWithPending::BlockId)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value {
                    "pending" => Ok(BlockIdWithPending::Pending),
                    _ => BlockIdVisitor
                        .visit_str(value)
                        .map(BlockIdWithPending::BlockId),
                }
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_str(value.as_ref())
            }
        }

        deserializer.deserialize_any(InnerVisitor)
    }
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Web3Filter {
    pub from_block: Option<BlockId>,
    pub to_block:   Option<BlockId>,
    pub block_hash: Option<H256>,
    #[serde(default)]
    pub address:    MultiType<H160>,
    pub topics:     Option<Vec<H256>>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum MultiType<T> {
    Single(T),
    Multi(Vec<T>),
    Null,
}

impl<T> Default for MultiType<T> {
    fn default() -> Self {
        MultiType::Null
    }
}

impl<T> From<MultiType<T>> for Option<Vec<T>> {
    fn from(src: MultiType<T>) -> Self {
        match src {
            MultiType::Null => None,
            MultiType::Single(i) => Some(vec![i]),
            MultiType::Multi(i) => Some(i),
        }
    }
}

impl<T> Serialize for MultiType<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MultiType::Single(inner) => inner.serialize(serializer),
            MultiType::Multi(inner) => inner.serialize(serializer),
            MultiType::Null => serializer.serialize_none(),
        }
    }
}

impl<'a, T> Deserialize<'a> for MultiType<T>
where
    T: for<'b> Deserialize<'b>,
{
    fn deserialize<D>(deserializer: D) -> Result<MultiType<T>, D::Error>
    where
        D: Deserializer<'a>,
    {
        let v: serde_json::Value = Deserialize::deserialize(deserializer)?;

        if v.is_null() {
            return Ok(MultiType::Null);
        }

        serde_json::from_value(v.clone())
            .map(MultiType::Single)
            .or_else(|_| serde_json::from_value(v).map(MultiType::Multi))
            .map_err(|err| D::Error::custom(format!("Invalid value type: {}", err)))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Web3Log {
    pub address:           H160,
    pub topics:            Vec<H256>,
    pub data:              Hex,
    pub block_hash:        Option<H256>,
    pub block_number:      Option<U256>,
    pub transaction_hash:  Option<H256>,
    pub transaction_index: Option<U256>,
    pub log_index:         Option<U256>,
    #[serde(default)]
    pub removed:           bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Web3SyncStatus {
    Doing(SyncStatus),
    False,
}

impl From<InnerSyncStatus> for Web3SyncStatus {
    fn from(inner: InnerSyncStatus) -> Self {
        match inner {
            InnerSyncStatus::False => Web3SyncStatus::False,
            InnerSyncStatus::Syncing {
                start,
                current,
                highest,
            } => Web3SyncStatus::Doing(SyncStatus {
                starting_block: start,
                current_block:  current,
                highest_block:  highest,
                known_states:   U256::default(),
                pulled_states:  U256::default(),
            }),
        }
    }
}

impl Serialize for Web3SyncStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Web3SyncStatus::Doing(status) => status.serialize(serializer),
            Web3SyncStatus::False => false.serialize(serializer),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub starting_block: U256,
    pub current_block:  U256,
    pub highest_block:  U256,
    pub known_states:   U256,
    pub pulled_states:  U256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Web3FeeHistory {
    pub oldest_block:     U256,
    pub reward:           Option<Vec<U256>>,
    pub base_fee_per_gas: Vec<U256>,
    pub gas_used_ratio:   Vec<U256>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Web3Header {
    pub difficulty:        U256,
    pub extra_data:        Hex,
    pub gas_limit:         U256,
    pub gas_used:          U256,
    pub logs_bloom:        Option<Bloom>,
    pub miner:             H160,
    pub nonce:             U256,
    pub number:            U256,
    pub parent_hash:       H256,
    pub receipts_root:     H256,
    #[serde(rename = "sha3Uncles")]
    pub sha3_uncles:       H256,
    pub state_root:        H256,
    pub timestamp:         U256,
    pub transactions_root: H256,
}

impl From<Header> for Web3Header {
    fn from(h: Header) -> Self {
        Web3Header {
            number:            h.number.into(),
            parent_hash:       h.prev_hash,
            sha3_uncles:       Default::default(),
            logs_bloom:        Some(h.log_bloom),
            transactions_root: h.transactions_root,
            state_root:        h.state_root,
            receipts_root:     h.receipts_root,
            miner:             h.proposer,
            difficulty:        h.difficulty,
            extra_data:        Hex::encode(&h.extra_data),
            gas_limit:         h.gas_limit,
            gas_used:          h.gas_used,
            timestamp:         h.timestamp.into(),
            nonce:             U256::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_json() {
        let status = Web3SyncStatus::False;
        let json = json::parse(&serde_json::to_string(&status).unwrap()).unwrap();
        assert!(json.is_boolean());

        let status = Web3SyncStatus::Doing(SyncStatus {
            starting_block: fastrand::u64(..).into(),
            current_block:  fastrand::u64(..).into(),
            highest_block:  fastrand::u64(..).into(),
            known_states:   U256::default(),
            pulled_states:  U256::default(),
        });
        let json = json::parse(&serde_json::to_string(&status).unwrap()).unwrap();
        assert!(json.is_object());
    }
}
