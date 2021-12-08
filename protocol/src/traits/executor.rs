use crate::types::{Hash, SignedTransaction};
use async_trait::async_trait;
use evm::executor::stack::Log;
use evm::ExitReason;

pub struct ExecuteResult {
    exit_reason: ExitReason,
    ret:         Vec<u8>,
    logs:        Vec<Log>,
}

pub struct BatchExecuteResult {
    root:    Hash,
    results: Vec<ExecuteResult>,
}

#[async_trait]
pub trait Executor: Send + Sync {
    async fn batch_execute(&self, signed_txs: Vec<SignedTransaction>) -> BatchExecuteResult;
}
