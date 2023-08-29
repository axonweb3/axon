use std::{path::Path, sync::Arc};

use common_config_parser::types::{spec::InitialAccount, ConfigRocksDB};
use core_db::{RocksAdapter, RocksDB};
use core_executor::{MPTTrie, RocksTrieDB};
use core_storage::ImplStorage;
use protocol::{
    async_trait,
    codec::ProtocolCodec,
    traits::{Context, Storage},
    trie::{self, Trie},
    types::{Account, Block, ExecResp, HasherKeccak, RichBlock, NIL_DATA, RLP_NULL},
    ProtocolResult,
};

pub(crate) struct DatabaseGroup {
    storage:  Arc<ImplStorage<RocksAdapter>>,
    trie_db:  Arc<RocksTrieDB>,
    inner_db: Arc<RocksDB>,
}

impl DatabaseGroup {
    pub(crate) fn new<P: AsRef<Path>>(
        config: &ConfigRocksDB,
        rocksdb_path: P,
        triedb_cache_size: usize,
    ) -> ProtocolResult<Self> {
        let adapter = Arc::new(RocksAdapter::new(rocksdb_path, config.clone())?);
        let inner_db = adapter.inner_db();
        let trie_db = Arc::new(RocksTrieDB::new_evm(adapter.inner_db(), triedb_cache_size));
        let storage = Arc::new(ImplStorage::new(adapter, config.cache_size));
        Ok(Self {
            storage,
            trie_db,
            inner_db,
        })
    }

    pub(crate) fn storage(&self) -> Arc<ImplStorage<RocksAdapter>> {
        Arc::clone(&self.storage)
    }

    pub(crate) fn trie_db(&self) -> Arc<RocksTrieDB> {
        Arc::clone(&self.trie_db)
    }

    pub(crate) fn inner_db(&self) -> Arc<RocksDB> {
        Arc::clone(&self.inner_db)
    }
}

#[async_trait]
pub(crate) trait StorageExt: Storage {
    async fn try_load_genesis(&self) -> ProtocolResult<Option<Block>> {
        self.get_block(Context::new(), 0).await.or_else(|e| {
            if e.to_string().contains("GetNone") {
                Ok(None)
            } else {
                Err(e)
            }
        })
    }

    async fn save_block(&self, rich: &RichBlock, resp: &ExecResp) -> ProtocolResult<()> {
        self.update_latest_proof(Context::new(), rich.block.header.proof.clone())
            .await?;
        self.insert_block(Context::new(), rich.block.clone())
            .await?;
        self.insert_transactions(Context::new(), rich.block.header.number, rich.txs.clone())
            .await?;
        let (receipts, _logs) = rich.generate_receipts_and_logs(resp);
        self.insert_receipts(Context::new(), rich.block.header.number, receipts)
            .await?;
        Ok(())
    }
}

impl StorageExt for ImplStorage<RocksAdapter> {}

pub(crate) trait TrieExt<D, H>: Trie<D, H> + Sized
where
    D: trie::DB,
    H: trie::Hasher,
{
    fn insert_accounts(mut self, accounts: &[InitialAccount]) -> ProtocolResult<Self> {
        for account in accounts {
            let raw_account = Account {
                nonce:        0u64.into(),
                balance:      account.balance,
                storage_root: RLP_NULL,
                code_hash:    NIL_DATA,
            }
            .encode()?;
            self.insert(account.address.as_bytes().to_vec(), raw_account.to_vec())?;
        }
        Ok(self)
    }
}

impl<D> TrieExt<D, HasherKeccak> for MPTTrie<D> where D: trie::DB {}
