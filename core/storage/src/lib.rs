#![feature(test)]
#![allow(clippy::mutable_key_type)]

#[cfg(test)]
mod tests;

pub mod adapter;

use std::collections::{HashMap, HashSet};
use std::convert::From;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use arc_swap::ArcSwap;

use common_apm::metrics::storage::on_storage_get_cf;
// use common_apm::muta_apm;

use protocol::codec::ProtocolCodec;
use protocol::traits::{
    CommonStorage, Context, Storage, StorageAdapter, StorageBatchModify, StorageCategory,
    StorageSchema,
};
use protocol::types::{
    Block, BlockNumber, Bytes, DBBytes, Hash, Hasher, Header, Proof, Receipt, SignedTransaction,
    H256,
};
use protocol::{
    async_trait, tokio, Display, From, ProtocolError, ProtocolErrorKind, ProtocolResult,
};

const BATCH_VALUE_DECODE_NUMBER: usize = 1000;

lazy_static::lazy_static! {
    pub static ref LATEST_BLOCK_KEY: Hash = Hasher::digest(Bytes::from("latest_hash"));
    pub static ref LATEST_PROOF_KEY: Hash = Hasher::digest(Bytes::from("latest_proof"));
    pub static ref OVERLORD_WAL_KEY: Hash = Hasher::digest(Bytes::from("overlord_wal"));
}

macro_rules! get {
    ($self_: ident, $key: expr, $schema: ident) => {{
        $self_.adapter.get::<$schema>($key).await
    }};
}

macro_rules! ensure_get {
    ($self_: ident, $key: expr, $schema: ident) => {{
        let opt = get!($self_, $key, $schema)?;
        opt.ok_or_else(|| StorageError::GetNone)?
    }};
}

macro_rules! impl_storage_schema_for {
    ($name: ident, $key: ident, $val: ident, $category: ident) => {
        pub struct $name;

        impl StorageSchema for $name {
            type Key = $key;
            type Value = $val;

            fn category() -> StorageCategory {
                StorageCategory::$category
            }
        }
    };
}

#[derive(Debug)]
pub struct ImplStorage<Adapter> {
    adapter:      Arc<Adapter>,
    latest_block: ArcSwap<Option<Block>>,
}

impl<Adapter: StorageAdapter> ImplStorage<Adapter> {
    pub fn new(adapter: Arc<Adapter>) -> Self {
        Self {
            adapter,
            latest_block: ArcSwap::from(Arc::new(None)),
        }
    }

    async fn batch_insert_stxs(
        &self,
        stxs: Vec<SignedTransaction>,
        block_number: BlockNumber,
    ) -> ProtocolResult<()> {
        let (hashes, heights) = stxs
            .iter()
            .map(|item| {
                (
                    item.transaction.hash,
                    StorageBatchModify::Insert(block_number),
                )
            })
            .unzip();

        let (keys, batch_stxs): (Vec<_>, Vec<_>) = stxs
            .into_iter()
            .map(|item| {
                (
                    CommonHashKey::new(block_number, item.transaction.hash),
                    StorageBatchModify::Insert(item),
                )
            })
            .unzip();

        self.adapter
            .batch_modify::<TransactionSchema>(keys, batch_stxs)
            .await?;

        self.adapter
            .batch_modify::<TxHashNumberSchema>(hashes, heights)
            .await?;

        Ok(())
    }

    async fn batch_insert_receipts(
        &self,
        receipts: Vec<Receipt>,
        block_number: BlockNumber,
    ) -> ProtocolResult<()> {
        let (hashes, heights) = receipts
            .iter()
            .map(|item| (item.tx_hash, StorageBatchModify::Insert(block_number)))
            .unzip();

        let (keys, batch_stxs): (Vec<_>, Vec<_>) = receipts
            .into_iter()
            .map(|item| {
                (
                    CommonHashKey::new(block_number, item.tx_hash),
                    StorageBatchModify::Insert(item),
                )
            })
            .unzip();

        self.adapter
            .batch_modify::<ReceiptSchema>(keys, batch_stxs)
            .await?;

        self.adapter
            .batch_modify::<TxHashNumberSchema>(hashes, heights)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommonPrefix {
    block_height: [u8; 8], // BigEndian
}

impl CommonPrefix {
    pub fn new(block_height: u64) -> Self {
        CommonPrefix {
            block_height: block_height.to_be_bytes(),
        }
    }

    pub fn len() -> usize {
        8
    }

    pub fn height(self) -> u64 {
        u64::from_be_bytes(self.block_height)
    }

    pub fn make_hash_key(self, hash: &Hash) -> [u8; 40] {
        debug_assert!(hash.as_bytes().len() == 32);

        let mut key = [0u8; 40];
        key[0..8].copy_from_slice(&self.block_height);
        key[8..40].copy_from_slice(&hash.as_bytes()[..32]);

        key
    }
}

impl AsRef<[u8]> for CommonPrefix {
    fn as_ref(&self) -> &[u8] {
        &self.block_height
    }
}

impl From<&[u8]> for CommonPrefix {
    fn from(bytes: &[u8]) -> CommonPrefix {
        debug_assert!(bytes.len() >= 8);

        let mut h_buf = [0u8; 8];
        h_buf.copy_from_slice(&bytes[0..8]);

        CommonPrefix {
            block_height: h_buf,
        }
    }
}

impl ProtocolCodec for CommonPrefix {
    fn encode(&self) -> ProtocolResult<Bytes> {
        Ok(Bytes::copy_from_slice(&self.block_height))
    }

    fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self> {
        Ok(CommonPrefix::from(&bytes.as_ref()[..8]))
    }
}

#[derive(Debug, Clone)]
pub struct CommonHashKey {
    prefix: CommonPrefix,
    hash:   Hash,
}

impl CommonHashKey {
    pub fn new(block_height: u64, hash: Hash) -> Self {
        CommonHashKey {
            prefix: CommonPrefix::new(block_height),
            hash,
        }
    }

    pub fn height(&self) -> u64 {
        self.prefix.height()
    }

    pub fn hash(&self) -> &Hash {
        &self.hash
    }
}

impl ProtocolCodec for CommonHashKey {
    fn encode(&self) -> ProtocolResult<Bytes> {
        Ok(Bytes::copy_from_slice(
            &self.prefix.make_hash_key(&self.hash),
        ))
    }

    fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self> {
        let mut bytes = bytes.as_ref().to_vec();
        debug_assert!(bytes.len() >= CommonPrefix::len());

        let prefix = CommonPrefix::from(&bytes[0..CommonPrefix::len()]);
        let hash = Hash::from_slice(&bytes.split_off(CommonPrefix::len()));

        Ok(CommonHashKey { prefix, hash })
    }
}

impl ToString for CommonHashKey {
    fn to_string(&self) -> String {
        format!("{}:{}", self.prefix.height(), self.hash)
    }
}

impl FromStr for CommonHashKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(':').collect::<Vec<_>>();
        debug_assert!(parts.len() == 2);

        let height = parts[0].parse::<u64>().map_err(|_| ())?;

        let hash = Hasher::digest(parts[1].as_bytes());

        Ok(CommonHashKey::new(height, hash))
    }
}

pub type BlockKey = CommonPrefix;

impl_storage_schema_for!(
    TransactionSchema,
    CommonHashKey,
    SignedTransaction,
    SignedTransaction
);
impl_storage_schema_for!(
    TransactionBytesSchema,
    CommonHashKey,
    DBBytes,
    SignedTransaction
);
impl_storage_schema_for!(BlockSchema, BlockKey, Block, Block);
impl_storage_schema_for!(BlockHeaderSchema, BlockKey, Header, BlockHeader);
impl_storage_schema_for!(BlockHashNumberSchema, Hash, u64, HashHeight);
impl_storage_schema_for!(ReceiptSchema, CommonHashKey, Receipt, Receipt);
impl_storage_schema_for!(ReceiptBytesSchema, CommonHashKey, DBBytes, Receipt);
impl_storage_schema_for!(TxHashNumberSchema, Hash, u64, HashHeight);
impl_storage_schema_for!(LatestBlockSchema, Hash, Block, Block);
impl_storage_schema_for!(LatestProofSchema, Hash, Proof, Block);
impl_storage_schema_for!(OverlordWalSchema, Hash, Bytes, Wal);
impl_storage_schema_for!(EvmCodeSchema, Hash, Bytes, Code);
impl_storage_schema_for!(EvmCodeAddressSchema, Hash, Hash, Code);

#[async_trait]
impl<Adapter: StorageAdapter> CommonStorage for ImplStorage<Adapter> {
    // #[muta_apm::derive::tracing_span(kind = "storage")]
    async fn insert_block(&self, ctx: Context, block: Block) -> ProtocolResult<()> {
        self.set_block(ctx.clone(), block.clone()).await?;

        self.set_latest_block(ctx, block).await?;

        Ok(())
    }

    async fn get_block(&self, _ctx: Context, height: u64) -> ProtocolResult<Option<Block>> {
        self.adapter.get::<BlockSchema>(BlockKey::new(height)).await
    }

    async fn get_block_header(&self, ctx: Context, height: u64) -> ProtocolResult<Option<Header>> {
        let opt_header = self
            .adapter
            .get::<BlockHeaderSchema>(BlockKey::new(height))
            .await?;
        if opt_header.is_some() {
            return Ok(opt_header);
        }

        Ok(self.get_block(ctx, height).await?.map(|b| b.header))
    }

    async fn set_block(&self, _ctx: Context, block: Block) -> ProtocolResult<()> {
        self.adapter
            .insert::<BlockSchema>(BlockKey::new(block.header.number), block.clone())
            .await?;
        self.adapter
            .insert::<BlockHeaderSchema>(BlockKey::new(block.header.number), block.header.clone())
            .await?;
        self.adapter
            .insert::<BlockHashNumberSchema>(block.header_hash(), block.header.number)
            .await?;
        Ok(())
    }

    async fn remove_block(&self, _ctx: Context, height: u64) -> ProtocolResult<()> {
        self.adapter
            .remove::<BlockSchema>(BlockKey::new(height))
            .await
    }

    async fn get_latest_block(&self, _ctx: Context) -> ProtocolResult<Block> {
        if let Some(block) = self.latest_block.load().as_ref().clone() {
            Ok(block)
        } else {
            let block = ensure_get!(self, *LATEST_BLOCK_KEY, LatestBlockSchema);
            Ok(block)
        }
    }

    async fn set_latest_block(&self, _ctx: Context, block: Block) -> ProtocolResult<()> {
        self.adapter
            .insert::<LatestBlockSchema>(*LATEST_BLOCK_KEY, block.clone())
            .await?;

        self.latest_block.store(Arc::new(Some(block)));

        Ok(())
    }

    async fn get_latest_block_header(&self, _ctx: Context) -> ProtocolResult<Header> {
        let opt_header = {
            let guard = self.latest_block.load();
            let opt_block = guard.as_ref();
            opt_block.as_ref().map(|b| b.header.clone())
        };

        if let Some(header) = opt_header {
            Ok(header)
        } else {
            let block = ensure_get!(self, *LATEST_BLOCK_KEY, LatestBlockSchema);
            Ok(block.header)
        }
    }
}

#[async_trait]
impl<Adapter: StorageAdapter> Storage for ImplStorage<Adapter> {
    // #[muta_apm::derive::tracing_span(kind = "storage")]
    async fn insert_transactions(
        &self,
        _ctx: Context,
        block_height: u64,
        signed_txs: Vec<SignedTransaction>,
    ) -> ProtocolResult<()> {
        self.batch_insert_stxs(signed_txs, block_height).await?;

        Ok(())
    }

    async fn get_block_by_hash(
        &self,
        ctx: Context,
        block_hash: &Hash,
    ) -> ProtocolResult<Option<Block>> {
        let block_number = self
            .adapter
            .get::<BlockHashNumberSchema>(*block_hash)
            .await?;

        if let Some(num) = block_number {
            return self.get_block(ctx, num).await;
        }

        Ok(None)
    }

    // #[muta_apm::derive::tracing_span(kind = "storage")]
    async fn get_transactions(
        &self,
        _ctx: Context,
        block_height: u64,
        hashes: &[Hash],
    ) -> ProtocolResult<Vec<Option<SignedTransaction>>> {
        let key_prefix = CommonPrefix::new(block_height);
        let mut found = Vec::with_capacity(hashes.len());

        {
            let inst = Instant::now();
            let prepare_iter = self
                .adapter
                .prepare_iter::<TransactionBytesSchema, _>(&key_prefix)?;
            let mut iter = prepare_iter.ref_to_iter();

            let set = hashes.iter().collect::<HashSet<_>>();
            let mut count = hashes.len();
            on_storage_get_cf(
                StorageCategory::SignedTransaction,
                common_apm::elapsed(inst),
                count as f64,
            );

            while count > 0 {
                let (key, stx_bytes) = match iter.next() {
                    None => break,
                    Some(Ok(key_to_stx_bytes)) => key_to_stx_bytes,
                    Some(Err(err)) => return Err(err),
                };

                // Note: fix clippy::suspicious_else_formatting
                if key.height() != block_height {
                    break;
                } else if !set.contains(&key.hash) {
                    continue;
                } else {
                    found.push((key.hash, stx_bytes));
                    count -= 1;
                }
            }
        }

        let mut found = {
            if found.len() <= BATCH_VALUE_DECODE_NUMBER {
                found
                    .drain(..)
                    .map(|(k, v): (Hash, DBBytes)| SignedTransaction::decode(v).map(|v| (k, v)))
                    .collect::<ProtocolResult<Vec<_>>>()?
                    .into_iter()
                    .collect::<HashMap<_, _>>()
            } else {
                let futs = found
                    .chunks(BATCH_VALUE_DECODE_NUMBER)
                    .map(|vals| {
                        let vals = vals.to_owned();

                        // FIXME: cancel decode
                        tokio::spawn(async move {
                            vals.into_iter()
                                .map(|(k, v): (Hash, DBBytes)| <_>::decode(v).map(|v| (k, v)))
                                .collect::<ProtocolResult<Vec<_>>>()
                        })
                    })
                    .collect::<Vec<_>>();

                futures::future::try_join_all(futs)
                    .await
                    .map_err(|_| StorageError::BatchDecode)?
                    .into_iter()
                    .collect::<ProtocolResult<Vec<Vec<_>>>>()?
                    .into_iter()
                    .flatten()
                    .collect::<HashMap<_, _>>()
            }
        };

        Ok(hashes.iter().map(|h| found.remove(h)).collect::<Vec<_>>())
    }

    async fn insert_code(
        &self,
        _ctx: Context,
        code_address: H256,
        code_hash: Hash,
        code: Bytes,
    ) -> ProtocolResult<()> {
        self.adapter
            .insert::<EvmCodeSchema>(code_hash, code)
            .await?;
        self.adapter
            .insert::<EvmCodeAddressSchema>(code_address, code_hash)
            .await
    }

    async fn get_code_by_hash(&self, _ctx: Context, hash: &Hash) -> ProtocolResult<Option<Bytes>> {
        self.adapter.get::<EvmCodeSchema>(*hash).await
    }

    async fn get_code_by_address(
        &self,
        ctx: Context,
        address: &H256,
    ) -> ProtocolResult<Option<Bytes>> {
        let code_hash = self.adapter.get::<EvmCodeAddressSchema>(*address).await?;

        if let Some(hash) = code_hash {
            self.get_code_by_hash(ctx, &hash).await
        } else {
            Ok(None)
        }
    }

    async fn get_transaction_by_hash(
        &self,
        _ctx: Context,
        hash: &Hash,
    ) -> ProtocolResult<Option<SignedTransaction>> {
        if let Some(block_height) = get!(self, *hash, TxHashNumberSchema)? {
            get!(
                self,
                CommonHashKey::new(block_height, *hash),
                TransactionSchema
            )
        } else {
            Ok(None)
        }
    }

    // #[muta_apm::derive::tracing_span(kind = "storage")]
    async fn insert_receipts(
        &self,
        _ctx: Context,
        block_height: u64,
        receipts: Vec<Receipt>,
    ) -> ProtocolResult<()> {
        self.batch_insert_receipts(receipts, block_height).await?;

        Ok(())
    }

    async fn get_receipt_by_hash(
        &self,
        _ctx: Context,
        hash: Hash,
    ) -> ProtocolResult<Option<Receipt>> {
        if let Some(block_height) = get!(self, hash, TxHashNumberSchema)? {
            get!(self, CommonHashKey::new(block_height, hash), ReceiptSchema)
        } else {
            Ok(None)
        }
    }

    // #[muta_apm::derive::tracing_span(kind = "storage")]
    async fn get_receipts(
        &self,
        _ctx: Context,
        block_height: u64,
        hashes: &[Hash],
    ) -> ProtocolResult<Vec<Option<Receipt>>> {
        let key_prefix = CommonPrefix::new(block_height);
        let mut found = Vec::with_capacity(hashes.len());

        {
            let inst = Instant::now();
            let prepare_iter = self
                .adapter
                .prepare_iter::<ReceiptBytesSchema, _>(&key_prefix)?;
            let mut iter = prepare_iter.ref_to_iter();

            let set = hashes.iter().collect::<HashSet<_>>();
            let mut count = hashes.len();
            on_storage_get_cf(
                StorageCategory::Receipt,
                common_apm::elapsed(inst),
                count as f64,
            );

            while count > 0 {
                let (key, stx_bytes) = match iter.next() {
                    None => break,
                    Some(Ok(key_to_stx_bytes)) => key_to_stx_bytes,
                    Some(Err(err)) => return Err(err),
                };

                // Note: fix clippy::suspicious_else_formatting
                if key.height() != block_height {
                    break;
                } else if !set.contains(&key.hash) {
                    continue;
                } else {
                    found.push((key.hash, stx_bytes));
                    count -= 1;
                }
            }
        }

        let mut found = {
            if found.len() <= BATCH_VALUE_DECODE_NUMBER {
                found
                    .drain(..)
                    .map(|(k, v): (Hash, DBBytes)| Receipt::decode(v).map(|v| (k, v)))
                    .collect::<ProtocolResult<Vec<_>>>()?
                    .into_iter()
                    .collect::<HashMap<_, _>>()
            } else {
                let futs = found
                    .chunks(BATCH_VALUE_DECODE_NUMBER)
                    .map(|vals| {
                        let vals = vals.to_owned();

                        // FIXME: cancel decode
                        tokio::spawn(async move {
                            vals.into_iter()
                                .map(|(k, v): (Hash, DBBytes)| <_>::decode(v).map(|v| (k, v)))
                                .collect::<ProtocolResult<Vec<_>>>()
                        })
                    })
                    .collect::<Vec<_>>();

                futures::future::try_join_all(futs)
                    .await
                    .map_err(|_| StorageError::BatchDecode)?
                    .into_iter()
                    .collect::<ProtocolResult<Vec<Vec<_>>>>()?
                    .into_iter()
                    .flatten()
                    .collect::<HashMap<_, _>>()
            }
        };

        Ok(hashes.iter().map(|h| found.remove(h)).collect::<Vec<_>>())
    }

    async fn update_latest_proof(&self, _ctx: Context, proof: Proof) -> ProtocolResult<()> {
        self.adapter
            .insert::<LatestProofSchema>(*LATEST_PROOF_KEY, proof)
            .await?;
        Ok(())
    }

    async fn get_latest_proof(&self, _ctx: Context) -> ProtocolResult<Proof> {
        let proof = ensure_get!(self, *LATEST_PROOF_KEY, LatestProofSchema);
        Ok(proof)
    }
}

#[derive(Debug, Display, From)]
pub enum StorageError {
    #[display(fmt = "get none")]
    GetNone,

    #[display(fmt = "decode batch value")]
    BatchDecode,
}

impl Error for StorageError {}

impl From<StorageError> for ProtocolError {
    fn from(err: StorageError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Storage, Box::new(err))
    }
}
