mod abi;
mod handle;
mod store;

pub use abi::ckb_light_client_abi;
pub use handle::CkbLightClientHandle;

use ckb_types::packed;
use ethers::abi::AbiDecode;
use std::sync::atomic::{AtomicBool, Ordering};

use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{SignedTransaction, TxResp, H160, H256};
use protocol::ProtocolResult;

use crate::exec_try;
use crate::system_contract::ckb_light_client::store::CkbLightClientStore;
use crate::system_contract::utils::{succeed_resp, update_mpt_root};
use crate::system_contract::{system_contract_address, SystemContract, CURRENT_HEADER_CELL_ROOT};

static ALLOW_READ: AtomicBool = AtomicBool::new(false);

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
                exec_try!(
                    store.update(data),
                    gas_limit,
                    "[ckb light client] update error:"
                );
            }
            Ok(ckb_light_client_abi::CkbLightClientCalls::Rollback(data)) => {
                exec_try!(
                    store.rollback(data),
                    gas_limit,
                    "[ckb light client] update error:"
                );
            }
            _ => unreachable!(),
        }
        update_mpt_root(backend, CkbLightClientContract::ADDRESS);
        succeed_resp(gas_limit)
    }
}

impl CkbLightClientContract {
    pub fn get_root(&self) -> H256 {
        **CURRENT_HEADER_CELL_ROOT.load()
    }

    pub fn get_header(&self, block_hash: &H256) -> ProtocolResult<Option<packed::Header>> {
        CkbLightClientStore::new()?.get_header(block_hash)
    }

    pub fn allow_read(&self) -> bool {
        ALLOW_READ.load(Ordering::Relaxed)
    }
}
