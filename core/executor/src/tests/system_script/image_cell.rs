use std::collections::BTreeMap;
use std::str::FromStr;

use ckb_types::{bytes::Bytes, packed, prelude::*};
use ethers::abi::AbiEncode;

use common_config_parser::types::ConfigRocksDB;
use protocol::types::{Backend, MemoryBackend, TxResp, H160, U256};

use crate::system_contract::image_cell::{image_cell_abi, CellInfo, CellKey, ImageCellContract};
use crate::system_contract::{init, CkbLightClientContract, SystemContract, HEADER_CELL_ROOT_KEY};
use crate::tests::{gen_tx, gen_vicinity};
use crate::{CURRENT_HEADER_CELL_ROOT, CURRENT_METADATA_ROOT};

static ROCKSDB_PATH: &str = "./free-space/system-contract/image-cell";

pub fn test_write_functions() {
    let vicinity = gen_vicinity();
    let mut backend = MemoryBackend::new(&vicinity, BTreeMap::new());

    let executor = ImageCellContract::default();
    let (m_root, h_root) = init(ROCKSDB_PATH, ConfigRocksDB::default(), &mut backend);

    CURRENT_METADATA_ROOT.with(|r| *r.borrow_mut() = m_root);
    CURRENT_HEADER_CELL_ROOT.with(|r| *r.borrow_mut() = h_root);

    test_update_first(&mut backend, &executor);
    test_update_second(&mut backend, &executor);

    test_rollback_first(&mut backend, &executor);
    test_rollback_second(&mut backend, &executor);

    test_set_state(&mut backend, &executor);
}

fn test_update_first(backend: &mut MemoryBackend, executor: &ImageCellContract) {
    let data = image_cell_abi::UpdateCall {
        blocks: vec![image_cell_abi::BlockUpdate {
            block_number: 0x1,
            tx_inputs:    vec![],
            tx_outputs:   prepare_outputs(),
        }],
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    check_nonce(backend, 1);

    let root = backend.storage(CkbLightClientContract::ADDRESS, *HEADER_CELL_ROOT_KEY);
    let cell_key = CellKey::new([7u8; 32], 0x0);
    let get_cell = executor.get_cell(root, &cell_key).unwrap().unwrap();
    check_cell(&get_cell, 0x1, None);
}

fn test_update_second(backend: &mut MemoryBackend, executor: &ImageCellContract) {
    let data = image_cell_abi::UpdateCall {
        blocks: vec![image_cell_abi::BlockUpdate {
            block_number: 0x2,
            tx_inputs:    vec![image_cell_abi::OutPoint {
                tx_hash: [7u8; 32],
                index:   0x0,
            }],
            tx_outputs:   vec![],
        }],
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    check_nonce(backend, 2);

    let root = backend.storage(CkbLightClientContract::ADDRESS, *HEADER_CELL_ROOT_KEY);
    let cell_key = CellKey::new([7u8; 32], 0x0);
    let get_cell = executor.get_cell(root, &cell_key).unwrap().unwrap();
    check_cell(&get_cell, 0x1, Some(0x2));
}

fn test_rollback_first(backend: &mut MemoryBackend, executor: &ImageCellContract) {
    let data = image_cell_abi::RollbackCall {
        blocks: vec![image_cell_abi::BlockRollBlack {
            tx_inputs:  vec![image_cell_abi::OutPoint {
                tx_hash: [7u8; 32],
                index:   0x0,
            }],
            tx_outputs: vec![],
        }],
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    let root = backend.storage(CkbLightClientContract::ADDRESS, *HEADER_CELL_ROOT_KEY);
    let cell_key = CellKey::new([7u8; 32], 0x0);
    let get_cell = executor.get_cell(root, &cell_key).unwrap().unwrap();
    check_cell(&get_cell, 0x1, None);
}

fn test_rollback_second(backend: &mut MemoryBackend, executor: &ImageCellContract) {
    let data = image_cell_abi::RollbackCall {
        blocks: vec![image_cell_abi::BlockRollBlack {
            tx_inputs:  vec![],
            tx_outputs: vec![image_cell_abi::OutPoint {
                tx_hash: [7u8; 32],
                index:   0x0,
            }],
        }],
    };

    let r = exec(backend, executor, data.encode());
    assert!(r.exit_reason.is_succeed());

    let root = backend.storage(CkbLightClientContract::ADDRESS, *HEADER_CELL_ROOT_KEY);
    let cell_key = CellKey::new([7u8; 32], 0x0);
    let get_cell = executor.get_cell(root, &cell_key).unwrap();
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

fn check_nonce(backend: &mut MemoryBackend, nonce: u64) {
    assert_eq!(
        backend.basic(CkbLightClientContract::ADDRESS).nonce,
        U256::zero()
    );
    assert_eq!(
        backend.basic(ImageCellContract::ADDRESS).nonce,
        U256::zero()
    );
    assert_eq!(
        backend
            .basic(H160::from_str("0xf000000000000000000000000000000000000000").unwrap())
            .nonce,
        nonce.into()
    )
}
