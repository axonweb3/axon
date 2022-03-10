use protocol::tokio;

use super::*;

// #[test]
// fn test() {
// 	use ethers::core::abi::AbiEncode;

//     let r =
// BufReader::new(File::open("../../devtools/chain/metadata.json").unwrap());
//     let metadata: Metadata = serde_json::from_reader(r).unwrap();
// 	let call = abi::MetadataContractCalls::AppendMetadata(abi::AppendMetadataCall
// { 		metadata: metadata.into(),
// 	});
// 	let raw_call = call.encode();
// 	println!("{:?}", raw_call);

// 	use protocol::types::{TransactionAction, Hex};

// 	let ac = TransactionAction::Call(H256::from_slice(&Hex::decode("
// 0xc34393e6a797d2b4e2aabbc7b9dc8bde1db42410d304b5e78c2ff843134e15e0".
// to_string()).unwrap()).into()); 	println!("{:?}",
// serde_json::to_string_pretty(&ac).unwrap()); }

#[tokio::test(flavor = "multi_thread")]
async fn test_1() {
    let handle = TestHandle::new().await;
    let _ctl = handle.metadata_controller(100);
}

#[test]
fn test_calc_epoch() {
    EPOCH_LEN.swap(Arc::new(100u64));

    assert_eq!(calc_epoch(1), 0);
    assert_eq!(calc_epoch(99), 0);
    assert_eq!(calc_epoch(100), 1);
    assert_eq!(calc_epoch(101), 1);
    assert_eq!(calc_epoch(200), 2);
}

// #[test]
// fn test_abi() {
//     Abigen::new("MetadataContract", "./metadata.abi")
//         .unwrap()
//         .generate()
//         .unwrap()
//         .write_to_file("./src/metadata_abi.rs")
//         .unwrap();
// }
