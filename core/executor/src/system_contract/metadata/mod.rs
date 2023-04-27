mod abi;
mod handle;
mod segment;
mod store;

pub use abi::metadata_abi;
pub use handle::MetadataHandle;
pub use store::MetadataStore;

use std::num::NonZeroUsize;

use ethers::abi::AbiDecode;
use lru::LruCache;
use parking_lot::RwLock;

use protocol::traits::ExecutorAdapter;
use protocol::types::{Hasher, Metadata, SignedTransaction, TxResp, H160, H256};

use crate::exec_try;
use crate::system_contract::utils::{revert_resp, succeed_resp, update_states};
use crate::system_contract::{system_contract_address, SystemContract, CURRENT_METADATA_ROOT};

type Epoch = u64;

const METADATA_CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(10) };

lazy_static::lazy_static! {
    static ref EPOCH_SEGMENT_KEY: H256 = Hasher::digest("epoch_segment");
    static ref CKB_RELATED_INFO_KEY: H256 = Hasher::digest("ckb_related_info");
    static ref METADATA_CACHE: RwLock<LruCache<Epoch, Metadata>> =  RwLock::new(LruCache::new(METADATA_CACHE_SIZE));
}

#[derive(Default)]
pub struct MetadataContract;

impl SystemContract for MetadataContract {
    const ADDRESS: H160 = system_contract_address(0x1);

    fn exec_<Adapter: ExecutorAdapter>(
        &self,
        adapter: &mut Adapter,
        tx: &SignedTransaction,
    ) -> TxResp {
        let sender = tx.sender;
        let tx = &tx.transaction.unsigned;
        let tx_data = tx.data();
        let gas_limit = *tx.gas_limit();
        let block_number = adapter.block_number().as_u64();

        let mut store = exec_try!(
            MetadataStore::new(),
            gas_limit,
            "[metadata] init metadata mpt"
        );

        if block_number != 0 {
            let handle = MetadataHandle::default();

            if !exec_try!(
                handle.is_validator(block_number, sender),
                gas_limit,
                "[metadata] is validator"
            ) {
                return revert_resp(gas_limit);
            }
        }

        let call_abi = exec_try!(
            metadata_abi::MetadataContractCalls::decode(tx_data),
            gas_limit,
            "[metadata] invalid tx data"
        );

        match call_abi {
            metadata_abi::MetadataContractCalls::AppendMetadata(c) => {
                exec_try!(
                    store.append_metadata(&c.metadata.into()),
                    gas_limit,
                    "[metadata] append metadata"
                );
            }
            metadata_abi::MetadataContractCalls::SetCkbRelatedInfo(c) => {
                if !adapter.block_number().is_zero() {
                    return revert_resp(gas_limit);
                }

                exec_try!(
                    store.set_ckb_related_info(&c.ckb_related_info.into()),
                    gas_limit,
                    "[metadata] set ckb related info"
                );
            }
            // TODO: Metadata doesn't accept all abi calls so far.
            _ => {
                log::error!("[metadata] invalid tx data");
                return revert_resp(gas_limit);
            }
        }

        update_states(adapter, sender, Self::ADDRESS);

        succeed_resp(gas_limit)
    }

    fn after_block_hook<Adapter: ExecutorAdapter>(&self, adapter: &mut Adapter) {
        MetadataStore::new()
            .unwrap()
            .update_propose_count(adapter.block_number().as_u64(), &adapter.origin())
            .unwrap();
    }
}

pub fn check_ckb_related_info_exist() -> bool {
    MetadataHandle::default().get_ckb_related_info().is_ok()
}
