mod abi;
mod store;

pub mod utils;

pub use abi::image_cell_abi;
pub use store::{CellInfo, CellKey};

use ethers::abi::AbiDecode;
use std::sync::atomic::{AtomicBool, Ordering};

use protocol::traits::ExecutorAdapter;
use protocol::types::{SignedTransaction, TxResp, H160, H256};
use protocol::ProtocolResult;

use crate::system_contract::image_cell::store::ImageCellStore;
use crate::system_contract::utils::{succeed_resp, update_states};
use crate::system_contract::{system_contract_address, SystemContract, CURRENT_HEADER_CELL_ROOT};
use crate::{exec_try, MPTTrie};

static ALLOW_READ: AtomicBool = AtomicBool::new(false);

#[derive(Default)]
pub struct ImageCellContract;

impl SystemContract for ImageCellContract {
    const ADDRESS: H160 = system_contract_address(0x3);

    fn exec_<Adapter: ExecutorAdapter>(
        &self,
        adapter: &mut Adapter,
        tx: &SignedTransaction,
    ) -> TxResp {
        let sender = tx.sender;
        let tx = &tx.transaction.unsigned;
        let tx_data = tx.data();
        let gas_limit = *tx.gas_limit();

        let mut store = exec_try!(
            ImageCellStore::new(),
            gas_limit,
            "[image cell] init image cell mpt"
        );

        let call_abi = exec_try!(
            image_cell_abi::ImageCellContractCalls::decode(tx_data),
            gas_limit,
            "[image cell] invalid tx data"
        );

        match call_abi {
            image_cell_abi::ImageCellContractCalls::SetState(data) => {
                ALLOW_READ.store(data.allow_read, Ordering::Relaxed);
            }
            image_cell_abi::ImageCellContractCalls::Update(data) => {
                exec_try!(store.update(data), gas_limit, "[image cell] update error:");
            }
            image_cell_abi::ImageCellContractCalls::Rollback(data) => {
                exec_try!(
                    store.rollback(data),
                    gas_limit,
                    "[image cell] rollback error:"
                );
            }
        }

        update_states(adapter, sender, Self::ADDRESS);

        succeed_resp(gas_limit)
    }
}

impl ImageCellContract {
    pub fn get_root(&self) -> H256 {
        **CURRENT_HEADER_CELL_ROOT.load()
    }

    pub fn get_cell(&self, key: &CellKey) -> ProtocolResult<Option<CellInfo>> {
        ImageCellStore::new()?.get_cell(key)
    }

    pub fn allow_read(&self) -> bool {
        ALLOW_READ.load(Ordering::Relaxed)
    }

    pub fn save_cells(
        &self,
        cells: Vec<image_cell_abi::CellInfo>,
        created_number: u64,
    ) -> ProtocolResult<()> {
        ImageCellStore::new()?.save_cells(cells, created_number)
    }
}
