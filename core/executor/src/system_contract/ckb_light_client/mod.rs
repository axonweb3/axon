mod abi;
mod handle;
mod store;
pub mod utils;

pub use abi::ckb_light_client_abi;
pub use handle::CkbLightClientHandle;

use arc_swap::ArcSwap;
use ckb_types::packed;
use protocol::ProtocolResult;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ethers::abi::AbiDecode;
use once_cell::sync::OnceCell;

use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{Hasher, SignedTransaction, TxResp, H160, H256};

use crate::exec_try;
use crate::system_contract::{system_contract_address, SystemContract};

use crate::system_contract::ckb_light_client::store::CkbLightClientStore;
use crate::system_contract::ckb_light_client::utils::update_mpt_root;

use super::trie_db::RocksTrieDB;
use super::utils::succeed_resp;

static ALLOW_READ: AtomicBool = AtomicBool::new(false);
static TRIE_DB: OnceCell<Arc<RocksTrieDB>> = OnceCell::new();

lazy_static::lazy_static! {
    pub static ref CELL_ROOT_KEY: H256 = Hasher::digest("cell_mpt_root");
    pub static ref CURRENT_CELL_ROOT: ArcSwap<H256> = ArcSwap::from_pointee(H256::default());
}

#[derive(Default)]
pub struct CkbLightClientContract;

impl SystemContract for CkbLightClientContract {
    const ADDRESS: H160 = system_contract_address(0x2);

    fn exec_<B: Backend + ApplyBackend>(&self, backend: &mut B, tx: &SignedTransaction) -> TxResp {
        let tx = &tx.transaction.unsigned;
        let tx_data = tx.data();
        let gas_limit = *tx.gas_limit();

        let mut store = exec_try!(
            CkbLightClientStore::new(),
            gas_limit,
            "[ckb light client] init ckb light client mpt"
        );

        match ckb_light_client_abi::CkbLightClientCalls::decode(tx_data) {
            Ok(ckb_light_client_abi::CkbLightClientCalls::SetState(data)) => {
                ALLOW_READ.store(data.allow_read, Ordering::Relaxed);
            }
            Ok(ckb_light_client_abi::CkbLightClientCalls::Update(data)) => {
                let root = exec_try!(
                    store.update(data),
                    gas_limit,
                    "[ckb light client] update error:"
                );
                update_mpt_root(backend, root, CkbLightClientContract::ADDRESS);
            }
            Ok(ckb_light_client_abi::CkbLightClientCalls::Rollback(data)) => {
                let root = exec_try!(
                    store.rollback(data),
                    gas_limit,
                    "[ckb light client] update error:"
                );
                update_mpt_root(backend, root, CkbLightClientContract::ADDRESS);
            }
            _ => unreachable!(),
        }

        succeed_resp(gas_limit)
    }
}

impl CkbLightClientContract {
    pub fn get_root(&self) -> H256 {
        **CURRENT_CELL_ROOT.load()
    }

    pub fn get_header(&self, block_hash: &H256) -> ProtocolResult<Option<packed::Header>> {
        CkbLightClientStore::new()?.get_header(block_hash)
    }

    pub fn allow_read(&self) -> bool {
        ALLOW_READ.load(Ordering::Relaxed)
    }
}
