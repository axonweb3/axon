use std::collections::HashMap;

use ckb_jsonrpc_types::TransactionWithStatus;
use ckb_types::packed::Transaction;
use ed25519_dalek::Signer;
use jsonrpsee_core::client::ClientT;
use jsonrpsee_http_client::{types::params::ParamsSer, HttpClientBuilder};
use serde_json::json;

use protocol::{codec::hex_decode, tokio, traits::Interoperation, types::H256};

use crate::{init_dispatcher, InteroperationImpl};

const MAX_CYCLES: u64 = 100_000_000;

#[ignore = "use 47.111.84.118:81 to replace 127.0.0.1 when manually running"]
#[tokio::test]
async fn test_ckb_ed25519() {
    // fetch contract binary via rpc client
    let client = HttpClientBuilder::default()
        .build("http://127.0.0.1")
        .unwrap();
    let tx_hash = {
        let bytes =
            hex_decode("ed5c4bfebf7e9d842122428380599146dffd2906a54296e568fc7d4833e21479").unwrap();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(bytes.as_slice());
        H256::from(hash)
    };
    let value: Option<TransactionWithStatus> = client
        .request(
            "get_transaction",
            Some(ParamsSer::Array(vec![json!(tx_hash)])),
        )
        .await
        .expect("rpc get_transaction");
    let ed25519_bin = {
        let tx = {
            let tx = value.unwrap();
            Transaction::from(tx.transaction.unwrap().inner).into_view()
        };
        let (_, ed25519_bin) = tx.output_with_data(0).unwrap();
        ed25519_bin
    };

    // setup contract binary
    let mut map = HashMap::new();
    map.insert(tx_hash, ed25519_bin);
    init_dispatcher(map).unwrap();

    // test main logic
    let mut csprng = rand::rngs::ThreadRng::default();
    let keypair = ed25519_dalek::Keypair::generate(&mut csprng);
    let message = vec![0u8; 32];
    let signature = keypair.sign(&message).to_bytes().to_vec();
    let public_key = keypair.public.to_bytes().to_vec();
    let args = vec![message.into(), signature.into(), public_key.into()];

    let result = InteroperationImpl::default()
        .call_ckb_vm(Default::default(), tx_hash, &args, MAX_CYCLES)
        .expect("vm");
    assert!(result.exit_code == 0);
}
