mod abi;
pub mod exec;
mod store;
pub mod utils;

pub use abi::image_cell_abi;
pub use store::{CellInfo, CellKey};

use std::sync::atomic::{AtomicBool, Ordering};

use ethers::abi::AbiDecode;

use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{MerkleRoot, SignedTransaction, TxResp, H160, H256};
use protocol::ProtocolResult;

use crate::system_contract::image_cell::store::get_cell;
use crate::system_contract::utils::succeed_resp;
use crate::system_contract::{system_contract_address, SystemContract};
use crate::{exec_try, MPTTrie};

pub use super::ckb_light_client::utils::init;
use super::ckb_light_client::utils::{get_mpt, update_mpt_root};
use super::ckb_light_client::CURRENT_CELL_ROOT;

static ALLOW_READ: AtomicBool = AtomicBool::new(false);

#[derive(Default)]
pub struct ImageCellContract;

impl SystemContract for ImageCellContract {
    const ADDRESS: H160 = system_contract_address(0x3);

    fn exec_<B: Backend + ApplyBackend>(&self, backend: &mut B, tx: &SignedTransaction) -> TxResp {
        let tx = &tx.transaction.unsigned;
        let tx_data = tx.data();
        let gas_limit = *tx.gas_limit();

        match image_cell_abi::ImageCellCalls::decode(tx_data) {
            Ok(image_cell_abi::ImageCellCalls::SetState(data)) => {
                ALLOW_READ.store(data.allow_read, Ordering::Relaxed);
            }
            Ok(image_cell_abi::ImageCellCalls::Update(data)) => {
                let mut mpt = exec_try!(get_mpt(), gas_limit, "[image cell] get mpt error:");

                let root: MerkleRoot = exec_try!(
                    exec::update(&mut mpt, data),
                    gas_limit,
                    "[image cell] update error:"
                );

                update_mpt_root(backend, root, ImageCellContract::ADDRESS);
            }
            Ok(image_cell_abi::ImageCellCalls::Rollback(data)) => {
                let mut mpt = exec_try!(get_mpt(), gas_limit, "[image cell] get mpt error:");

                let root: MerkleRoot = exec_try!(
                    exec::rollback(&mut mpt, data),
                    gas_limit,
                    "[image cell] rollback error:"
                );

                update_mpt_root(backend, root, ImageCellContract::ADDRESS);
            }
            _ => unreachable!(),
        }

        succeed_resp(gas_limit)
    }
}

impl ImageCellContract {
    pub fn get_root(&self) -> H256 {
        **CURRENT_CELL_ROOT.load()
    }

    pub fn get_cell(&self, key: &CellKey) -> ProtocolResult<Option<CellInfo>> {
        let mpt = get_mpt()?;
        get_cell(&mpt, key)
    }

    pub fn allow_read(&self) -> bool {
        ALLOW_READ.load(Ordering::Relaxed)
    }
}
