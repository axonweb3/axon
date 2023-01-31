use ckb_types::core::{DepType, HeaderView, TransactionView};
use ckb_types::{bytes::Bytes, h256, packed, prelude::*};
use ethers_core::abi::AbiEncode;

use core_executor::precompiles::{build_mock_tx, CellDep, CellWithWitness};
use core_executor::system_contract::image_cell::{image_cell_abi, DataProvider};
use protocol::{codec::hex_decode, tokio, traits::CkbClient, traits::Interoperation, types::H256};

use crate::InteroperationImpl;

use super::*;

const JOYID_TEST_TX_HASH: ckb_types::H256 =
    h256!("0xb18f9d2a719a77a85d73535acaf871950b99cd481ad2ceaafae009dbb6d46f69");

#[tokio::test(flavor = "multi_thread")]
async fn test_verify_joyid() {
    let mut handle = TestHandle::new(0).await;
    let tx = mock_signed_tx(1, build_image_cell_payload().await);
    let _ = handle.exec(vec![tx]);
    let case = test_case().await;
    let witness = build_witness("0xe80100001000000083010000830100006f010000021b155901e901eafebb7b6f4c9f9d3c46e60348f5d2e1adae0c04edfadaa84b4af56aac3dac2f0a56112161b773bf15be41ba5061f25130023c0ac4d03695e21daec35a04b6a9fb2f0cb59aad8dc26aed7e35df661eda4e75ed9c95d78ee9f65d8639e3fe5b2ada115867d1584b091a3ea9560108cb571835abd3182fb46671175913119670aa30099572b168ab0df94c4478648f2501f5f3c823023cff3529dc05000000097b2274797065223a22776562617574686e2e676574222c226368616c6c656e6765223a224d6a4130595752694e6a67334f4455795a6a637a4e324a6d5a4445344d6a59345a475a6a4f4441354f47566d5a4467314d6a417a5a474d774d57466b59574977596a6c6c5a444d78596a566d4e7a51334d6a637a5951222c226f726967696e223a2268747470733a5c2f5c2f6170702e6a6f7969642e646576222c22616e64726f69645061636b6167654e616d65223a22636f6d2e616e64726f69642e6368726f6d65227d6100000061000000100000001400000016000000000000010001470000004c4f595159db911f0aaf38c0729f381b5762b08fd46237424ec35a579a8b950284d1646c04ff007375626b65790000000000000000000000000000000000000000000000004fa6");

    let mock_tx = build_mock_tx(
        vec![CellWithWitness {
            tx_hash:      H256(case.0.tx_hash.0),
            index:        case.0.index,
            witness_type: witness.0,
            witness_lock: witness.1,
        }],
        vec![
            CellDep {
                tx_hash:  H256(
                    h256!("0xfda887b673dbc8af7ef64b03c37854d5f6eac3ec18c1961159572c1ee4ab499b").0,
                ),
                index:    0,
                dep_type: DepType::Code.into(),
            },
            CellDep {
                tx_hash:  H256(
                    h256!("0x073e67aec72467d75b36b2f2a3b8d211b91f687119e88a03639541b4c009e274").0,
                ),
                index:    0,
                dep_type: DepType::DepGroup.into(),
            },
            CellDep {
                tx_hash:  H256(
                    h256!("0x636a786001f87cb615acfcf408be0f9a1f077001f0bbc75ca54eadfe7e221713").0,
                ),
                index:    0,
                dep_type: DepType::DepGroup.into(),
            },
        ],
        vec![],
    );
    println!(
        "{:?}",
        serde_json::to_string_pretty(&ckb_jsonrpc_types::TransactionView::from(mock_tx.clone()))
            .unwrap()
    );

    let r = <InteroperationImpl as Interoperation>::verify_by_ckb_vm(
        Default::default(),
        DataProvider::default(),
        &mock_tx,
        u64::MAX,
    )
    .unwrap();
    println!("{:?}", r);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutPoint {
    pub tx_hash: ckb_types::H256,
    pub index:   u32,
}

impl From<packed::OutPoint> for OutPoint {
    fn from(value: packed::OutPoint) -> Self {
        OutPoint {
            tx_hash: value.tx_hash().unpack(),
            index:   value.index().unpack(),
        }
    }
}

impl From<OutPoint> for image_cell_abi::OutPoint {
    fn from(value: OutPoint) -> Self {
        image_cell_abi::OutPoint {
            tx_hash: value.tx_hash.0,
            index:   value.index,
        }
    }
}

async fn test_case() -> (OutPoint, Bytes) {
    let witness: Vec<u8> = get_ckb_tx(JOYID_TEST_TX_HASH)
        .await
        .witnesses()
        .get(0)
        .unwrap()
        .unpack();

    (
        OutPoint {
            tx_hash: JOYID_TEST_TX_HASH,
            index:   0,
        },
        witness.into(),
    )
}

fn build_witness(raw: &str) -> (Option<Bytes>, Option<Bytes>) {
    let witness = packed::WitnessArgs::from_slice(&hex_decode(raw).unwrap()).unwrap();

    (
        witness.input_type().to_opt().map(|r| r.unpack()),
        witness.lock().to_opt().map(|r| r.unpack()),
    )
}

async fn mock_header() -> image_cell_abi::Header {
    let rpc = init_rpc_client();
    let header: HeaderView = rpc
        .get_block_by_number(Context::new(), 7990521u64.into())
        .await
        .unwrap()
        .header
        .into();

    image_cell_abi::Header {
        version:           header.version(),
        compact_target:    header.compact_target(),
        timestamp:         header.timestamp(),
        number:            header.number(),
        epoch:             header.epoch().full_value(),
        parent_hash:       header.parent_hash().unpack().0,
        transactions_root: header.transactions_root().unpack().0,
        proposals_hash:    header.proposals_hash().unpack().0,
        uncles_hash:       [0u8; 32],
        dao:               header.dao().unpack().0,
        nonce:             header.nonce(),
        block_hash:        header.hash().unpack().0,
    }
}

async fn build_image_cell_payload() -> Vec<u8> {
    image_cell_abi::UpdateCall {
        header:  mock_header().await,
        inputs:  vec![],
        outputs: get_tx_cells(JOYID_TEST_TX_HASH).await,
    }
    .encode()
}

async fn get_cell_by_out_point(out_point: OutPoint) -> image_cell_abi::CellInfo {
    let (cell, data) = get_ckb_tx(out_point.tx_hash.0)
        .await
        .output_with_data(out_point.index as usize)
        .unwrap();

    let lock_script = image_cell_abi::Script {
        code_hash: cell.lock().code_hash().unpack().0,
        hash_type: cell.lock().hash_type().as_slice()[0],
        args:      {
            let tmp: Vec<u8> = cell.lock().args().unpack();
            tmp.into()
        },
    };
    let mut type_script = vec![];
    if let Some(s) = cell.type_().to_opt() {
        type_script.push(image_cell_abi::Script {
            code_hash: s.code_hash().unpack().0,
            hash_type: s.hash_type().as_slice()[0],
            args:      {
                let tmp: Vec<u8> = s.args().unpack();
                tmp.into()
            },
        })
    }

    let cell_output = image_cell_abi::CellOutput {
        capacity: cell.capacity().unpack(),
        lock:     lock_script,
        type_:    type_script,
    };

    image_cell_abi::CellInfo {
        out_point: out_point.into(),
        output:    cell_output,
        data:      data.into(),
    }
}

async fn get_tx_cells<T: Into<ckb_types::H256>>(hash: T) -> Vec<image_cell_abi::CellInfo> {
    let mut ret = Vec::new();
    let tx = get_ckb_tx(hash).await;

    // Get cell deps
    for cell_dep in tx.cell_deps().into_iter() {
        let out_point = cell_dep.out_point();
        let info = get_cell_by_out_point(OutPoint {
            tx_hash: out_point.tx_hash().unpack(),
            index:   out_point.index().unpack(),
        })
        .await;
        ret.push(info.clone());

        if cell_dep.dep_type() == DepType::DepGroup.into() {
            for op in parse_dep_group_data(&info.data).unwrap().into_iter() {
                let cell = get_cell_by_out_point(OutPoint {
                    tx_hash: op.tx_hash().unpack(),
                    index:   op.index().unpack(),
                })
                .await;
                ret.push(cell.clone());
            }
        }
    }

    // Get tx input cells
    for input in tx.inputs().into_iter() {
        let out_point = input.previous_output();
        let cell = get_cell_by_out_point(OutPoint {
            tx_hash: out_point.tx_hash().unpack(),
            index:   out_point.index().unpack(),
        })
        .await;
        ret.push(cell.clone());
    }

    ret
}

async fn get_ckb_tx<T: Into<ckb_types::H256>>(hash: T) -> TransactionView {
    let tx: packed::Transaction = RPC
        .get_txs_by_hashes(Context::new(), vec![hash.into()])
        .await
        .unwrap()
        .get(0)
        .cloned()
        .unwrap()
        .unwrap()
        .inner
        .into();
    tx.into_view()
}

fn parse_dep_group_data(slice: &[u8]) -> Result<packed::OutPointVec, String> {
    if slice.is_empty() {
        Err("data is empty".to_owned())
    } else {
        match packed::OutPointVec::from_slice(slice) {
            Ok(v) => {
                if v.is_empty() {
                    Err("dep group is empty".to_owned())
                } else {
                    Ok(v)
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }
}
