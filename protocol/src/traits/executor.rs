pub use evm::backend::{ApplyBackend, Backend};

use crate::types::{Address, ExecResp, SignedTransaction};

pub trait Executor: Send + Sync {
    fn call<B: Backend>(&self, backend: &mut B, addr: Address, data: Vec<u8>) -> ExecResp;

    fn exec<B: Backend + ApplyBackend>(&self, backend: &mut B, tx: SignedTransaction) -> ExecResp;
}
