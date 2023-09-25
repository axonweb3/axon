use ethers_core::abi::AbiEncode;

use common_config_parser::types::spec::ChainSpec;
use common_crypto::{PrivateKey as _, Secp256k1RecoverablePrivateKey, Signature};
use core_executor::system_contract::metadata::metadata_abi::{
    AppendMetadataCall, MetadataContractCalls,
};

use protocol::types::{
    Hasher, Metadata, RichBlock, SignedTransaction, UnsignedTransaction, UnverifiedTransaction,
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

        let mut genesis = self.genesis.build_rich_block();
        for (idx, tx) in genesis.txs.iter_mut().enumerate() {
            let mut utx = tx.transaction.unsigned.clone();

            if idx == 0 {
                utx.set_data(data_0.clone().into());
            } else if idx == 1 {
                utx.set_data(data_1.clone().into())
            }

            let new_tx = build_transaction(&genesis_key, utx, genesis.block.header.chain_id);
            *tx = new_tx;
        }

        let hashes = genesis
            .txs
            .iter()
            .map(|tx| tx.transaction.hash)
            .collect::<Vec<_>>();
        genesis.block.tx_hashes = hashes;

        genesis
    }
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
