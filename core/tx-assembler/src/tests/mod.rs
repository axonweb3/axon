use std::sync::Arc;

use ckb_jsonrpc_types::{Transaction as JsonTx, TransactionView as JsonTxView};
use ckb_types::{core::Capacity, h256};

use common_crypto::{
    BlsPrivateKey, BlsPublicKey, BlsSignature, HashValue, PrivateKey, ToBlsPublicKey,
};
use protocol::traits::CkbClient;
use protocol::types::{crosschain, H160, H256};
use protocol::{tokio, traits::TxAssembler};

use core_rpc_client::RpcClient;

use crate::{IndexerAdapter, TxAssemblerImpl};

const INDEXER_URL: &str = "http://47.111.84.118:81/indexer";
const METADATA_TYPEID_ARGS: ckb_types::H256 =
    h256!("0x490d951fe6d4d34d0c4f238b50b8b1d524ddf737275b1a1f1e3216f0af5c522e");
const STAKE_TYPEID_ARGS: ckb_types::H256 =
    h256!("0x0000000000000000000000000000000000000000000000000000000000000000");
const METADATA_TYPEID: ckb_types::H256 =
    h256!("0xedc5d491da94ef638eefec43372a293879c518dbb4af3be0766ce6806befa3ec");
const RECEIVE_ADDRESS: &str = "ckt1qyqy76t2hhemukpjsa6aue37q7fyzgkneuhswnd2pa";

fn gen_sig_pubkeys(size: usize, hash: &H256) -> (BlsSignature, Vec<BlsPublicKey>) {
    let mut sigs = vec![];
    let mut pubkeys = vec![];
    for _i in 0..size {
        let bls_priv_key = BlsPrivateKey::generate(&mut rand::rngs::OsRng);
        let bls_pub_key = bls_priv_key.pub_key(&String::new());

        let sig =
            bls_priv_key.sign_message(&HashValue::from_bytes_unchecked(hash.to_fixed_bytes()));
        pubkeys.push(bls_pub_key.clone());
        sigs.push((sig, bls_pub_key));
    }
    let sig = BlsSignature::combine(sigs).expect("bls combine");
    (sig, pubkeys)
}

fn adapter() -> Arc<IndexerAdapter<RpcClient>> {
    Arc::new(IndexerAdapter::new(Arc::new(RpcClient::new(
        "http://47.111.84.118:81/",
        "http://127.0.0.1:8116",
        INDEXER_URL,
    ))))
}

#[ignore]
#[tokio::test]
async fn test_acs_complete_transacion() {
    let transfer = crosschain::Transfer {
        direction:     crosschain::Direction::FromAxon,
        ckb_address:   RECEIVE_ADDRESS.into(),
        address:       H160::default(),
        ckb_amount:    Capacity::bytes(85).unwrap().as_u64(),
        erc20_address: H160::default(),
        sudt_amount:   0,
        tx_hash:       H256::default(),
    };
    let acs = TxAssemblerImpl::new(adapter());
    let metadata_typeid_args = H256::from_slice(METADATA_TYPEID_ARGS.as_bytes());
    let stake_typeid_args = H256::from_slice(STAKE_TYPEID_ARGS.as_bytes());
    let typeid = acs
        .update_metadata(metadata_typeid_args, stake_typeid_args, 5, true)
        .await
        .expect("update metadata");
    println!("typeid = {}", typeid);
    assert!(typeid == H256::from_slice(METADATA_TYPEID.as_bytes()));
    let digest = acs
        .generate_crosschain_transaction_digest(Default::default(), &[transfer])
        .await
        .expect("generate digest")
        .hash()
        .raw_data();

    let (signature, pubkeys) = gen_sig_pubkeys(3, &H256::from_slice(&digest));
    let tx = acs
        .complete_crosschain_transaction(
            Default::default(),
            H256::from_slice(&digest),
            &signature,
            &pubkeys,
        )
        .unwrap();
    assert!(digest == tx.hash().raw_data());
    println!(
        "[with signatures] tx = {}",
        serde_json::to_string_pretty(&JsonTxView::from(tx.clone())).unwrap()
    );
    let rpc = RpcClient::new(
        "http://47.111.84.118:81/",
        "http://127.0.0.1:8116",
        INDEXER_URL,
    );
    let result = rpc
        .send_transaction(Default::default(), &JsonTx::from(tx.data()), None)
        .await
        .expect("send ckb");
    println!("result = {}", result);
}
