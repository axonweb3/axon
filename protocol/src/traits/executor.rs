pub use evm::backend::{ApplyBackend, Backend};

use crate::types::{
    Account, Bytes, ExecResp, ExecutorContext, Log, MerkleRoot, SignedTransaction, TxResp, H160,
    U256,
};

pub trait ExecutorAdapter {
    fn set_gas_price(&mut self, gas_price: U256);

    fn get_logs(&mut self) -> Vec<Log>;

    fn state_root(&self) -> MerkleRoot;

    fn get(&self, key: &[u8]) -> Option<Bytes>;

    fn get_ctx(&self) -> ExecutorContext;
}

pub trait Executor: Send + Sync {
    fn call<B: Backend>(
        &self,
        backend: &mut B,
        from: Option<H160>,
        to: Option<H160>,
        data: Vec<u8>,
    ) -> TxResp;

    fn exec<B: Backend + ApplyBackend + ExecutorAdapter>(
        &self,
        backend: &mut B,
        txs: Vec<SignedTransaction>,
    ) -> ExecResp;

    fn get_account<B: Backend + ExecutorAdapter>(&self, backend: &B, address: &H160) -> Account;
}
