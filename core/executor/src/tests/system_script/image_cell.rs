use ckb_types::{bytes::Bytes, packed, prelude::*};
use ethers::abi::AbiEncode;

use common_config_parser::types::ConfigRocksDB;
use protocol::types::{Hasher, TxResp};

use crate::system_contract::image_cell::{
    cell_key, header_key, image_cell_abi, init, CellInfo, ImageCellContract,
};
use crate::system_contract::SystemContract;

use super::*;

static ROCKDB_PATH: &str = "./free-space/image-cell";

lazy_static::lazy_static! {
    static ref CELL_ROOT_KEY: H256 = Hasher::digest("cell_mpt_root");
}

#[test]
fn test_write_functions() {
    let vicinity = gen_vicinity();
    let mut backend = MemoryBackend::new(&vicinity, BTreeMap::new());

    let executor = ImageCellContract::default();
    init(ROCKDB_PATH, ConfigRocksDB::default(), 100);

    test_update_first(&mut backend, &executor);
    test_update_second(&mut backend, &executor);

    test_rollback_first(&mut backend, &executor);
    test_rollback_second(&mut backend, &executor);

    test_set_state(&mut backend, &executor);
}

fn test_update_first(backend: &mut MemoryBackend, executor: &ImageCellContract) {
    let data = image_cell_abi::UpdateCall {
        header:  prepare_header(),
        inputs:  vec![],
        outputs: prepare_outputs(),
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    check_root(backend, executor);

    let header_key = header_key(&[5u8; 32], 0x1);
    let get_header = executor.get_header(backend, &header_key).unwrap().unwrap();
    check_header(&get_header);

    let cell_key = cell_key(&[7u8; 32], 0x0);
    let get_cell = executor.get_cell(backend, &cell_key).unwrap().unwrap();
    check_cell(&get_cell, 0x1, None);
}

fn test_update_second(backend: &mut MemoryBackend, executor: &ImageCellContract) {
    let data = image_cell_abi::UpdateCall {
        header:  prepare_header_2(),
        inputs:  vec![image_cell_abi::OutPoint {
            tx_hash: [7u8; 32],
            index:   0x0,
        }],
        outputs: vec![],
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    check_root(backend, executor);

    let cell_key = cell_key(&[7u8; 32], 0x0);
    let get_cell = executor.get_cell(backend, &cell_key).unwrap().unwrap();
    check_cell(&get_cell, 0x1, Some(0x2));
}

fn test_rollback_first(backend: &mut MemoryBackend, executor: &ImageCellContract) {
    let data = image_cell_abi::RollbackCall {
        block_hash:   [5u8; 32],
        block_number: 0x2,
        inputs:       vec![image_cell_abi::OutPoint {
            tx_hash: [7u8; 32],
            index:   0x0,
        }],
        outputs:      vec![],
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    check_root(backend, executor);

    let header_key = header_key(&[5u8; 32], 0x2);
    let get_header = executor.get_header(backend, &header_key).unwrap();
    assert!(get_header.is_none());

    let cell_key = cell_key(&[7u8; 32], 0x0);
    let get_cell = executor.get_cell(backend, &cell_key).unwrap().unwrap();
    check_cell(&get_cell, 0x1, None);
}

fn test_rollback_second(backend: &mut MemoryBackend, executor: &ImageCellContract) {
    let data = image_cell_abi::RollbackCall {
        block_hash:   [5u8; 32],
        block_number: 0x1,
        inputs:       vec![],
        outputs:      vec![image_cell_abi::OutPoint {
            tx_hash: [7u8; 32],
            index:   0x0,
        }],
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    check_root(backend, executor);

    let cell_key = cell_key(&[7u8; 32], 0x0);
    let get_cell = executor.get_cell(backend, &cell_key).unwrap();
    assert!(get_cell.is_none());
}

fn test_set_state(backend: &mut MemoryBackend, executor: &ImageCellContract) {
    let data = image_cell_abi::SetStateCall { allow_read: true };

    assert!(!executor.allow_read());

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    assert!(executor.allow_read());
}

fn exec(backend: &mut MemoryBackend, executor: &ImageCellContract, data: Vec<u8>) -> TxResp {
    let addr = H160::from_str("0xf000000000000000000000000000000000000000").unwrap();
    let tx = gen_tx(addr, ImageCellContract::ADDRESS, 1000, data);
    executor.exec_(backend, &tx)
}

fn check_root(backend: &MemoryBackend, executor: &ImageCellContract) {
    let account = backend.state().get(&ImageCellContract::ADDRESS).unwrap();
    assert_eq!(
        &executor.get_root(backend),
        account.storage.get(&CELL_ROOT_KEY).unwrap(),
    );
}

fn check_header(get_header: &packed::Header) {
    let header = prepare_header();

    let nonce: packed::Uint128 = header.nonce.pack();
    assert_eq!(get_header.nonce().raw_data(), nonce.raw_data());

    let get_header = get_header.raw();

    assert_eq!(get_header.dao(), header.dao.pack());
    assert_eq!(get_header.extra_hash(), header.block_hash.pack());
    assert_eq!(get_header.parent_hash(), header.parent_hash.pack());
    assert_eq!(get_header.proposals_hash(), header.proposals_hash.pack());
    assert_eq!(
        get_header.transactions_root(),
        header.transactions_root.pack()
    );

    let version: packed::Uint32 = header.version.pack();
    assert_eq!(get_header.version().raw_data(), version.raw_data());

    let compact_target: packed::Uint32 = header.compact_target.pack();
    assert_eq!(
        get_header.compact_target().raw_data(),
        compact_target.raw_data()
    );

    let timestamp: packed::Uint64 = header.timestamp.pack();
    assert_eq!(get_header.timestamp().raw_data(), timestamp.raw_data());

    let number: packed::Uint64 = header.number.pack();
    assert_eq!(get_header.number().raw_data(), number.raw_data());

    let epoch: packed::Uint64 = header.epoch.pack();
    assert_eq!(get_header.epoch().raw_data(), epoch.raw_data());
}

fn check_cell(get_cell: &CellInfo, created_number: u64, consumed_number: Option<u64>) {
    let cell = &prepare_outputs()[0];

    let data: packed::Bytes = cell.data.pack();
    assert_eq!(get_cell.cell_data, data.raw_data());

    check_cell_output(&get_cell.cell_output);

    assert_eq!(get_cell.created_number, created_number);
    if get_cell.consumed_number.is_some() {
        assert_eq!(get_cell.consumed_number.unwrap(), consumed_number.unwrap());
    }
}

fn check_cell_output(get_output: &Bytes) {
    let output = &prepare_outputs()[0].output;
    let get_output: packed::CellOutput = packed::CellOutput::from_slice(get_output).unwrap();

    let capacity: packed::Uint64 = output.capacity.pack();
    assert_eq!(get_output.capacity().raw_data(), capacity.raw_data());

    check_script(&get_output.lock(), &output.lock);

    if !output.type_.is_empty() {
        check_script(&get_output.type_().to_opt().unwrap(), &output.type_[0]);
    } else {
        assert!(get_output.type_().to_opt().is_none());
    }
}

fn check_script(get_script: &packed::Script, script: &image_cell_abi::Script) {
    assert_eq!(get_script.code_hash(), script.code_hash.pack());

    let hash_type: packed::Byte = packed::Byte::new(script.hash_type);
    assert_eq!(get_script.hash_type(), hash_type);

    let args: packed::Bytes = script.args.pack();
    assert_eq!(get_script.args().raw_data(), args.raw_data());
}

fn prepare_header() -> image_cell_abi::Header {
    image_cell_abi::Header {
        version:           0x0,
        compact_target:    0x1a9c7b1a,
        timestamp:         0x16e62df76ed,
        number:            0x1,
        epoch:             0x7080291000049,
        parent_hash:       [0u8; 32],
        transactions_root: [1u8; 32],
        proposals_hash:    [2u8; 32],
        uncles_hash:       [3u8; 32],
        dao:               [4u8; 32],
        nonce:             0x78b105de64fc38a200000004139b0200,
        block_hash:        [5u8; 32],
    }
}

fn prepare_header_2() -> image_cell_abi::Header {
    image_cell_abi::Header {
        version:           0x0,
        compact_target:    0x1a9c7b1a,
        timestamp:         0x16e62df76ed,
        number:            0x2,
        epoch:             0x7080291000049,
        parent_hash:       [0u8; 32],
        transactions_root: [1u8; 32],
        proposals_hash:    [2u8; 32],
        uncles_hash:       [3u8; 32],
        dao:               [4u8; 32],
        nonce:             0x78b105de64fc38a200000004139b0200,
        block_hash:        [5u8; 32],
    }
}

fn prepare_outputs() -> Vec<image_cell_abi::CellInfo> {
    vec![image_cell_abi::CellInfo {
        out_point: image_cell_abi::OutPoint {
            tx_hash: [7u8; 32],
            index:   0x0,
        },
        output:    image_cell_abi::CellOutput {
            capacity: 0x34e62ce00,
            lock:     image_cell_abi::Script {
                args:      ethers::core::types::Bytes::from_str(
                    "0x927f3e74dceb87c81ba65a19da4f098b4de75a0d",
                )
                .unwrap(),
                code_hash: [8u8; 32],
                hash_type: 1,
            },
            type_:    vec![image_cell_abi::Script {
                args:      ethers::core::types::Bytes::from_str(
                    "0x6e9b17739760ffc617017f157ed40641f7aa51b2af9ee017b35a0b35a1e2297b",
                )
                .unwrap(),
                code_hash: [9u8; 32],
                hash_type: 0,
            }],
        },
        data:      ethers::core::types::Bytes::from_str("0x40420f00000000000000000000000000")
            .unwrap(),
    }]
}
