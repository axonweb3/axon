mod metadata_abi;
mod segment;

use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{Hasher, SignedTransaction, TxResp, H160, H256};

use crate::system_contract::{system_contract_address, SystemContract};

lazy_static::lazy_static! {
    static ref EPOCH_SEGMENT_KEY: H256 = Hasher::digest("epoch_segment");
}

#[derive(Default)]
pub struct MetadataContract;

impl SystemContract for MetadataContract {
    const ADDRESS: H160 = system_contract_address(0x00);

    fn exec_<B: Backend + ApplyBackend>(&self, backend: &mut B, tx: &SignedTransaction) -> TxResp {
        todo!()
    }
}
