use async_trait::async_trait;
use derive_more::Display;

use crate::traits::Context;
use crate::types::{Block, Bytes, Hash, Header, Proof, Receipt, SignedTransaction, H256};
use crate::{codec::ProtocolCodec, ProtocolResult};

#[derive(Debug, Copy, Clone, Display)]
pub enum StorageCategory {
    Block,
    BlockHeader,
    Receipt,
    SignedTransaction,
    Wal,
    HashHeight,
    Code,
}

pub type StorageIterator<'a, S> = Box<
    dyn Iterator<Item = ProtocolResult<(<S as StorageSchema>::Key, <S as StorageSchema>::Value)>>
        + 'a,
>;

pub trait StorageSchema {
    type Key: ProtocolCodec + Send;
    type Value: ProtocolCodec + Send;

    fn category() -> StorageCategory;
}

pub trait IntoIteratorByRef<S: StorageSchema> {
    fn ref_to_iter<'a, 'b: 'a>(&'b self) -> StorageIterator<'a, S>;
}

#[async_trait]
pub trait CommonStorage: Send + Sync {
    async fn insert_block(&self, ctx: Context, block: Block) -> ProtocolResult<()>;

    async fn get_block(&self, ctx: Context, height: u64) -> ProtocolResult<Option<Block>>;

    async fn get_block_header(&self, ctx: Context, height: u64) -> ProtocolResult<Option<Header>>;

    async fn set_block(&self, _ctx: Context, block: Block) -> ProtocolResult<()>;

    async fn remove_block(&self, ctx: Context, height: u64) -> ProtocolResult<()>;

    async fn get_latest_block(&self, ctx: Context) -> ProtocolResult<Block>;

    async fn set_latest_block(&self, ctx: Context, block: Block) -> ProtocolResult<()>;

    async fn get_latest_block_header(&self, ctx: Context) -> ProtocolResult<Header>;
}

#[async_trait]
pub trait Storage: CommonStorage {
    async fn insert_transactions(
        &self,
        ctx: Context,
        block_height: u64,
        signed_txs: Vec<SignedTransaction>,
    ) -> ProtocolResult<()>;

    async fn get_block_by_hash(
        &self,
        ctx: Context,
        block_hash: &Hash,
    ) -> ProtocolResult<Option<Block>>;

    async fn get_transactions(
        &self,
        ctx: Context,
        block_height: u64,
        hashes: &[Hash],
    ) -> ProtocolResult<Vec<Option<SignedTransaction>>>;

    async fn get_transaction_by_hash(
        &self,
        ctx: Context,
        hash: &Hash,
    ) -> ProtocolResult<Option<SignedTransaction>>;

    async fn insert_receipts(
        &self,
        ctx: Context,
        block_height: u64,
        receipts: Vec<Receipt>,
    ) -> ProtocolResult<()>;

    async fn insert_code(
        &self,
        ctx: Context,
        code_address: H256,
        code_hash: Hash,
        code: Bytes,
    ) -> ProtocolResult<()>;

    async fn get_code_by_hash(&self, ctx: Context, hash: &Hash) -> ProtocolResult<Option<Bytes>>;

    async fn get_code_by_address(
        &self,
        _ctx: Context,
        address: &H256,
    ) -> ProtocolResult<Option<Bytes>>;

    async fn get_receipt_by_hash(
        &self,
        ctx: Context,
        hash: Hash,
    ) -> ProtocolResult<Option<Receipt>>;

    async fn get_receipts(
        &self,
        ctx: Context,
        block_height: u64,
        hashes: &[Hash],
    ) -> ProtocolResult<Vec<Option<Receipt>>>;

    async fn update_latest_proof(&self, ctx: Context, proof: Proof) -> ProtocolResult<()>;

    async fn get_latest_proof(&self, ctx: Context) -> ProtocolResult<Proof>;
}

#[async_trait]
pub trait MaintenanceStorage: CommonStorage {}

pub enum StorageBatchModify<S: StorageSchema> {
    Remove,
    Insert(<S as StorageSchema>::Value),
}

#[async_trait]
pub trait StorageAdapter: Send + Sync {
    async fn insert<S: StorageSchema>(
        &self,
        key: <S as StorageSchema>::Key,
        val: <S as StorageSchema>::Value,
    ) -> ProtocolResult<()>;

    async fn get<S: StorageSchema>(
        &self,
        key: <S as StorageSchema>::Key,
    ) -> ProtocolResult<Option<<S as StorageSchema>::Value>>;

    async fn get_batch<S: StorageSchema>(
        &self,
        keys: Vec<<S as StorageSchema>::Key>,
    ) -> ProtocolResult<Vec<Option<<S as StorageSchema>::Value>>> {
        let mut vec = Vec::new();

        for key in keys {
            vec.push(self.get::<S>(key).await?);
        }

        Ok(vec)
    }

    async fn remove<S: StorageSchema>(&self, key: <S as StorageSchema>::Key) -> ProtocolResult<()>;

    async fn contains<S: StorageSchema>(
        &self,
        key: <S as StorageSchema>::Key,
    ) -> ProtocolResult<bool>;

    async fn batch_modify<S: StorageSchema>(
        &self,
        keys: Vec<<S as StorageSchema>::Key>,
        vals: Vec<StorageBatchModify<S>>,
    ) -> ProtocolResult<()>;

    fn prepare_iter<'a, 'b: 'a, S: StorageSchema + 'static, P: AsRef<[u8]> + 'a>(
        &'b self,
        prefix: &'a P,
    ) -> ProtocolResult<Box<dyn IntoIteratorByRef<S> + 'a>>;
}
