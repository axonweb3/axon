use ckb_jsonrpc_types::BlockView;
use ckb_types::{h256, prelude::Pack};
use ethers_contract::decode_logs;
use ethers_core::abi::{AbiEncode, RawLog};

use protocol::types::{
    Address, Eip1559Transaction, TransactionAction, H160, H256, MAX_BLOCK_GAS_LIMIT,
    MAX_PRIORITY_FEE_PER_GAS, U256,
};
use protocol::{codec::hex_decode, tokio};

use core_cross_client::crosschain_abi::{
    CkbtoAxonRecord, CrossFromCKBCall, CrossFromCKBFilter, CrossToCKBAlertFilter,
};
use core_cross_client::{build_axon_txs, monitor::search_tx};

use crate::debugger::{clear_data, mock_efficient_signed_tx, mock_signed_tx, EvmDebugger};
use crate::CROSSCHAIN_CONTRACT_ADDRESS;

const CKB_BLOCK_5910757: &str = "./src/debugger/block_5910757.json";
const ACS_CODE_HASH: ckb_types::H256 =
    h256!("0x97e6179be134d47ca10322a1534d8dcb65052de7e099b5556bea924137839bab");
const REQUEST_CODE_HASH: ckb_types::H256 =
    h256!("0xd8f9afaad8eb3e26a1ef2538bac91d68635502508358ae901941513bfe2edb1d");

#[tokio::test]
async fn test() {
    use protocol::traits::{CkbClient, Context};
    let client = core_rpc_client::RpcClient::new(
        "https://mercury-testnet.ckbapp.dev/rpc",
        "http://127.0.0.1:8116",
        "http://127.0.0.1:8116",
    );
    let block = client
        .get_block_by_number(Context::new(), 5910757u64.into())
        .await
        .unwrap();
    let file = std::fs::File::options()
        .create_new(true)
        .write(true)
        .open(CKB_BLOCK_5910757)
        .unwrap();
    serde_json::to_writer_pretty(file, &block).unwrap();
}

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
    let to = H160::from_slice(&hex_decode("421871e656E04c9A106A55CEd53Fc9A49560a424").unwrap());

    let mut debugger =
        EvmDebugger::new(address, 10000000000000000000u64.into(), "./free-space/db2");
    debugger.init_genesis();

    let ckb_txs = search_tx(
        load_block().into(),
        &(ACS_CODE_HASH.pack()),
        &(REQUEST_CODE_HASH.pack()),
    );
    let mut ckb_tx_hash = [0u8; 32];
    ckb_tx_hash.copy_from_slice(&ckb_txs[0].hash().raw_data()[..32]);

    let (_, stx) = build_axon_txs(ckb_txs, debugger.nonce(address), &priv_key);

    let resp = debugger.exec(1, vec![stx]);

    println!("{:?}", resp);

    let logs: Vec<CrossFromCKBFilter> = decode_logs(
        &resp.tx_resp[0]
            .logs
            .iter()
            .skip(1)
            .map(|l| RawLog::from((l.topics.clone(), l.data.clone())))
            .collect::<Vec<_>>(),
    )
    .unwrap();

    println!("{:?}", logs);

    assert_eq!(
        logs[0].records[0],
        (
            to,
            H160::default(),
            U256::zero(),
            U256::from(450),
            ckb_tx_hash
        )
    );

    clear_data("./free-space");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_crosschain() {
    use common_crypto::{Secp256k1RecoverablePrivateKey, ToPublicKey, UncompressedPublicKey};

    let self_priv_key =
        hex_decode("37aa0f893d05914a4def0460c0a984d3611546cfb26924d7a7ca6e0db9950a2d").unwrap();
    let priv_key = Secp256k1RecoverablePrivateKey::try_from(self_priv_key.as_ref())
        .expect("Invalid secp private key");
    let address = Address::from_pubkey_bytes(priv_key.pub_key().to_uncompressed_bytes())
        .unwrap()
        .0;

    let mut debugger =
        EvmDebugger::new(address, 10000000000000000000u64.into(), "./free-space/db2");
    debugger.init_genesis();

    let to = H160::from_slice(&hex_decode("8ab0cf264df99d83525e9e11c7e4db01558ae1b1").unwrap());
    let stx = mock_signed_tx(build_ckb_to_axon_txs(to), address);
    let resp = debugger.exec(1, vec![stx]);

    let logs: Vec<CrossFromCKBFilter> = decode_logs(
        &resp.tx_resp[0]
            .logs
            .iter()
            .skip(1)
            .map(|l| RawLog::from((l.topics.clone(), l.data.clone())))
            .collect::<Vec<_>>(),
    )
    .unwrap();

    assert_eq!(
        logs[0].records[0],
        (
            to,
            H160::default(),
            U256::zero(),
            U256::from(100000u64),
            H256::default().0
        )
    );

    let priv_key = "37aa0f893d05914a4def0460c0a984d3611546cfb26924d7a7ca6e0db9950a2d";
    let tx = mock_efficient_signed_tx(build_axon_to_ckb_txs(), priv_key);
    let resp = debugger.exec(2, vec![tx]);

    println!("{:?}", resp);

    let logs: Vec<CrossToCKBAlertFilter> = decode_logs(
        &resp.tx_resp[0]
            .logs
            .iter()
            .skip(2)
            .map(|l| RawLog::from((l.topics.clone(), l.data.clone())))
            .collect::<Vec<_>>(),
    )
    .unwrap();

    println!("{:?}", logs);

    clear_data("./free-space");
}

fn build_ckb_to_axon_txs(to_address: H160) -> Eip1559Transaction {
    let call_data = CrossFromCKBCall {
        records: vec![CkbtoAxonRecord {
            to:            to_address,
            token_address: H160::default(),
            s_udt_amount:  0u64.into(),
            ckb_amount:    100000u64.into(),
            tx_hash:       H256::default().0,
        }],
        nonce:   U256::zero(),
    };

    Eip1559Transaction {
        nonce:                    6u64.into(),
        max_priority_fee_per_gas: MAX_PRIORITY_FEE_PER_GAS.into(),
        gas_price:                U256::one(),
        gas_limit:                MAX_BLOCK_GAS_LIMIT.into(),
        action:                   TransactionAction::Call(CROSSCHAIN_CONTRACT_ADDRESS),
        value:                    U256::zero(),
        data:                     AbiEncode::encode(call_data).into(),
        access_list:              vec![],
    }
}

fn build_axon_to_ckb_txs() -> Eip1559Transaction {
    let data = hex_decode("db2b749f000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000076466657266776500000000000000000000000000000000000000000000000000").unwrap();

    Eip1559Transaction {
        nonce:                    7u64.into(),
        max_priority_fee_per_gas: MAX_PRIORITY_FEE_PER_GAS.into(),
        gas_price:                U256::one(),
        gas_limit:                MAX_BLOCK_GAS_LIMIT.into(),
        action:                   TransactionAction::Call(CROSSCHAIN_CONTRACT_ADDRESS),
        value:                    100000000000000000u64.into(),
        data:                     data.into(),
        access_list:              vec![],
    }
}
