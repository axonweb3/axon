use std::collections::HashMap;

use cardano_message_signing::{self as MS, utils::ToBytes};
use cardano_serialization_lib as Cardano;
use core_rpc_client::RpcClient;
use ed25519_dalek::Keypair;
use rand::rngs::ThreadRng;

use protocol::{codec::hex_decode, tokio, traits::Interoperation, types::H256};

use crate::{get_ckb_transaction_hash, InteroperationImpl};

const MAX_CYCLES: u64 = 100_000_000;

async fn init_interoperation_handler(
    transaction_hash_map: HashMap<u8, H256>,
) -> InteroperationImpl {
    let rpc_client = RpcClient::new("http://127.0.0.1:8114", "http://127.0.0.1:8116");
    InteroperationImpl::new(transaction_hash_map, rpc_client)
        .await
        .unwrap()
}

fn parse_h256(hex_str: &str) -> H256 {
    let bytes = hex_decode(hex_str).unwrap();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(bytes.as_slice());
    hash.into()
}

#[ignore]
#[tokio::test]
async fn test_ckb_cardano() {
    // fetch contract binary via rpc client
    let tx_hash = parse_h256("b1af175009413bf9670dffb7b120f0eca52896a9798bda123df9b25ff7d8f721");
    let mut transaction_hash_map = HashMap::new();
    transaction_hash_map.insert(2u8, tx_hash);
    let handler = init_interoperation_handler(transaction_hash_map).await;
    assert!(tx_hash == get_ckb_transaction_hash(2u8).unwrap());

    // generate random keypair
    let mut csprng = ThreadRng::default();
    let keypair = Keypair::generate(&mut csprng);
    let payload = vec![1u8; 32]; // represent axon transaction hash

    // generate Cardano singing key
    let private_key =
        Cardano::crypto::PrivateKey::from_normal_bytes(keypair.secret.to_bytes().as_ref()).unwrap();
    let public_key = private_key.to_public();
    let mut address = vec![2u8; 57];

    // generate signing message
    let mut protected_headers = MS::HeaderMap::new();
    protected_headers.set_algorithm_id(&MS::Label::from_algorithm_id(
        MS::builders::AlgorithmId::EdDSA,
    ));
    protected_headers
        .set_header(
            &MS::Label::new_text("address".into()),
            &MS::cbor::CBORValue::new_bytes(address.clone()),
        )
        .unwrap();
    let protected_serialized = MS::ProtectedHeaderMap::new(&protected_headers);
    let unprotected = MS::HeaderMap::new();
    let headers = MS::Headers::new(&protected_serialized, &unprotected);
    let builder = MS::builders::COSESign1Builder::new(&headers, payload.clone(), false);
    let message = builder.make_data_to_sign().to_bytes();

    // verify signature by Cardano self
    let signature = private_key.sign(message.as_slice());
    assert!(public_key.verify(&message, &signature));

    // run ckv-vm
    let mut pubkey_plus_address = public_key.as_bytes().to_vec();
    pubkey_plus_address.append(&mut address);
    let args = vec![
        payload.into(),
        signature.to_bytes().into(),
        pubkey_plus_address.into(),
    ];
    let result = handler
        .call_ckb_vm(Default::default(), tx_hash, &args, MAX_CYCLES)
        .expect("vm");
    assert!(result.exit_code == 0);
}
