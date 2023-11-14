use std::{fmt, str::FromStr};

use either::Either;
use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use protocol::codec::ProtocolCodec;
use protocol::types::{
    AccessList, Block, Bloom, Bytes, Hash, Header, Hex, Public, Receipt, SignedTransaction, H160,
    H256, H64, MAX_PRIORITY_FEE_PER_GAS, U256, U64,
};

pub const EMPTY_UNCLE_HASH: H256 = H256([
    0x1d, 0xcc, 0x4d, 0xe8, 0xde, 0xc7, 0x5d, 0x7a, 0xab, 0x85, 0xb5, 0x67, 0xb6, 0xcc, 0xd4, 0x1a,
    0xd3, 0x12, 0x45, 0x1b, 0x94, 0x8a, 0x74, 0x13, 0xf0, 0xa1, 0x42, 0xfd, 0x40, 0xd4, 0x93, 0x47,
]);

use core_consensus::SyncStatus as InnerSyncStatus;

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
    // Quantity for eth sig, hex bytes for interop sig.
    #[serde(with = "either::serde_untagged")]
    pub r:                        Either<U256, Hex>,
    // Quantity for eth sig, hex bytes for interop sig.
    #[serde(with = "either::serde_untagged")]
    pub s:                        Either<U256, Hex>,
}

impl From<SignedTransaction> for Web3Transaction {
    fn from(stx: SignedTransaction) -> Web3Transaction {
        let signature = stx.transaction.signature.clone().unwrap_or_default();
        let is_eip1559 = stx.transaction.unsigned.is_eip1559();

        let sig_v = signature.add_chain_replay_protection(stx.transaction.chain_id);
        let (sig_r, sig_s) = if signature.is_eth_sig() {
            (
                Either::Left(U256::from(&signature.r[..])),
                Either::Left(U256::from(&signature.s[..])),
            )
        } else {
            (
                Either::Right(Hex::encode(signature.r)),
                Either::Right(Hex::encode(signature.s)),
            )
        };

        Web3Transaction {
            type_:                    Some(stx.type_().into()),
            block_number:             None,
            block_hash:               None,
            raw:                      Hex::encode(stx.transaction.encode().unwrap()),
            public_key:               stx.public,
            gas:                      *stx.transaction.unsigned.gas_limit(),
            gas_price:                stx.transaction.unsigned.gas_price(),
            max_fee_per_gas:          if is_eip1559 {
                Some(U256::from(MAX_PRIORITY_FEE_PER_GAS))
            } else {
                None
            },
            max_priority_fee_per_gas: if is_eip1559 {
                Some(*stx.transaction.unsigned.max_priority_fee_per_gas())
            } else {
                None
            },
            hash:                     stx.transaction.hash,
            from:                     stx.sender,
            to:                       stx.get_to(),
            input:                    Hex::encode(stx.transaction.unsigned.data()),
            nonce:                    *stx.transaction.unsigned.nonce(),
            transaction_index:        None,
            value:                    *stx.transaction.unsigned.value(),
            access_list:              Some(stx.transaction.unsigned.access_list()),
            chain_id:                 stx.transaction.chain_id.map(|id| id.into()),
            standard_v:               None,
            v:                        sig_v.into(),
            r:                        sig_r,
            s:                        sig_s,
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

    pub fn update_with_receipt(&mut self, receipt: &Receipt) {
        self.block_number = Some(receipt.block_number.into());
        self.block_hash = Some(receipt.block_hash);
        self.transaction_index = Some(receipt.tx_index.into());
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
            .enumerate()
            .map(|(idx, log)| Web3ReceiptLog {
                address:           log.address,
                topics:            log.topics.clone(),
                data:              Hex::encode(&log.data),
                block_number:      receipt.block_number.into(),
                block_hash:        receipt.block_hash,
                transaction_hash:  receipt.tx_hash,
                transaction_index: Some(receipt.tx_index.into()),
                log_index:         idx.into(),
                removed:           false,
            })
            .collect::<Vec<_>>();

        Web3Receipt {
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
            transaction_type:    Some(stx.type_().into()),
        }
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
    pub nonce:             H64,
}

impl From<Block> for Web3Block {
    fn from(b: Block) -> Self {
        Web3Block {
            hash:              b.hash(),
            number:            b.header.number.into(),
            author:            b.header.proposer,
            parent_hash:       b.header.prev_hash,
            sha3_uncles:       EMPTY_UNCLE_HASH,
            logs_bloom:        Some(b.header.log_bloom),
            transactions_root: b.header.transactions_root,
            state_root:        b.header.state_root,
            receipts_root:     b.header.receipts_root,
            miner:             b.header.proposer,
            difficulty:        U256::one(),
            total_difficulty:  Some(b.header.number.into()),
            seal_fields:       vec![],
            base_fee_per_gas:  b.header.base_fee_per_gas,
            extra_data:        Hex::encode(rlp::encode_list(&b.header.extra_data)),
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
            nonce:             H64::default(),
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
    pub data:                     Option<Hex>,
    pub nonce:                    Option<U256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list:              Option<AccessList>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockId {
    Num(U64),
    Hash(H256),
    #[default]
    Latest,
    Earliest,
    Pending,
}

impl From<BlockId> for Option<u64> {
    fn from(id: BlockId) -> Self {
        match id {
            // The BlockId deserialize visitor will ensure that the number is in u64 range.
            BlockId::Num(num) => Some(num.low_u64()),
            BlockId::Earliest => Some(0),
            _ => None,
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
            BlockId::Num(ref x) => serializer.serialize_str(&format!("{:x}", x)),
            BlockId::Hash(ref hash) => serializer.serialize_str(&format!("{:x}", hash)),
            BlockId::Latest => serializer.serialize_str("latest"),
            BlockId::Earliest => serializer.serialize_str("earliest"),
            BlockId::Pending => serializer.serialize_str("pending"),
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
        let mut block_hash = None;

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

                            block_number = Some(U64::from(number));
                            break;
                        } else {
                            return Err(Error::custom(
                                "Invalid block number: missing 0x prefix".to_string(),
                            ));
                        }
                    }
                    "blockHash" => {
                        let value: String = visitor.next_value()?;
                        let raw_value = Hex::from_str(&value)
                            .map_err(|e| Error::custom(format!("Invalid hex code: {}", e)))?;
                        if raw_value.len() != 32 {
                            return Err(Error::custom(format!("Invalid block hash: {}", value)));
                        } else {
                            let mut v = [0u8; 32];
                            v.copy_from_slice(raw_value.as_ref());
                            block_hash = Some(v.into());
                            break;
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

        if let Some(hash) = block_hash {
            return Ok(BlockId::Hash(hash));
        }

        Err(Error::custom("Invalid input"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match value {
            "latest" => Ok(BlockId::Latest),
            "earliest" => Ok(BlockId::Earliest),
            "pending" => Ok(BlockId::Pending),
            _ if value.starts_with("0x") => u64::from_str_radix(&value[2..], 16)
                .map(|n| BlockId::Num(U64::from(n)))
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
    pub topics:     Option<Vec<MultiNestType<Hash>>>,
}

#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub enum MultiNestType<T> {
    Single(T),
    Multi(Vec<Option<T>>),
    #[default]
    Null,
}

impl<T> From<MultiNestType<T>> for Option<Vec<Option<T>>> {
    fn from(src: MultiNestType<T>) -> Self {
        match src {
            MultiNestType::Null => None,
            MultiNestType::Single(i) => Some(vec![Some(i)]),
            MultiNestType::Multi(i) => Some(i),
        }
    }
}

impl<T> Serialize for MultiNestType<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MultiNestType::Single(inner) => inner.serialize(serializer),
            MultiNestType::Multi(inner) => inner.serialize(serializer),
            MultiNestType::Null => serializer.serialize_none(),
        }
    }
}

impl<'a, T> Deserialize<'a> for MultiNestType<T>
where
    T: for<'b> Deserialize<'b>,
{
    fn deserialize<D>(deserializer: D) -> Result<MultiNestType<T>, D::Error>
    where
        D: Deserializer<'a>,
    {
        let v: serde_json::Value = Deserialize::deserialize(deserializer)?;

        if v.is_null() {
            return Ok(MultiNestType::Null);
        }

        serde_json::from_value(v.clone())
            .map(MultiNestType::Single)
            .or_else(|_| serde_json::from_value(v).map(MultiNestType::Multi))
            .map_err(|err| D::Error::custom(format!("Invalid value type: {}", err)))
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub enum MultiType<T> {
    Single(T),
    Multi(Vec<T>),
    #[default]
    Null,
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

/// Response type for `eth_feeHistory` RPC call.
/// Three types of response are possible:
/// 1. With reward, it returned when request parameter REWARDPERCENTILES is not
/// null: `{"oldestBlock": "0x...", "reward": [["0x...", ...], ...],
/// "baseFeePerGas": ["0x...", ...], "gasUsedRatio": [0.0, ...]}` 2. Without
/// reward, it returned when request parameter REWARDPERCENTILES is null:
/// `{"oldestBlock": "0x...", "baseFeePerGas": ["0x...", ...], "gasUsedRatio":
/// [0.0, ...]}` 3. Zero block count, it returned when request parameter
/// BLOCKCOUNT is 0: `{"oldestBlock": "0x...", "baseFeePerGas": [],
/// "gasUsedRatio": []}` See https://docs.infura.io/infura/networks/ethereum/json-rpc-methods/eth_feehistory
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Web3FeeHistory {
    WithReward(FeeHistoryWithReward),
    WithoutReward(FeeHistoryWithoutReward),
    ZeroBlockCount(FeeHistoryEmpty),
}

/// Response type for `eth_feeHistory` RPC call with parameter REWARDPERCENTILES
/// is not null.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeeHistoryWithReward {
    /// Lowest number block of the returned range.
    pub oldest_block:     U256,
    /// An array of block base fees per gas.
    /// This includes the next block after the newest of the returned range,
    /// because this value can be derived from the newest block. Zeroes are
    /// returned for pre-EIP-1559 blocks.
    pub base_fee_per_gas: Vec<U256>,
    /// An array of block gas used ratios. These are calculated as the ratio
    /// of `gasUsed` and `gasLimit`.
    pub gas_used_ratio:   Vec<f64>,
    /// An (optional) array of effective priority fee per gas data points from a
    /// single block. All zeroes are returned if the block is empty.
    pub reward:           Vec<Vec<U256>>,
}

/// Response type for `eth_feeHistory` RPC call with parameter REWARDPERCENTILES
/// is null.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeeHistoryWithoutReward {
    /// Lowest number block of the returned range.
    pub oldest_block:     U256,
    /// An array of block base fees per gas.
    /// This includes the next block after the newest of the returned range,
    /// because this value can be derived from the newest block. Zeroes are
    /// returned for pre-EIP-1559 blocks.
    pub base_fee_per_gas: Vec<U256>,
    /// An array of block gas used ratios. These are calculated as the ratio
    /// of `gasUsed` and `gasLimit`.
    pub gas_used_ratio:   Vec<f64>,
}

/// Response type for `eth_feeHistory` RPC call with parameter BLOCKCOUNT is 0.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeeHistoryEmpty {
    /// Lowest number block of the returned range.
    pub oldest_block:   U256,
    /// An array of block gas used ratios. These are calculated as the ratio
    /// of `gasUsed` and `gasLimit`.
    pub gas_used_ratio: Option<Vec<f64>>,
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
            sha3_uncles:       EMPTY_UNCLE_HASH,
            logs_bloom:        Some(h.log_bloom),
            transactions_root: h.transactions_root,
            state_root:        h.state_root,
            receipts_root:     h.receipts_root,
            miner:             h.proposer,
            difficulty:        U256::one(),
            extra_data:        Hex::encode(rlp::encode_list(&h.extra_data)),
            gas_limit:         h.gas_limit,
            gas_used:          h.gas_used,
            timestamp:         h.timestamp.into(),
            nonce:             U256::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase", untagged)]
pub enum FilterChanges {
    Blocks(Vec<H256>),
    Logs(Vec<Web3Log>),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RawLoggerFilter {
    pub from_block: Option<BlockId>,
    pub to_block:   Option<BlockId>,
    #[serde(default)]
    pub address:    MultiType<H160>,
    pub topics:     Option<Vec<MultiNestType<Hash>>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum HardforkStatus {
    Proposed,
    Determined,
    Enabled,
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::{rand::random, types::UnverifiedTransaction};

    #[test]
    fn test_sync_status_json() {
        let status = Web3SyncStatus::False;
        let json = json::parse(&serde_json::to_string(&status).unwrap()).unwrap();
        assert!(json.is_boolean());

        let status = Web3SyncStatus::Doing(SyncStatus {
            starting_block: random::<u64>().into(),
            current_block:  random::<u64>().into(),
            highest_block:  random::<u64>().into(),
            known_states:   U256::default(),
            pulled_states:  U256::default(),
        });
        let json = json::parse(&serde_json::to_string(&status).unwrap()).unwrap();
        assert!(json.is_object());
    }

    // Test json serialization of web3 transactions, esp. that r/s don't have
    // leading zeros.
    #[test]
    fn test_web3_transaction_json() {
        // https://etherscan.io/getRawTx?tx=0x07c7388b03ab8403deeaefc551efbc632f8531f04dc9993a274dbba9bbb98cbf
        let tx = Hex::from_str("0x02f902f801728405f5e1008509898edcf78302ffb8943fc91a3afd70395cd496c647d5a6cc9d4b2b7fad8802c68af0bb140000b902843593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006480c64700000000000000000000000000000000000000000000000000000000000000020b080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000002c68af0bb1400000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000002c68af0bb1400000000000000000000000000000000000000000004a715ce36374beaa635218d9700000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000c3681a720605bd6f8fe9a2fabff6a7cdecdc605dc080a0d253ee687ab2d9734a5073d64a0ba26bc3bc1cf4582005137bba05ef88616ea89e8ba79925267b17403fdf3ab47641b4aa52322dc385429cc92a7003c5d7c2").unwrap();
        let tx = UnverifiedTransaction::decode(tx).unwrap();
        let tx = SignedTransaction::from_unverified(tx).unwrap();
        let tx_json = serde_json::to_value(Web3Transaction::from(tx)).unwrap();

        assert_eq!(
            tx_json["r"],
            "0xd253ee687ab2d9734a5073d64a0ba26bc3bc1cf4582005137bba05ef88616ea8"
        );
        assert_eq!(
            tx_json["s"],
            "0x8ba79925267b17403fdf3ab47641b4aa52322dc385429cc92a7003c5d7c2"
        );
        assert_eq!(tx_json["v"], "0x25");
    }
}
