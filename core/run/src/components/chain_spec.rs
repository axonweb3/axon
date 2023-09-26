use ethers_core::abi::AbiEncode;

use common_config_parser::types::spec::ChainSpec;
use common_crypto::{PrivateKey as _, Secp256k1RecoverablePrivateKey, Signature};
use core_executor::system_contract::{
    metadata::metadata_abi::{AppendMetadataCall, MetadataContractCalls},
    METADATA_CONTRACT_ADDRESS,
};

use protocol::types::{
    Block, Eip1559Transaction, Hasher, Metadata, RichBlock, SignedTransaction, TransactionAction,
    UnsignedTransaction, UnverifiedTransaction, BASE_FEE_PER_GAS,
};

pub(crate) trait ChainSpecExt {
    //! Generate the genesis block.
    fn generate_genesis_block(&self, genesis_key: Secp256k1RecoverablePrivateKey) -> RichBlock;
}

impl ChainSpecExt for ChainSpec {
    fn generate_genesis_block(&self, genesis_key: Secp256k1RecoverablePrivateKey) -> RichBlock {
        let metadata_0 = self.params.clone();
        let metadata_1 = {
            let mut tmp = metadata_0.clone();
            tmp.epoch = metadata_0.epoch + 1;
            tmp.version.start = metadata_0.version.end + 1;
            tmp.version.end = tmp.version.start + metadata_0.version.end - 1;
            tmp
        };
        let data_0 = encode_metadata(metadata_0);
        let data_1 = encode_metadata(metadata_1);

        let chain_id = self.genesis.chain_id;

        let txs: Vec<_> = [data_0, data_1]
            .into_iter()
            .enumerate()
            .map(|(index, data)| {
                let nonce = index as u64;
                let action = TransactionAction::Call(METADATA_CONTRACT_ADDRESS);
                let utx = build_unverified_transaction(nonce, action, data);
                build_transaction(&genesis_key, utx, chain_id)
            })
            .collect();

        let header = self.genesis.build_header();
        let tx_hashes = txs.iter().map(|tx| tx.transaction.hash).collect::<Vec<_>>();
        let block = Block { header, tx_hashes };

        RichBlock { block, txs }
    }
}

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

fn encode_metadata(metadata: Metadata) -> Vec<u8> {
    MetadataContractCalls::AppendMetadata(AppendMetadataCall {
        metadata: metadata.into(),
    })
    .encode()
}
