use async_trait::async_trait;
pub use evm::backend::{ApplyBackend, Backend};

use crate::types::{Address, ExecResponse, SignedTransaction};

#[async_trait]
pub trait Executor: Send + Sync {
    async fn call<B: Backend + Send>(
        &self,
        backend: &mut B,
        addr: Address,
        data: Vec<u8>,
    ) -> ExecResponse;

    async fn exec<B: Backend + ApplyBackend + Send>(
        &self,
        backend: &mut B,
        tx: SignedTransaction,
    ) -> ExecResponse;
}

// #[async_trait]
// pub trait ExecutorAdapter: Send + Sync {
//     async fn
// }
