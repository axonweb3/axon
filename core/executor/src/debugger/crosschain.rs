use ckb_jsonrpc_types::BlockView;
use ckb_types::{h256, prelude::Pack};
use ethers_contract::decode_logs;
use ethers_core::abi::{AbiDecode, AbiEncode, RawLog};

use protocol::types::{
    Address, Eip1559Transaction, TransactionAction, H160, H256, MAX_BLOCK_GAS_LIMIT,
    MAX_PRIORITY_FEE_PER_GAS, U256,
};
use protocol::{codec::hex_decode, tokio};

use core_cross_client::crosschain_abi as abi;
use core_cross_client::{build_axon_txs, monitor::search_tx};

use crate::debugger::{clear_data, mock_signed_tx, EvmDebugger, CROSSCHAIN_CONTRACT_ADDRESS};

const CKB_BLOCK_5910757: &str = "./src/debugger/block_5910757.json";
const ACS_CODE_HASH: ckb_types::H256 =
    h256!("0x97e6179be134d47ca10322a1534d8dcb65052de7e099b5556bea924137839bab");
const REQUEST_CODE_HASH: ckb_types::H256 =
    h256!("0xd8f9afaad8eb3e26a1ef2538bac91d68635502508358ae901941513bfe2edb1d");

fn load_block() -> BlockView {
    let file = std::fs::File::options()
        .read(true)
        .open(CKB_BLOCK_5910757)
        .unwrap();

    serde_json::from_reader(file).unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn test_cross_from_ckb() {
    use common_crypto::{Secp256k1RecoverablePrivateKey, ToPublicKey, UncompressedPublicKey};

    let self_priv_key =
        hex_decode("37aa0f893d05914a4def0460c0a984d3611546cfb26924d7a7ca6e0db9950a2d").unwrap();
    let priv_key = Secp256k1RecoverablePrivateKey::try_from(self_priv_key.as_ref())
        .expect("Invalid secp private key");
    let address = Address::from_pubkey_bytes(priv_key.pub_key().to_uncompressed_bytes())
        .unwrap()
        .0;
    let to_ckb_sender =
        H160::from_slice(&hex_decode("0x421871e656E04c9A106A55CEd53Fc9A49560a424").unwrap());
    let to = H160::from_slice(&hex_decode("421871e656E04c9A106A55CEd53Fc9A49560a424").unwrap());

    let mut debugger = EvmDebugger::new(
        vec![address, to_ckb_sender],
        10000000000000000000u64.into(),
        "./free-space/db2",
    );
    debugger.init_genesis();

    let ckb_txs = search_tx(
        load_block().into(),
        &(ACS_CODE_HASH.pack()),
        &(REQUEST_CODE_HASH.pack()),
    );
    let mut ckb_tx_hash = [0u8; 32];
    ckb_tx_hash.copy_from_slice(&ckb_txs[0].hash().raw_data()[..32]);

    let (_, stx) = build_axon_txs(
        ckb_txs,
        debugger.nonce(address),
        &priv_key,
        CROSSCHAIN_CONTRACT_ADDRESS,
    );
    let resp = debugger.exec(1, vec![stx]);

    println!("{:?}", resp);

    let logs: Vec<abi::CrossFromCKBFilter> = decode_logs(&[resp.tx_resp[0]
        .logs
        .last()
        .cloned()
        .map(|l| RawLog::from((l.topics.clone(), l.data)))
        .unwrap()])
    .unwrap();

    assert_eq!(logs[0].records[0], abi::CkbtoAxonRecord {
        to,
        token_address: H160::default(),
        s_udt_amount: U256::zero(),
        ckb_amount: U256::from(450),
        tx_hash: ckb_tx_hash,
    });

    let tx = mock_signed_tx(
        build_change_limit_tx(debugger.nonce(address).as_u64()),
        address,
    );
    let _ = debugger.exec(2, vec![tx]);

    let to_ckb_tx = mock_signed_tx(build_axon_to_ckb_tx(1), to_ckb_sender);

    // let mut to_ckb_tx: SignedTransaction =
    //     UnverifiedTransaction::decode(hex_decode("02f9017105098459682f00845968397283013e6c94f67bc4e50d1df92b0e4c61794a4517af6a995cb280b90104b8f564f800000000000000000000000000000000000000000000000000000000000000600000000000000000000000004af5ec5e3d29d9ddd7f4bf91a022131c41b7235200000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000061636b7431717a646130637230386d38356863386a6c6e6670337a65723778756c656a79777434396b74327272307674687977616135307877737167797934736a6333616539647676683630306e366563716667326673717873346373646a77376500000000000000000000000000000000000000000000000000000000000000c080a06d992b3f958161bca24246caa99f3ac5a171e844a44e9feb8be96f847edbb5d7a05d5158ef716f6f4cdcb471eef6408dfd00e56167ff4809ca0abb424bb58bf90d").unwrap())
    //     .unwrap()
    //     .try_into()
    //     .unwrap();
    let resp = debugger.exec(3, vec![to_ckb_tx]);

    let logs: Vec<abi::CrossToCKBFilter> = decode_logs(&[resp.tx_resp[0]
        .logs
        .last()
        .cloned()
        .map(|l| RawLog::from((l.topics.clone(), l.data)))
        .unwrap()])
    .unwrap();

    println!("{:?}", logs);

    clear_data("./free-space");
}

// #[tokio::test(flavor = "multi_thread")]
// async fn test_crosschain() {
//     use common_crypto::{Secp256k1RecoverablePrivateKey, ToPublicKey,
// UncompressedPublicKey};

//     let self_priv_key =
//         hex_decode("
// 0x37aa0f893d05914a4def0460c0a984d3611546cfb26924d7a7ca6e0db9950a2d").
// unwrap();     let priv_key =
// Secp256k1RecoverablePrivateKey::try_from(self_priv_key.as_ref())
//         .expect("Invalid secp private key");
//     let address =
// Address::from_pubkey_bytes(priv_key.pub_key().to_uncompressed_bytes())
//         .unwrap()
//         .0;

//     let mut debugger =
//         EvmDebugger::new(address, 10000000000000000000u64.into(),
// "./free-space/db2");     debugger.init_genesis();

//     let to =
// H160::from_slice(&hex_decode("0x8ab0cf264df99d83525e9e11c7e4db01558ae1b1").
// unwrap());     let stx = mock_signed_tx(build_ckb_to_axon_tx(to), address);
//     let resp = debugger.exec(1, vec![stx]);

//     let logs: Vec<CrossFromCKBFilter> = decode_logs(
//         &resp.tx_resp[0]
//             .logs
//             .iter()
//             .skip(1)
//             .map(|l| RawLog::from((l.topics.clone(), l.data.clone())))
//             .collect::<Vec<_>>(),
//     )
//     .unwrap();

//     println!("{:?}", resp);

//     assert_eq!(
//         logs[0].records[0],
//         (
//             to,
//             H160::default(),
//             U256::zero(),
//             U256::from(100000u64),
//             H256::default().0
//         )
//     );

//     let priv_key =
// "37aa0f893d05914a4def0460c0a984d3611546cfb26924d7a7ca6e0db9950a2d";
//     let tx = mock_efficient_signed_tx(build_axon_to_ckb_tx(7), priv_key);
//     let resp = debugger.exec(2, vec![tx]);

//     println!("{:?}", resp);

//     let logs: Vec<CrossToCKBAlertFilter> = decode_logs(
//         &resp.tx_resp[0]
//             .logs
//             .iter()
//             .skip(2)
//             .map(|l| RawLog::from((l.topics.clone(), l.data.clone())))
//             .collect::<Vec<_>>(),
//     )
//     .unwrap();

//     println!("{:?}", logs);

//     let tx = mock_signed_tx(build_change_limit_tx(8), address);
//     let resp = debugger.exec(3, vec![tx]);
//     println!("{:?}", resp);

//     assert!(resp.tx_resp[0].exit_reason.is_succeed());

//     clear_data("./free-space");
// }

fn build_ckb_to_axon_tx(to_address: H160) -> Eip1559Transaction {
    let call_data = abi::CrossFromCKBCall {
        records: vec![abi::CkbtoAxonRecord {
            to:            to_address,
            token_address: H160::default(),
            s_udt_amount:  0u64.into(),
            ckb_amount:    100000u64.into(),
            tx_hash:       H256::default().0,
        }],
        nonce:   U256::zero(),
    };

    build_eip1559_tx(6, AbiEncode::encode(call_data))
}

fn build_axon_to_ckb_tx(nonce: u64) -> Eip1559Transaction {
    let data = hex_decode("0xb8f564f800000000000000000000000000000000000000000000000000000000000000600000000000000000000000004af5ec5e3d29d9ddd7f4bf91a022131c41b7235200000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000061636b7431717a646130637230386d38356863386a6c6e6670337a65723778756c656a79777434396b74327272307674687977616135307877737167797934736a6333616539647676683630306e366563716667326673717873346373646a77376500000000000000000000000000000000000000000000000000000000000000").unwrap();
    let call: abi::crosschainCalls = AbiDecode::decode(&data).unwrap();
    println!("axon to ckb call {:?}", call);
    build_eip1559_tx(nonce, data)
}

fn build_change_limit_tx(nonce: u64) -> Eip1559Transaction {
    let data = hex_decode("0x3edfdbb00000000000000000000000004af5ec5e3d29d9ddd7f4bf91a022131c41b72352000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000152d02c7e14af6800000").unwrap();
    // let _call: abi::crosschainCalls = AbiDecode::decode(&data).unwrap();
    build_eip1559_tx(nonce, data)
}

fn build_eip1559_tx(nonce: u64, data: Vec<u8>) -> Eip1559Transaction {
    Eip1559Transaction {
        nonce:                    nonce.into(),
        max_priority_fee_per_gas: MAX_PRIORITY_FEE_PER_GAS.into(),
        gas_price:                U256::one(),
        gas_limit:                MAX_BLOCK_GAS_LIMIT.into(),
        action:                   TransactionAction::Call(CROSSCHAIN_CONTRACT_ADDRESS),
        value:                    U256::zero(),
        data:                     data.into(),
        access_list:              vec![],
    }
}
