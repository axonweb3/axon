mod abi;
use abi::election_abi;

#[derive(Default)]
pub struct ElectionContract;

impl SystemContract for ElectionContract {
    fn exec_<B: Backend + ApplyBackend>(&self, backend: &mut B, tx: &SignedTransaction) -> TxResp {
        let tx = &tx.transaction.unsigned;
        let tx_data = tx.data();

        match election_abi::ElectionCalls::decode(tx_data) {
            // implement!()
        }

        TxResp {
            exit_reason:  ExitReason::Succeed(ExitSucceed::Returned),
            ret:          vec![],
            gas_used:     0u64,
            remain_gas:   tx.gas_limit().as_u64(),
            fee_cost:     U256::zero(),
            logs:         vec![],
            code_address: None,
            removed:      false,
        }
    }

}