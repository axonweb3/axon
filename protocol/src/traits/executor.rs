use crate::types::{Address, SignedTransaction};
use async_trait::async_trait;
use evm::backend::{ApplyBackend, Backend};
use evm::ExitReason;

pub struct ExecuteResult {
    pub exit_reason: ExitReason,
    pub ret:         Vec<u8>,
    pub remain_gas:  u64,
}

#[async_trait]
pub trait Executor: Send + Sync {
    async fn call<B: Backend + Send>(
        &self,
        backend: &mut B,
        addr: Address,
        data: Vec<u8>,
    ) -> ExecuteResult;

    async fn execute<B: Backend + ApplyBackend + Send>(
        &self,
        backend: &mut B,
        tx: SignedTransaction,
    ) -> ExecuteResult;
}
