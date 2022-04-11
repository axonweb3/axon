use crate::types::{Bytes, SignedTransaction, VMResp, H256};
use crate::{traits::Context, ProtocolResult};

pub trait Interoperation: Sync + Send {
    fn verify_external_signature(&self, ctx: Context, tx: SignedTransaction) -> ProtocolResult<()>;

    fn call_ckb_vm(
        &self,
        ctx: Context,
        code_hash: H256,
        args: &[Bytes],
        max_cycles: u64,
    ) -> ProtocolResult<VMResp>;
}
