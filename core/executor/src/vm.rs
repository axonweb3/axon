use std::collections::BTreeMap;

use evm::executor::stack::{MemoryStackState, PrecompileFn, StackExecutor, StackSubstateMetadata};

use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{
    Config, Hasher, SignedTransaction, TransactionAction, TxResp, H160, H256, U256,
};

pub const METADATA_CONTRACT_ADDRESS: H160 = H160([
    74, 245, 236, 94, 61, 41, 217, 221, 215, 244, 191, 145, 160, 34, 19, 28, 65, 183, 35, 82,
]);
pub const WCKB_CONTRACT_ADDRESS: H160 = H160([
    176, 13, 97, 107, 130, 12, 57, 97, 158, 226, 158, 81, 68, 208, 34, 108, 248, 181, 193, 90,
]);
pub const CROSSCHAIN_CONTRACT_ADDRESS: H160 = H160([
    185, 220, 139, 222, 29, 180, 36, 16, 211, 4, 181, 231, 140, 47, 248, 67, 19, 78, 21, 224,
]);

#[derive(Default)]
pub struct EvmExecutor;

impl EvmExecutor {
    pub fn new() -> Self {
        EvmExecutor::default()
    }

    pub fn inner_exec<B: Backend + ApplyBackend>(
        &self,
        backend: &mut B,
        config: &Config,
        precompiles: &BTreeMap<H160, PrecompileFn>,
        tx: SignedTransaction,
    ) -> TxResp {
        // let old_nonce = backend.basic(tx.sender).nonce;
        let metadata =
            StackSubstateMetadata::new(tx.transaction.unsigned.gas_limit().as_u64(), config);
        let mut executor = StackExecutor::new_with_precompiles(
            MemoryStackState::new(metadata, backend),
            config,
            precompiles,
        );
        let (exit_reason, ret) = match tx.transaction.unsigned.action() {
            TransactionAction::Call(addr) => executor.transact_call(
                tx.sender,
                *addr,
                *tx.transaction.unsigned.value(),
                tx.transaction.unsigned.data().to_vec(),
                tx.transaction.unsigned.gas_limit().as_u64(),
                tx.transaction
                    .unsigned
                    .access_list()
                    .into_iter()
                    .map(|x| (x.address, x.storage_keys))
                    .collect(),
            ),
            TransactionAction::Create => executor.transact_create(
                tx.sender,
                *tx.transaction.unsigned.value(),
                tx.transaction.unsigned.data().to_vec(),
                tx.transaction.unsigned.gas_limit().as_u64(),
                tx.transaction
                    .unsigned
                    .access_list()
                    .into_iter()
                    .map(|x| (x.address, x.storage_keys))
                    .collect(),
            ),
        };

        let remain_gas = executor.gas();
        let gas_used = executor.used_gas();
        let (values, logs) = executor.into_state().deconstruct();
        backend.apply(values, logs, true);

        let code_address = if tx.transaction.unsigned.action() == &TransactionAction::Create
            && exit_reason.is_succeed()
        {
            Some(code_address(&tx.sender, tx.transaction.unsigned.nonce()))
        } else {
            None
        };

        TxResp {
            exit_reason,
            ret,
            remain_gas,
            gas_used,
            logs: vec![],
            code_address,
            removed: false,
        }
    }
}

pub fn code_address(sender: &H160, nonce: &U256) -> H256 {
    let mut stream = rlp::RlpStream::new_list(2);
    stream.append(sender);
    stream.append(nonce);
    Hasher::digest(&stream.out())
}

#[cfg(test)]
mod tests {
    use protocol::codec::{hex_decode, hex_encode};

    use super::*;

    #[test]
    fn test_code_address() {
        let sender = H160::from_slice(
            hex_decode("8ab0cf264df99d83525e9e11c7e4db01558ae1b1")
                .unwrap()
                .as_ref(),
        );
        let nonce: U256 = 0u64.into();
        let addr: H160 = code_address(&sender, &nonce).into();
        assert_eq!(
            hex_encode(addr.0).as_str(),
            "a13763691970d9373d4fab7cc323d7ba06fa9986"
        );

        let sender = H160::from_slice(
            hex_decode("6ac7ea33f8831ea9dcc53393aaa88b25a785dbf0")
                .unwrap()
                .as_ref(),
        );
        let addr: H160 = code_address(&sender, &nonce).into();
        assert_eq!(
            hex_encode(addr.0).as_str(),
            "cd234a471b72ba2f1ccf0a70fcaba648a5eecd8d"
        )
    }
}
