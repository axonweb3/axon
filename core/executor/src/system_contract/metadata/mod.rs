mod abi;
pub(crate) mod handle;
mod segment;
mod store;

pub use abi::metadata_abi;
pub use handle::MetadataHandle;
pub use store::MetadataStore;

use std::num::NonZeroUsize;

use ethers::abi::AbiDecode;
use lru::LruCache;
use parking_lot::RwLock;

use protocol::traits::{ApplyBackend, ExecutorAdapter};
use protocol::types::{Hasher, Metadata, SignedTransaction, TxResp, H160, H256};

use crate::system_contract::utils::{
    generate_mpt_root_changes, revert_resp, succeed_resp, update_states,
};
use crate::system_contract::{system_contract_address, SystemContract};
use crate::{exec_try, system_contract_struct, CURRENT_METADATA_ROOT};

type Epoch = u64;

pub const METADATA_CONTRACT_ADDRESS: H160 = system_contract_address(0x1);
const METADATA_CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(10) };

lazy_static::lazy_static! {
    static ref EPOCH_SEGMENT_KEY: H256 = Hasher::digest("epoch_segment");
    static ref CKB_RELATED_INFO_KEY: H256 = Hasher::digest("ckb_related_info");
    static ref METADATA_CACHE: RwLock<LruCache<Epoch, Metadata>> =  RwLock::new(LruCache::new(METADATA_CACHE_SIZE));
}

system_contract_struct!(MetadataContract);

impl<Adapter: ExecutorAdapter + ApplyBackend> SystemContract<Adapter>
    for MetadataContract<Adapter>
{
    const ADDRESS: H160 = METADATA_CONTRACT_ADDRESS;

    fn exec_(&self, adapter: &mut Adapter, tx: &SignedTransaction) -> TxResp {
        let sender = tx.sender;
        let tx = &tx.transaction.unsigned;
        let tx_data = tx.data();
        let gas_limit = *tx.gas_limit();
        let block_number = adapter.block_number().as_u64();
        let root = CURRENT_METADATA_ROOT.with(|r| *r.borrow());

        let mut store = exec_try!(
            MetadataStore::new(root),
            gas_limit,
            "[metadata] init metadata mpt"
        );

        if block_number != 0 {
            let handle = MetadataHandle::new(CURRENT_METADATA_ROOT.with(|r| *r.borrow()));

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

    fn after_block_hook(&self, adapter: &mut Adapter) {
        let block_number = adapter.block_number();
        if block_number.is_zero() {
            return;
        }

        let root = CURRENT_METADATA_ROOT.with(|r| *r.borrow());
        if let Err(e) = MetadataStore::new(root)
            .unwrap()
            .update_propose_count(block_number.as_u64(), &adapter.origin())
        {
            panic!("Update propose count at {:?} failed: {:?}", block_number, e)
        }

        let changes = generate_mpt_root_changes(adapter, Self::ADDRESS);
        adapter.apply(changes, vec![], false);
    }
}

pub fn check_ckb_related_info_exist(root: H256) -> bool {
    MetadataHandle::new(root).get_ckb_related_info().is_ok()
}
