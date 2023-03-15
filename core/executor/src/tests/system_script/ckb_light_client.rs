use std::collections::BTreeMap;
use std::str::FromStr;

use ckb_types::{packed, prelude::*};
use protocol::types::{Backend, MemoryBackend, TxResp, H160, H256, U256};

use common_config_parser::types::ConfigRocksDB;
use ethers::abi::AbiEncode;

use crate::system_contract::ckb_light_client::{ckb_light_client_abi, CkbLightClientContract};
use crate::system_contract::{init, ImageCellContract, SystemContract, HEADER_CELL_ROOT_KEY};
use crate::tests::{gen_tx, gen_vicinity};

static ROCKSDB_PATH: &str = "./free-space/system-contract";

#[test]
fn test_write_functions() {
    let vicinity = gen_vicinity();
    let mut backend = MemoryBackend::new(&vicinity, BTreeMap::new());

    let executor = CkbLightClientContract::default();
    init(ROCKSDB_PATH, ConfigRocksDB::default(), backend.clone());

    // need to refactor to be OO
    test_update_first(&mut backend, &executor);
    test_update_second(&mut backend, &executor);

    test_roll_back_first(&mut backend, &executor);
    test_roll_back_second(&mut backend, &executor);

    test_set_state(&mut backend, &executor);
}

fn prepare_header_1() -> ckb_light_client_abi::Header {
    ckb_light_client_abi::Header::default()
}

fn prepare_header_2() -> ckb_light_client_abi::Header {
    ckb_light_client_abi::Header {
        compact_target:    0x1,
        dao:               [1u8; 32],
        epoch:             1u64,
        block_hash:        [1u8; 32],
        number:            0x1,
        parent_hash:       [1u8; 32],
        proposals_hash:    [1u8; 32],
        timestamp:         0x1,
        transactions_root: [1u8; 32],
        version:           0x1,
        nonce:             0x1,
        uncles_hash:       [0u8; 32],
    }
}

fn test_update_first(backend: &mut MemoryBackend, executor: &CkbLightClientContract) {
    let header = prepare_header_1();
    let data = ckb_light_client_abi::UpdateCall {
        headers: vec![header.clone()],
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    check_root(backend, executor);
    check_nounce(backend, U256::one());

    let queried_header = executor
        .get_header_by_block_hash(&H256::default())
        .unwrap()
        .unwrap();

    assert_eq!(
        queried_header.as_bytes(),
        packed::Header::from(header).as_bytes()
    );
}

fn test_update_second(backend: &mut MemoryBackend, executor: &CkbLightClientContract) {
    let header = prepare_header_2();
    let data = ckb_light_client_abi::UpdateCall {
        headers: vec![header.clone()],
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    check_root(backend, executor);
    check_nounce(backend, U256::from(2));

    let queried_header = executor
        .get_header_by_block_hash(&H256::from_slice(&header.block_hash))
        .unwrap()
        .unwrap();

    assert_eq!(
        queried_header.as_bytes(),
        packed::Header::from(header).as_bytes()
    );
}

fn test_roll_back_first(backend: &mut MemoryBackend, executor: &CkbLightClientContract) {
    let data = ckb_light_client_abi::RollbackCall {
        block_hashes: vec![prepare_header_2().block_hash],
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    check_root(backend, executor);

    let queried_header = executor
        .get_header_by_block_hash(&H256::default())
        .unwrap()
        .unwrap();
    assert_eq!(
        queried_header.as_bytes(),
        packed::Header::from(prepare_header_1()).as_bytes()
    );
}

fn test_roll_back_second(backend: &mut MemoryBackend, executor: &CkbLightClientContract) {
    let data = ckb_light_client_abi::RollbackCall {
        block_hashes: vec![prepare_header_1().block_hash],
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    check_root(backend, executor);

    let queried_header = executor.get_header_by_block_hash(&H256::default()).unwrap();
    assert!(queried_header.is_none());
}

fn test_set_state(backend: &mut MemoryBackend, executor: &CkbLightClientContract) {
    let data = ckb_light_client_abi::SetStateCall { allow_read: true };

    assert!(!executor.allow_read());

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    assert!(executor.allow_read());
}

fn exec(backend: &mut MemoryBackend, executor: &CkbLightClientContract, data: Vec<u8>) -> TxResp {
    let addr = H160::from_str("0xf000000000000000000000000000000000000000").unwrap();
    let tx = gen_tx(addr, CkbLightClientContract::ADDRESS, 1000, data);
    executor.exec_(backend, &tx)
}

fn check_root(backend: &MemoryBackend, executor: &CkbLightClientContract) {
    let account = backend
        .state()
        .get(&CkbLightClientContract::ADDRESS)
        .unwrap();
    assert_eq!(
        &executor.get_root(),
        account.storage.get(&HEADER_CELL_ROOT_KEY).unwrap(),
    );
}

fn check_nounce(backend: &mut MemoryBackend, nounce: U256) {
    let ckb_account = backend.basic(CkbLightClientContract::ADDRESS);
    let ic_account = backend.basic(ImageCellContract::ADDRESS);
    assert_eq!(ckb_account.nonce, nounce);
    assert_eq!(ic_account.nonce, U256::zero());
}
