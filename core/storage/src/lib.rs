#![allow(clippy::mutable_key_type, clippy::uninlined_format_args)]

pub mod adapter;
mod cache;
mod hash_key;
mod schema;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::convert::From;
use std::error::Error;
use std::sync::Arc;

use arc_swap::ArcSwap;

use common_apm::metrics::storage::on_storage_get_cf;
use common_apm::Instant;
use common_apm_derive::trace_span;
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

use crate::cache::StorageCache;
use crate::hash_key::{BlockKey, CommonHashKey, CommonPrefix};
use crate::schema::{
    BlockHashNumberSchema, BlockHeaderSchema, BlockSchema, EvmCodeAddressSchema, EvmCodeSchema,
    LatestBlockSchema, LatestProofSchema, ReceiptBytesSchema, ReceiptSchema,
    TransactionBytesSchema, TransactionSchema, TxHashNumberSchema,
};

const BATCH_VALUE_DECODE_NUMBER: usize = 1000;

lazy_static::lazy_static! {
    pub static ref LATEST_BLOCK_KEY: Hash = Hasher::digest(Bytes::from("latest_hash"));
    pub static ref LATEST_PROOF_KEY: Hash = Hasher::digest(Bytes::from("latest_proof"));
    pub static ref OVERLORD_WAL_KEY: Hash = Hasher::digest(Bytes::from("overlord_wal"));
    pub static ref MONITOR_CKB_NUMBER_KEY: Hash = Hasher::digest(Bytes::from("monitor_ckb_number"));
}

macro_rules! get_cache {
    ($self_: ident, $key: expr, $category: ident) => {{
        let mut cache = $self_.cache.$category.lock();
        if let Some(ret) = cache.get($key).cloned() {
            return Ok(Some(ret));
        }
    }};
}

macro_rules! put_cache {
    ($self_: ident, $key: expr, $val: expr, $category: ident) => {{
        if let Some(val) = $val.clone() {
            let mut cache = $self_.cache.$category.lock();
            let _ = cache.put($key.clone(), val);
        }
    }};
}

macro_rules! get {
    ($self_: ident, $key: expr, $schema: ident) => {{
        let inst = Instant::now();
        let res = $self_.adapter.get::<$schema>($key);
        on_storage_get_cf($schema::category(), inst.elapsed(), 1.0f64);
        res
    }};

    ($self_: ident, $key: expr, $schema: ident, $cache_key: expr, $category: ident) => {{
        let inst = Instant::now();
        let res = $self_.adapter.get::<$schema>($key)?;
        put_cache!($self_, $cache_key, res, $category);
        on_storage_get_cf($schema::category(), inst.elapsed(), 1.0f64);
        Ok(res)
    }};
}

macro_rules! ensure_get {
    ($self_: ident, $key: expr, $schema: ident) => {{
        let opt = get!($self_, $key, $schema)?;
        opt.ok_or_else(|| StorageError::GetNone($key.to_string()))?
    }};
}

#[derive(Debug)]
pub struct ImplStorage<Adapter> {
    adapter:      Arc<Adapter>,
    cache:        Arc<StorageCache>,
    latest_block: ArcSwap<Option<Block>>,
    latest_proof: ArcSwap<Option<Proof>>,
}

impl<Adapter: StorageAdapter> ImplStorage<Adapter> {
    pub fn new(adapter: Arc<Adapter>, cache_size: usize) -> Self {
        Self {
            adapter,
            cache: Arc::new(StorageCache::new(cache_size)),
            latest_block: ArcSwap::new(Arc::new(None)),
            latest_proof: ArcSwap::new(Arc::new(None)),
        }
    }

    async fn get_block_number_by_hash(&self, hash: &Hash) -> ProtocolResult<Option<u64>> {
        get_cache!(self, hash, block_numbers);
        let ret = self.adapter.get::<BlockHashNumberSchema>(*hash)?;
        put_cache!(self, hash, ret, block_numbers);
        Ok(ret)
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
            .batch_modify::<TransactionSchema>(keys, batch_stxs)?;

        self.adapter
            .batch_modify::<TxHashNumberSchema>(hashes, heights)?;

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
            .batch_modify::<ReceiptSchema>(keys, batch_stxs)?;

        self.adapter
            .batch_modify::<TxHashNumberSchema>(hashes, heights)?;

        Ok(())
    }
}

#[async_trait]
impl<Adapter: StorageAdapter> CommonStorage for ImplStorage<Adapter> {
    #[trace_span(kind = "storage")]
    async fn insert_block(&self, ctx: Context, block: Block) -> ProtocolResult<()> {
        self.set_block(ctx.clone(), block.clone()).await?;

        self.set_latest_block(ctx, block).await?;

        Ok(())
    }

    async fn get_block(&self, _ctx: Context, height: u64) -> ProtocolResult<Option<Block>> {
        get_cache!(self, &height, blocks);
        let ret = self.adapter.get::<BlockSchema>(BlockKey::new(height))?;
        put_cache!(self, height, ret, blocks);
        Ok(ret)
    }

    async fn get_block_header(&self, ctx: Context, height: u64) -> ProtocolResult<Option<Header>> {
        get_cache!(self, &height, headers);
        let opt_header = self
            .adapter
            .get::<BlockHeaderSchema>(BlockKey::new(height))?;
        if opt_header.is_some() {
            put_cache!(self, height, opt_header, headers);
            return Ok(opt_header);
        }

        Ok(self.get_block(ctx, height).await?.map(|b| b.header))
    }

    async fn set_block(&self, _ctx: Context, block: Block) -> ProtocolResult<()> {
        self.adapter
            .insert::<BlockSchema>(BlockKey::new(block.header.number), block.clone())?;
        self.adapter.insert::<BlockHeaderSchema>(
            BlockKey::new(block.header.number),
            block.header.clone(),
        )?;
        self.adapter
            .insert::<BlockHashNumberSchema>(block.hash(), block.header.number)?;
        Ok(())
    }

    async fn remove_block(&self, _ctx: Context, height: u64) -> ProtocolResult<()> {
        self.adapter.remove::<BlockSchema>(BlockKey::new(height))
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
            .insert::<LatestBlockSchema>(*LATEST_BLOCK_KEY, block.clone())?;

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
    #[trace_span(kind = "storage")]
    async fn insert_transactions(
        &self,
        ctx: Context,
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
        if let Some(num) = self.get_block_number_by_hash(block_hash).await? {
            return self.get_block(ctx, num).await;
        }

        Ok(None)
    }

    async fn get_block_header_by_hash(
        &self,
        ctx: Context,
        block_hash: &Hash,
    ) -> ProtocolResult<Option<Header>> {
        if let Some(num) = self.get_block_number_by_hash(block_hash).await? {
            return self.get_block_header(ctx, num).await;
        }

        Ok(None)
    }

    async fn get_block_number_by_hash(
        &self,
        _: Context,
        block_hash: &Hash,
    ) -> ProtocolResult<Option<u64>> {
        self.get_block_number_by_hash(block_hash).await
    }

    #[trace_span(kind = "storage")]
    async fn get_transactions(
        &self,
        ctx: Context,
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
                inst.elapsed(),
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
                } else if !set.contains(key.hash()) {
                    continue;
                } else {
                    found.push((*key.hash(), stx_bytes));
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
        self.adapter.insert::<EvmCodeSchema>(code_hash, code)?;
        self.adapter
            .insert::<EvmCodeAddressSchema>(code_address, code_hash)
    }

    async fn get_code_by_hash(&self, _ctx: Context, hash: &Hash) -> ProtocolResult<Option<Bytes>> {
        get_cache!(self, hash, codes);
        let ret = self.adapter.get::<EvmCodeSchema>(*hash)?;
        put_cache!(self, hash, ret, codes);
        Ok(ret)
    }

    async fn get_code_by_address(
        &self,
        ctx: Context,
        address: &H256,
    ) -> ProtocolResult<Option<Bytes>> {
        let code_hash = self.adapter.get::<EvmCodeAddressSchema>(*address)?;

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
        get_cache!(self, hash, transactions);

        if let Some(block_height) = get!(self, *hash, TxHashNumberSchema)? {
            get!(
                self,
                CommonHashKey::new(block_height, *hash),
                TransactionSchema,
                hash,
                transactions
            )
        } else {
            Ok(None)
        }
    }

    #[trace_span(kind = "storage")]
    async fn insert_receipts(
        &self,
        ctx: Context,
        block_height: u64,
        receipts: Vec<Receipt>,
    ) -> ProtocolResult<()> {
        self.batch_insert_receipts(receipts, block_height).await?;

        Ok(())
    }

    async fn get_receipt_by_hash(
        &self,
        _ctx: Context,
        hash: &Hash,
    ) -> ProtocolResult<Option<Receipt>> {
        get_cache!(self, hash, receipts);

        if let Some(block_height) = get!(self, *hash, TxHashNumberSchema)? {
            get!(
                self,
                CommonHashKey::new(block_height, *hash),
                ReceiptSchema,
                hash,
                receipts
            )
        } else {
            Ok(None)
        }
    }

    #[trace_span(kind = "storage")]
    async fn get_receipts(
        &self,
        ctx: Context,
        block_height: u64,
        hashes: &[Hash],
    ) -> ProtocolResult<Vec<Option<Receipt>>> {
        if hashes.is_empty() {
            return Ok(Vec::new());
        }

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
            on_storage_get_cf(StorageCategory::Receipt, inst.elapsed(), count as f64);

            while count > 0 {
                let (key, stx_bytes) = match iter.next() {
                    None => break,
                    Some(Ok(key_to_stx_bytes)) => key_to_stx_bytes,
                    Some(Err(err)) => return Err(err),
                };

                // Note: fix clippy::suspicious_else_formatting
                if key.height() != block_height {
                    break;
                } else if !set.contains(key.hash()) {
                    continue;
                } else {
                    found.push((*key.hash(), stx_bytes));
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
            .insert::<LatestProofSchema>(*LATEST_PROOF_KEY, proof.clone())?;

        self.latest_proof.store(Arc::new(Some(proof)));

        Ok(())
    }

    async fn get_latest_proof(&self, _ctx: Context) -> ProtocolResult<Proof> {
        if let Some(proof) = self.latest_proof.load().as_ref().clone() {
            Ok(proof)
        } else {
            let proof = ensure_get!(self, *LATEST_PROOF_KEY, LatestProofSchema);
            Ok(proof)
        }
    }
}

#[derive(Debug, Display, From)]
pub enum StorageError {
    #[display(fmt = "get none {:?}", _0)]
    GetNone(String),

    #[display(fmt = "decode batch value")]
    BatchDecode,
}

impl Error for StorageError {}

impl From<StorageError> for ProtocolError {
    fn from(err: StorageError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Storage, Box::new(err))
    }
}
