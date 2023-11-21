use std::sync::Arc;

use evm::backend::Basic;

use protocol::traits::{Backend, Context, ExecutorReadOnlyAdapter, ReadOnlyStorage};
use protocol::trie::Trie as _;
use protocol::types::{
    Account, BigEndianHash, Bytes, ExecutorContext, MerkleRoot, H160, H256, NIL_DATA, RLP_NULL,
    U256,
};
use protocol::{codec::ProtocolCodec, trie, ProtocolResult};

use crate::system_contract::{
    HEADER_CELL_ROOT_KEY, IMAGE_CELL_CONTRACT_ADDRESS, METADATA_CONTRACT_ADDRESS, METADATA_ROOT_KEY,
};
use crate::{blocking_async, MPTTrie};

const GET_BLOCK_HASH_NUMBER_RANGE: u64 = 256;

pub struct AxonExecutorReadOnlyAdapter<S, DB: trie::DB> {
    pub(crate) exec_ctx: ExecutorContext,
    pub(crate) trie:     MPTTrie<DB>,
    pub(crate) storage:  Arc<S>,
    pub(crate) db:       Arc<DB>,
}

impl<S, DB> ExecutorReadOnlyAdapter for AxonExecutorReadOnlyAdapter<S, DB>
where
    S: ReadOnlyStorage + 'static,
    DB: trie::DB + 'static,
{
    fn get_ctx(&self) -> ExecutorContext {
        self.exec_ctx.clone()
    }

    fn get(&self, key: &[u8]) -> Option<Bytes> {
        self.trie.get(key).ok().flatten().map(Into::into)
    }

    fn get_account(&self, address: &H160) -> Account {
        if let Ok(Some(raw)) = self.trie.get(address.as_bytes()) {
            return Account::decode(raw).unwrap();
        }

        Account {
            nonce:        U256::zero(),
            balance:      U256::zero(),
            storage_root: RLP_NULL,
            code_hash:    NIL_DATA,
        }
    }
}

impl<S, DB> Backend for AxonExecutorReadOnlyAdapter<S, DB>
where
    S: ReadOnlyStorage + 'static,
    DB: trie::DB + 'static,
{
    fn gas_price(&self) -> U256 {
        self.exec_ctx.gas_price
    }

    fn origin(&self) -> H160 {
        self.exec_ctx.origin
    }

    fn block_number(&self) -> U256 {
        self.exec_ctx.block_number
    }

    fn block_hash(&self, number: U256) -> H256 {
        let current_number = self.block_number();
        if number >= current_number {
            return H256::default();
        }

        if (current_number - number) > U256::from(GET_BLOCK_HASH_NUMBER_RANGE) {
            return H256::default();
        }

        let number = number.as_u64();
        blocking_async!(self, get_storage, get_block, Context::new(), number)
            .map(|b| b.hash())
            .unwrap_or_default()
    }

    fn block_coinbase(&self) -> H160 {
        self.exec_ctx.block_coinbase
    }

    fn block_timestamp(&self) -> U256 {
        self.exec_ctx.block_timestamp
    }

    fn block_difficulty(&self) -> U256 {
        U256::one()
    }

    fn block_gas_limit(&self) -> U256 {
        self.exec_ctx.block_gas_limit
    }

    fn block_base_fee_per_gas(&self) -> U256 {
        self.exec_ctx.block_base_fee_per_gas
    }

    fn chain_id(&self) -> U256 {
        self.exec_ctx.chain_id
    }

    fn exists(&self, address: H160) -> bool {
        self.trie
            .contains(&Bytes::from(address.as_bytes().to_vec()))
            .unwrap_or(false)
    }

    fn basic(&self, address: H160) -> Basic {
        self.trie
            .get(address.as_bytes())
            .map(|raw| {
                if raw.is_none() {
                    return Basic::default();
                }
                Account::decode(raw.unwrap()).map_or_else(
                    |_| Default::default(),
                    |account| Basic {
                        balance: account.balance,
                        nonce:   account.nonce,
                    },
                )
            })
            .unwrap_or_default()
    }

    fn code(&self, address: H160) -> Vec<u8> {
        let code_hash = if let Some(bytes) = self.trie.get(address.as_bytes()).unwrap() {
            Account::decode(bytes).unwrap().code_hash
        } else {
            return Vec::new();
        };

        if code_hash == NIL_DATA {
            return Vec::new();
        }

        let res = blocking_async!(
            self,
            get_storage,
            get_code_by_hash,
            Context::new(),
            &code_hash
        );

        res.unwrap_or_default().to_vec()
    }

    // ### Notes
    //
    // - If a MPT tree is empty, the root should be `RLP_NULL`.
    // - In this function, when returns `H256::default()`, that means the tree is
    //   not initialized.
    fn storage(&self, address: H160, index: H256) -> H256 {
        if let Ok(raw) = self.trie.get(address.as_bytes()) {
            if raw.is_none() {
                return H256::default();
            }

            Account::decode(raw.unwrap())
                .and_then(|account| {
                    let storage_root = account.storage_root;
                    if storage_root == RLP_NULL {
                        Ok(H256::default())
                    } else {
                        MPTTrie::from_root(storage_root, Arc::clone(&self.db)).map(
                            |trie| match trie.get(index.as_bytes()) {
                                Ok(Some(res)) => {
                                    let value = U256::decode(res).unwrap();
                                    BigEndianHash::from_uint(&value)
                                }
                                _ => H256::default(),
                            },
                        )
                    }
                })
                .unwrap_or_default()
        } else {
            H256::default()
        }
    }

    fn original_storage(&self, address: H160, index: H256) -> Option<H256> {
        // Fixme
        Some(self.storage(address, index))
    }
}

impl<S, DB> AxonExecutorReadOnlyAdapter<S, DB>
where
    S: ReadOnlyStorage + 'static,
    DB: trie::DB + 'static,
{
    pub fn new(db: Arc<DB>, storage: Arc<S>, exec_ctx: ExecutorContext) -> ProtocolResult<Self> {
        let trie = MPTTrie::new(Arc::clone(&db));
        Ok(AxonExecutorReadOnlyAdapter {
            trie,
            db,
            storage,
            exec_ctx,
        })
    }

    pub fn from_root(
        state_root: MerkleRoot,
        db: Arc<DB>,
        storage: Arc<S>,
        exec_ctx: ExecutorContext,
    ) -> ProtocolResult<Self> {
        let trie = MPTTrie::from_root(state_root, Arc::clone(&db))?;

        Ok(AxonExecutorReadOnlyAdapter {
            trie,
            db,
            storage,
            exec_ctx,
        })
    }

    pub fn get_metadata_root(&self) -> H256 {
        self.storage(METADATA_CONTRACT_ADDRESS, *METADATA_ROOT_KEY)
    }

    pub fn get_image_cell_root(&self) -> H256 {
        self.storage(IMAGE_CELL_CONTRACT_ADDRESS, *HEADER_CELL_ROOT_KEY)
    }

    fn get_storage(&self) -> Arc<S> {
        Arc::clone(&self.storage)
    }
}
