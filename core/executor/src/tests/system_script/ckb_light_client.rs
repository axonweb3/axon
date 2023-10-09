use std::collections::BTreeMap;
use std::str::FromStr;

use ethers::abi::AbiEncode;

use core_db::RocksAdapter;
use protocol::types::{Backend, MemoryBackend, TxResp, H160, H256, U256};

use crate::system_contract::ckb_light_client::{
    ckb_light_client_abi, CkbHeaderReader, CkbLightClientContract,
};
use crate::system_contract::{
    init_system_contract_db, SystemContract, CKB_LIGHT_CLIENT_CONTRACT_ADDRESS,
    HEADER_CELL_ROOT_KEY, IMAGE_CELL_CONTRACT_ADDRESS,
};
use crate::tests::{gen_tx, gen_vicinity};

static ROCKSDB_PATH: &str = "./free-space/system-contract/ckb-light-client";

pub fn test_write_functions() {
    let vicinity = gen_vicinity();
    let mut backend = MemoryBackend::new(&vicinity, BTreeMap::new());

    let executor = CkbLightClientContract::default();
    let inner_db = RocksAdapter::new(ROCKSDB_PATH, Default::default())
        .unwrap()
        .inner_db();
    init_system_contract_db(inner_db, &mut backend);

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
        extra_hash:        [0u8; 32],
        extension:         Default::default(),
    }
}

fn test_update_first<'a>(
    backend: &mut MemoryBackend<'a>,
    executor: &CkbLightClientContract<MemoryBackend<'a>>,
) {
    let header = prepare_header_1();
    let data = ckb_light_client_abi::UpdateCall {
        headers: vec![header.clone()],
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    check_nonce(backend, 1);

    let root = backend.storage(CKB_LIGHT_CLIENT_CONTRACT_ADDRESS, *HEADER_CELL_ROOT_KEY);
    let queried_header = CkbHeaderReader
        .get_header_by_block_hash(root, &H256::default())
        .unwrap()
        .unwrap();

    assert_eq!(queried_header, header);
}

fn test_update_second<'a>(
    backend: &mut MemoryBackend<'a>,
    executor: &CkbLightClientContract<MemoryBackend<'a>>,
) {
    let header = prepare_header_2();
    let data = ckb_light_client_abi::UpdateCall {
        headers: vec![header.clone()],
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    check_nonce(backend, 2);

    let root = backend.storage(CKB_LIGHT_CLIENT_CONTRACT_ADDRESS, *HEADER_CELL_ROOT_KEY);
    let queried_header = CkbHeaderReader
        .get_header_by_block_hash(root, &H256::from_slice(&header.block_hash))
        .unwrap()
        .unwrap();

    assert_eq!(queried_header, header);
}

fn test_roll_back_first<'a>(
    backend: &mut MemoryBackend<'a>,
    executor: &CkbLightClientContract<MemoryBackend<'a>>,
) {
    let data = ckb_light_client_abi::RollbackCall {
        block_hashes: vec![prepare_header_2().block_hash],
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    let root = backend.storage(CKB_LIGHT_CLIENT_CONTRACT_ADDRESS, *HEADER_CELL_ROOT_KEY);
    let queried_header = CkbHeaderReader
        .get_header_by_block_hash(root, &H256::default())
        .unwrap()
        .unwrap();

    assert_eq!(queried_header, prepare_header_1());
}

fn test_roll_back_second<'a>(
    backend: &mut MemoryBackend<'a>,
    executor: &CkbLightClientContract<MemoryBackend<'a>>,
) {
    let data = ckb_light_client_abi::RollbackCall {
        block_hashes: vec![prepare_header_1().block_hash],
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    let root = backend.storage(CKB_LIGHT_CLIENT_CONTRACT_ADDRESS, *HEADER_CELL_ROOT_KEY);
    let queried_header = CkbHeaderReader
        .get_header_by_block_hash(root, &H256::default())
        .unwrap();
    assert!(queried_header.is_none());
}

fn test_set_state<'a>(
    backend: &mut MemoryBackend<'a>,
    executor: &CkbLightClientContract<MemoryBackend<'a>>,
) {
    let data = ckb_light_client_abi::SetStateCall { allow_read: true };
    let querier = CkbHeaderReader;

    assert!(!querier.allow_read());

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    assert!(querier.allow_read());
}

fn exec<'a>(
    backend: &mut MemoryBackend<'a>,
    executor: &CkbLightClientContract<MemoryBackend<'a>>,
    data: Vec<u8>,
) -> TxResp {
    let addr = H160::from_str("0xf000000000000000000000000000000000000000").unwrap();
    let tx = gen_tx(addr, CKB_LIGHT_CLIENT_CONTRACT_ADDRESS, 1000, data);
    executor.exec_(backend, &tx)
}

fn check_nonce(backend: &mut MemoryBackend<'_>, nonce: u64) {
    assert_eq!(
        backend.basic(CKB_LIGHT_CLIENT_CONTRACT_ADDRESS).nonce,
        U256::zero()
    );
    assert_eq!(
        backend.basic(IMAGE_CELL_CONTRACT_ADDRESS).nonce,
        U256::zero()
    );
    assert_eq!(
        backend
            .basic(H160::from_str("0xf000000000000000000000000000000000000000").unwrap())
            .nonce,
        nonce.into()
    )
}
