mod abi;
pub(crate) mod handle;
pub mod segment;
mod store;

pub use abi::metadata_abi;
pub use handle::MetadataHandle;
pub use store::{encode_consensus_config, MetadataStore};

use std::{num::NonZeroUsize, sync::Arc};

use arc_swap::ArcSwap;
use ethers::abi::AbiDecode;
use lru::LruCache;
use parking_lot::RwLock;

use protocol::codec::ProtocolCodec;
use protocol::traits::{ApplyBackend, ExecutorAdapter};
use protocol::types::{HardforkInfoInner, Hasher, Metadata, SignedTransaction, TxResp, H160, H256};

use crate::system_contract::utils::{
    generate_mpt_root_changes, revert_resp, succeed_resp, update_states,
};
use crate::system_contract::{system_contract_address, SystemContract};
use crate::{exec_try, system_contract_struct, CURRENT_METADATA_ROOT};

type Epoch = u64;

pub const METADATA_CONTRACT_ADDRESS: H160 = system_contract_address(0x1);
const METADATA_CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(10) };

lazy_static::lazy_static! {
    pub static ref EPOCH_SEGMENT_KEY: H256 = Hasher::digest("epoch_segment");
    static ref CKB_RELATED_INFO_KEY: H256 = Hasher::digest("ckb_related_info");
    pub static ref CONSENSUS_CONFIG: H256 = Hasher::digest("consensus_config");
    pub static ref HARDFORK_KEY: H256 = Hasher::digest("hardfork");
    pub static ref HARDFORK_INFO: ArcSwap<H256> = ArcSwap::new(Arc::new(H256::zero()));
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

        let mut store =
            exec_try!(
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
                exec_try!(
                    store.set_ckb_related_info(&c.info.into()),
                    gas_limit,
                    "[metadata] set ckb related info"
                );
            }
            metadata_abi::MetadataContractCalls::UpdateConsensusConfig(c) => {
                exec_try!(
                    store.update_consensus_config(c.config.into()),
                    gas_limit,
                    "[metadata] update consensus config"
                );
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

        let mut store = MetadataStore::new(root).unwrap();

        if let Some(t) = adapter.get_ctx().extra_data.get(0) {
            if let Ok(data) = HardforkInfoInner::decode(&t.inner) {
                store
                    .set_hardfork_info(data.block_number, data.flags)
                    .expect("set new hardfork info fail")
            }
        }

        let hardfork = store.hardfork_info(block_number.as_u64()).unwrap();

        HARDFORK_INFO.swap(Arc::new(hardfork));

        if let Err(e) = store.update_propose_count(block_number.as_u64(), &adapter.origin()) {
            panic!("Update propose count at {:?} failed: {:?}", block_number, e)
        }

        let changes = generate_mpt_root_changes(adapter, Self::ADDRESS);
        adapter.apply(changes, vec![], false);
    }
}

pub fn check_ckb_related_info_exist(root: H256) -> bool {
    MetadataHandle::new(root).get_ckb_related_info().is_ok()
}
