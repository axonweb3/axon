use common_config_parser::types::spec::ChainSpec;

use protocol::types::{
    Block, Eip1559Transaction, RichBlock, TransactionAction, UnsignedTransaction, BASE_FEE_PER_GAS,
};

pub(crate) trait ChainSpecExt {
    //! Generate the genesis block.
    fn generate_genesis_block(&self) -> RichBlock;
}

impl ChainSpecExt for ChainSpec {
    fn generate_genesis_block(&self) -> RichBlock {
        let txs = vec![];
        let block = Block {
            header:    self.genesis.build_header(),
            tx_hashes: vec![],
        };

        RichBlock { block, txs }
    }
}

#[allow(dead_code)]
fn build_unverified_transaction(
    nonce: u64,
    action: TransactionAction,
    data: Vec<u8>,
) -> UnsignedTransaction {
    let tx =
        Eip1559Transaction {
            nonce: nonce.into(),
            max_priority_fee_per_gas: BASE_FEE_PER_GAS.into(),
            gas_price: 0u64.into(),
            gas_limit: 30000000u64.into(),
            value: 0u64.into(),
            data: data.into(),
            access_list: vec![],
            action,
        };
    UnsignedTransaction::Eip1559(tx)
}
