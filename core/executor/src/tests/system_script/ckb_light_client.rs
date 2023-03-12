use std::collections::BTreeMap;
use std::str::FromStr;

use ckb_types::{packed, prelude::*};
use ethers::abi::AbiEncode;

use common_config_parser::types::ConfigRocksDB;
use protocol::types::{MemoryBackend, TxResp, H160, H256};

use crate::system_contract::ckb_light_client::{
    ckb_light_client_abi, CkbLightClientContract, CkbLightClientHandle,
};
use crate::system_contract::{init, SystemContract, HEADER_CELL_ROOT_KEY};
use crate::tests::{gen_tx, gen_vicinity};

static ROCKSDB_PATH: &str = "./free-space/system-contract";

#[test]
fn test_write_functions() {
    let vicinity = gen_vicinity();
    let mut backend = MemoryBackend::new(&vicinity, BTreeMap::new());

    let executor = CkbLightClientContract::default();
    init(ROCKSDB_PATH, ConfigRocksDB::default(), backend.clone());

    test_update_first(&mut backend, &executor);

    test_set_state(&mut backend, &executor);
}

fn prepare_header() -> packed::Header {
    let header = ckb_light_client_abi::Header::default();
    let raw = packed::RawHeader::new_builder()
        .compact_target(header.compact_target.pack())
        .dao(header.dao.pack())
        .epoch(header.epoch.pack())
        .extra_hash(header.block_hash.pack())
        .number(header.number.pack())
        .parent_hash(header.parent_hash.pack())
        .proposals_hash(header.proposals_hash.pack())
        .timestamp(header.timestamp.pack())
        .transactions_root(header.transactions_root.pack())
        .version(header.version.pack())
        .build();

    packed::Header::new_builder()
        .raw(raw)
        .nonce(header.nonce.pack())
        .build()
}

fn test_update_first(backend: &mut MemoryBackend, executor: &CkbLightClientContract) {
    let data = ckb_light_client_abi::UpdateCall {
        header: ckb_light_client_abi::Header::default(),
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    check_root(backend, executor);

    let header = CkbLightClientHandle::default().get_header_by_block_hash(&H256::default());

    if let Ok(Some(header)) = header {
        assert_eq!(header.as_bytes(), prepare_header().as_bytes())
    } else {
        panic!("header not found");
    }
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
