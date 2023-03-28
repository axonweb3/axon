pub use evm::backend::{ApplyBackend, Backend, MemoryBackend};

use crate::types::{
    Account, Bytes, ExecResp, ExecutorContext, Log, MerkleRoot, SignedTransaction, TxResp,
    ValidatorExtend, H160, U256,
};

pub trait ExecutorAdapter: ApplyBackend + Backend {
    fn set_origin(&mut self, origin: H160);

    fn set_gas_price(&mut self, gas_price: U256);

    fn get_logs(&mut self) -> Vec<Log>;

    fn commit(&mut self) -> MerkleRoot;

    fn get(&self, key: &[u8]) -> Option<Bytes>;

    fn get_ctx(&self) -> ExecutorContext;

    fn get_account(&self, address: &H160) -> Account;

    fn save_account(&mut self, address: &H160, account: &Account);
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
    ) -> TxResp;

    fn exec<Adapter: ExecutorAdapter>(
        &self,
        adapter: &mut Adapter,
        txs: &[SignedTransaction],
        validators: &[ValidatorExtend],
    ) -> ExecResp;

    fn get_account<Adapter: ExecutorAdapter>(&self, adapter: &Adapter, address: &H160) -> Account;
}

/// This implementation is only used for test.
impl<'a> ExecutorAdapter for MemoryBackend<'a> {
    fn set_origin(&mut self, _origin: H160) {
        unreachable!()
    }

    fn set_gas_price(&mut self, _gas_price: U256) {
        unreachable!()
    }

    fn get_logs(&mut self) -> Vec<Log> {
        unreachable!()
    }

    fn commit(&mut self) -> MerkleRoot {
        unreachable!()
    }

    fn get(&self, _key: &[u8]) -> Option<Bytes> {
        unreachable!()
    }

    fn get_ctx(&self) -> ExecutorContext {
        unreachable!()
    }

    fn get_account(&self, _address: &H160) -> Account {
        unreachable!()
    }

    fn save_account(&mut self, _address: &H160, _account: &Account) {
        unreachable!()
    }
}
