mod abi;
mod exec;
mod trie_db;

use std::cell::RefCell;
use std::sync::Arc;

use ethers::abi::AbiDecode;

use common_config_parser::types::ConfigRocksDB;
use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{
    Apply, Basic, Bytes, ExitReason, ExitRevert, ExitSucceed, MerkleRoot, SignedTransaction,
    TxResp, H160, H256, U256,
};
use protocol::ProtocolResult;

use crate::system_contract::{system_contract_address, SystemContract};
use crate::MPTTrie;

pub use abi::image_cell_abi;
pub use exec::{CellInfo, CellKey, HeaderKey};
use trie_db::RocksTrieDB;

const BLOCK_NUMBER_KEY: &str = "BlockNumber";

lazy_static::lazy_static! {
    static ref MPT_ROOT_KEY: H256 = H256::default();
}

pub struct MptConfig {
    pub path:          String,
    pub cache_size:    usize,
    pub rockdb_config: ConfigRocksDB,
}

pub struct ImageCellContract {
    // todo: convert to static
    mpt: RefCell<MPTTrie<RocksTrieDB>>,
}

impl ImageCellContract {
    pub fn new(config: MptConfig) -> Self {
        let trie_db = Arc::new(
            RocksTrieDB::new(config.path, config.rockdb_config, config.cache_size)
                .expect("new rocksdb error"),
        );

        ImageCellContract {
            mpt: RefCell::new(MPTTrie::new(trie_db)),
        }
    }
}

impl SystemContract for ImageCellContract {
    const ADDRESS: H160 = system_contract_address(0x1);

    fn exec_<B: Backend + ApplyBackend>(&self, backend: &mut B, tx: &SignedTransaction) -> TxResp {
        let tx = &tx.transaction.unsigned;
        let tx_data = tx.data();

        match image_cell_abi::ImageCellCalls::decode(tx_data) {
            Ok(image_cell_abi::ImageCellCalls::SetState(_)) => {
                // todo
            }
            Ok(image_cell_abi::ImageCellCalls::Update(data)) => {
                let mut mpt = self.mpt.borrow_mut();

                if exec::update(&mut mpt, data).is_err() {
                    return revert_resp(*tx.gas_limit());
                }

                let root = match mpt.commit() {
                    Ok(root) => root,
                    Err(_) => return revert_resp(*tx.gas_limit()),
                };

                if self.update_mpt_root(backend, root).is_err() {
                    return revert_resp(*tx.gas_limit());
                }
            }
            Ok(image_cell_abi::ImageCellCalls::Rollback(_)) => {
                // todo
            }
            Err(_) => {
                return revert_resp(*tx.gas_limit());
            }
        }

        TxResp {
            exit_reason:  ExitReason::Succeed(ExitSucceed::Returned),
            ret:          vec![],
            gas_used:     0u64,
            remain_gas:   tx.gas_limit().as_u64(),
            fee_cost:     U256::zero(),
            logs:         vec![],
            code_address: None,
            removed:      false,
        }
    }
}

impl ImageCellContract {
    fn update_mpt_root<B: Backend + ApplyBackend>(
        &self,
        backend: &mut B,
        root: H256,
    ) -> Result<(), String> {
        let account = backend.basic(ImageCellContract::ADDRESS);

        backend.apply(
            vec![Apply::Modify {
                address:       ImageCellContract::ADDRESS,
                basic:         Basic {
                    balance: account.balance,
                    nonce:   account.nonce + U256::one(),
                },
                code:          None,
                storage:       vec![(*MPT_ROOT_KEY, root)],
                reset_storage: false,
            }],
            vec![],
            false,
        );
        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> ProtocolResult<Option<Bytes>> {
        self.mpt.borrow().get(key)
    }

    pub fn get_root(&self) -> ProtocolResult<MerkleRoot> {
        let mut mpt = self.mpt.borrow_mut();
        mpt.commit()
    }

    pub fn get_block_number(&self) -> ProtocolResult<Option<Bytes>> {
        self.mpt.borrow().get(BLOCK_NUMBER_KEY.as_bytes())
    }
}

fn revert_resp(gas_limit: U256) -> TxResp {
    TxResp {
        exit_reason:  ExitReason::Revert(ExitRevert::Reverted),
        ret:          vec![],
        gas_used:     1u64,
        remain_gas:   (gas_limit - 1).as_u64(),
        fee_cost:     U256::one(),
        logs:         vec![],
        code_address: None,
        removed:      false,
    }
}
