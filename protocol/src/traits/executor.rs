pub use evm::backend::{ApplyBackend, Backend};

use crate::types::{Address, ExecResp, Log, MerkleRoot, SignedTransaction, TxResp};

pub trait ExecutorAdapter {
    fn get_logs(&self) -> Vec<Log>;

    fn state_root(&self) -> MerkleRoot;
}

pub trait Executor: Send + Sync {
    fn call<B: Backend>(&self, backend: &mut B, addr: Address, data: Vec<u8>) -> TxResp;

    fn exec<B: Backend + ApplyBackend + ExecutorAdapter>(
        &self,
        backend: &mut B,
        txs: Vec<SignedTransaction>,
    ) -> ExecResp;
}
