use common_config_parser::types::spec::ChainSpec;
use common_crypto::{PrivateKey as _, Secp256k1RecoverablePrivateKey, Signature};

use protocol::types::{
    Block, Eip1559Transaction, Hasher, RichBlock, SignedTransaction, TransactionAction,
    UnsignedTransaction, UnverifiedTransaction, BASE_FEE_PER_GAS,
};

pub(crate) trait ChainSpecExt {
    //! Generate the genesis block.
    fn generate_genesis_block(&self, genesis_key: Secp256k1RecoverablePrivateKey) -> RichBlock;
}

impl ChainSpecExt for ChainSpec {
    fn generate_genesis_block(&self, _genesis_key: Secp256k1RecoverablePrivateKey) -> RichBlock {
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
    let tx = Eip1559Transaction {
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

#[allow(dead_code)]
fn build_transaction(
    priv_key: &Secp256k1RecoverablePrivateKey,
    tx: UnsignedTransaction,
    id: u64,
) -> SignedTransaction {
    let signature = priv_key.sign_message(
        &Hasher::digest(tx.encode(Some(id), None))
            .as_bytes()
            .try_into()
            .unwrap(),
    );
    let utx = UnverifiedTransaction {
        unsigned:  tx,
        signature: Some(signature.to_bytes().into()),
        chain_id:  Some(id),
        hash:      Default::default(),
    }
    .calc_hash();

    SignedTransaction::from_unverified(utx, None).unwrap()
}
