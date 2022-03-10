use protocol::{tokio, traits::MetadataControl};

use super::*;

// #[test]
// fn test_2() {
// 	use ethers::core::abi::AbiEncode;

//     let r =
// BufReader::new(File::open("../../devtools/chain/metadata.json").unwrap());
//     let metadata: Metadata = serde_json::from_reader(r).unwrap();
// 	let call = abi::MetadataContractCalls::AppendMetadata(abi::AppendMetadataCall
// { 		metadata: metadata.into(),
// 	});
// 	let raw_call = call.encode();
// 	println!("{:?}", raw_call);
//  }

#[tokio::test(flavor = "multi_thread")]
async fn test_1() {
    let handle = TestHandle::new().await;
    let ctl = handle.metadata_controller(100);
    let res = ctl.get_metadata(Context::new(), &mock_header(0, handle.state_root));
    println!("{:?}", res);
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

#[test]
fn metadata_address() {
    let sender = H160::from_slice(
        &protocol::codec::hex_decode("8ab0cf264df99d83525e9e11c7e4db01558ae1b1").unwrap(),
    );
    let nonce: U256 = 4u64.into();
    let addr: H160 = core_executor::code_address(&sender, &nonce).into();
    println!("{:?}", addr)
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
