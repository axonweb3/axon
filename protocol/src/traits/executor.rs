pub use evm::backend::{ApplyBackend, Backend, MemoryBackend};

use crate::types::{
    Account, Bytes, ExecResp, ExecutorContext, Log, MerkleRoot, SignedTransaction, TxResp,
    ValidatorExtend, H160, U256, U64,
};

pub trait ExecutorReadOnlyAdapter: Backend {
    fn get(&self, key: &[u8]) -> Option<Bytes>;

    fn get_ctx(&self) -> ExecutorContext;

    fn get_account(&self, address: &H160) -> Account;
}

pub trait ExecutorAdapter: ExecutorReadOnlyAdapter + ApplyBackend {
    fn set_origin(&mut self, origin: H160);

    fn set_gas_price(&mut self, gas_price: U64);

    fn save_account(&mut self, address: &H160, account: &Account);

    fn commit(&mut self) -> MerkleRoot;

    fn take_logs(&mut self) -> Vec<Log>;
}

pub trait Executor: Send + Sync {
    fn call<B: Backend>(
        &self,
        backend: &B,
        gas_limit: u64,
        from: Option<H160>,
        to: Option<H160>,
        value: U256,
        data: Vec<u8>,
        estimate: bool,
    ) -> TxResp;

    fn exec<Adapter: ExecutorAdapter + ApplyBackend>(
        &self,
        adapter: &mut Adapter,
        txs: &[SignedTransaction],
        validators: &[ValidatorExtend],
    ) -> ExecResp;
}

/// This implementation is only used for test.
impl<'a> ExecutorReadOnlyAdapter for MemoryBackend<'a> {
    fn get(&self, _key: &[u8]) -> Option<Bytes> {
        unreachable!()
    }

    fn get_ctx(&self) -> ExecutorContext {
        unreachable!()
    }

    fn get_account(&self, _address: &H160) -> Account {
        unreachable!()
    }
}

impl<'a> ExecutorAdapter for MemoryBackend<'a> {
    fn set_origin(&mut self, _origin: H160) {
        unreachable!()
    }

    fn set_gas_price(&mut self, _gas_price: U64) {
        unreachable!()
    }

    fn take_logs(&mut self) -> Vec<Log> {
        unreachable!()
    }

    fn commit(&mut self) -> MerkleRoot {
        unreachable!()
    }

    fn save_account(&mut self, _address: &H160, _account: &Account) {
        unreachable!()
    }
}
