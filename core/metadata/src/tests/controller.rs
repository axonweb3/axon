use protocol::{tokio, traits::MetadataControl};

use super::*;

#[tokio::test(flavor = "multi_thread")]
async fn test_get_metadata() {
    let handle = TestHandle::new().await;
    let ctl = handle.metadata_controller(100000000);
    let res = ctl
        .get_metadata(Context::new(), &mock_header(0, handle.state_root))
        .unwrap();

    assert_eq!(res.epoch, 0);
    assert_eq!(res.version.start, 0);
    assert_eq!(res.version.end, 999999);
    assert_eq!(res.verifier_list.len(), 1);
    assert_eq!(res.interval, 3000);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_update_metadata() {
    let mut handle = TestHandle::new().await;
    handle.exec(vec![mock_signed_tx(
        5,
        mock_metadata(1, 100000000, 199999999),
    )]);
    let ctl = handle.metadata_controller(100000000);
    ctl.update_metadata(Context::new(), &mock_header(1, handle.state_root))
        .unwrap();
    // let res = ctl
    //     .get_metadata(Context::new(), &mock_header(0, handle.state_root))
    //     .unwrap();

    // assert_eq!(res.epoch, 0);
    // assert_eq!(res.version.start, 0);
    // assert_eq!(res.version.end, 999999);
    // assert_eq!(res.verifier_list.len(), 1);
    // assert_eq!(res.interval, 3000);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_need_change_metadata() {
    let handle = TestHandle::new().await;
    let ctl = handle.metadata_controller(100000000);
    assert!(ctl.need_change_metadata(100_000_000));
    assert!(!ctl.need_change_metadata(99_999_999));
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
// fn test_output_metadata() {
//     use ethers::core::abi::AbiEncode;

//     let r =
// BufReader::new(File::open("../../devtools/chain/nodes/metadata.json").
// unwrap());     let metadata: Metadata = serde_json::from_reader(r).unwrap();
//     let call =
// abi::MetadataContractCalls::AppendMetadata(abi::AppendMetadataCall {
//         metadata: metadata.into(),
//     });
//     let raw_call = call.encode();
//     println!("{:?}", raw_call);
// }

// #[test]
// fn test_abi() {
//     Abigen::new("MetadataContract", "./metadata.abi")
//         .unwrap()
//         .generate()
//         .unwrap()
//         .write_to_file("./src/metadata_abi.rs")
//         .unwrap();
// }
