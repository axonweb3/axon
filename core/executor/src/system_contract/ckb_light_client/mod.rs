mod abi;
mod store;

pub use abi::ckb_light_client_abi;

use ckb_types::packed;
use ethers::abi::AbiDecode;
use protocol::ProtocolResult;
use std::sync::atomic::{AtomicBool, Ordering};

use protocol::traits::ExecutorAdapter;
use protocol::types::{SignedTransaction, TxResp, H160, H256};

use crate::exec_try;
use crate::system_contract::ckb_light_client::store::CkbLightClientStore;
use crate::system_contract::utils::{succeed_resp, update_mpt_root};
use crate::system_contract::{system_contract_address, SystemContract, CURRENT_HEADER_CELL_ROOT};

static ALLOW_READ: AtomicBool = AtomicBool::new(false);

#[derive(Default)]
pub struct CkbLightClientContract;

impl SystemContract for CkbLightClientContract {
    const ADDRESS: H160 = system_contract_address(0x2);

    fn exec_<Adapter: ExecutorAdapter>(
        &self,
        adapter: &mut Adapter,
        tx: &SignedTransaction,
    ) -> TxResp {
        let tx = &tx.transaction.unsigned;
        let tx_data = tx.data();
        let gas_limit = *tx.gas_limit();

        let mut store = exec_try!(
            CkbLightClientStore::new(),
            gas_limit,
            "[ckb light client] init ckb light client mpt"
        );

        let call_abi = exec_try!(
            ckb_light_client_abi::CkbLightClientContractCalls::decode(tx_data),
            gas_limit,
            "[ckb light client] invalid tx data"
        );

        match call_abi {
            ckb_light_client_abi::CkbLightClientContractCalls::SetState(data) => {
                ALLOW_READ.store(data.allow_read, Ordering::Relaxed);
            }
            ckb_light_client_abi::CkbLightClientContractCalls::Update(data) => {
                exec_try!(
                    store.update(data),
                    gas_limit,
                    "[ckb light client] update error:"
                );
            }
            ckb_light_client_abi::CkbLightClientContractCalls::Rollback(data) => {
                exec_try!(
                    store.rollback(data),
                    gas_limit,
                    "[ckb light client] update error:"
                );
            }
        }

        update_mpt_root(adapter, CkbLightClientContract::ADDRESS);

        succeed_resp(gas_limit)
    }
}

impl CkbLightClientContract {
    pub fn get_root(&self) -> H256 {
        **CURRENT_HEADER_CELL_ROOT.load()
    }

    pub fn get_header_by_block_hash(
        &self,
        block_hash: &H256,
    ) -> ProtocolResult<Option<packed::Header>> {
        let store = CkbLightClientStore::new()?;
        store.get_header(&block_hash.0)
    }

    pub fn allow_read(&self) -> bool {
        ALLOW_READ.load(Ordering::Relaxed)
    }
}
