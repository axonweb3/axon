use ckb_types::core::{DepType, HeaderView, TransactionView};
use ckb_types::{bytes::Bytes, h256, packed, prelude::*};
use ethers_core::abi::AbiEncode;

use core_executor::precompiles::{build_mock_tx, CellDep, CellWithWitness};
use core_executor::system_contract::image_cell::{image_cell_abi, DataProvider};
use protocol::{codec::hex_decode, tokio, traits::CkbClient, traits::Interoperation, types::H256};

use super::*;
use crate::{parse_dep_group_data, tests::TestHandle, InteroperationImpl};

const JOYID_TEST_TX_HASH: ckb_types::H256 =
    h256!("0x718930de57046ced9eba895b0c9d8ecba41f08ebe8b3ef2e6cc5bc8e1cd88d4f");

#[tokio::test(flavor = "multi_thread")]
async fn test_verify_joyid() {
    let mut handle = TestHandle::new(0).await;
    let tx = mock_signed_tx(1, build_image_cell_payload().await);
    let _ = handle.exec(vec![tx]);

    let case = OutPoint {
        tx_hash: h256!("0xf8fc23655fe15dd4a39337155f4dcfe0ef59a6f2d7fb7f083cc3c351e9ff80d2"),
        index:   1,
    };
    let witness = build_witness("0x830100001000000083010000830100006f01000001780326dedc58aef92d9a76f46e3517eb90e84e966360db25ed128500368c02cbc3a7d5af2f8805ead57f7effa9dba177911abde069838cdd03aaaaf5a8ba5da067ae11e8a7282b178d133b183f32450d413c2ed5231d6e47785aa659bf112cfb492042e7f9cc68e1a8097ea068f3a305424ee33c712aa067a2ac65ea7db542825913119670aa30099572b168ab0df94c4478648f2501f5f3c823023cff3529dc05000000477b2274797065223a22776562617574686e2e676574222c226368616c6c656e6765223a224e6a517959544e6d5a44597a4d7a6c6d4d5755354e5451304f54466b5954637a4d7a6c6a4e57457a4e6d4d355a44526c597a4932596d497a4d3245334d6a45784e57566a4e4451784e3256695a4452684e324a6a5951222c226f726967696e223a2268747470733a5c2f5c2f6170702e6a6f7969642e646576222c22616e64726f69645061636b6167654e616d65223a22636f6d2e616e64726f69642e6368726f6d65227d");

    let mock_tx = build_mock_tx(
        vec![CellWithWitness {
            tx_hash:      H256(case.tx_hash.0),
            index:        case.index,
            witness_type: witness.0,
            witness_lock: witness.1,
        }],
        vec![
            CellDep {
                tx_hash:  H256(
                    h256!("0xe778611f59d65bc0c558a0a14a7fe12c4a937712f9cae6ca7aa952802703bd5a").0,
                ),
                index:    0,
                dep_type: DepType::DepGroup.into(),
            },
            // CellDep {
            //     tx_hash:  H256(
            //         h256!("0x073e67aec72467d75b36b2f2a3b8d211b91f687119e88a03639541b4c009e274").
            // 0,     ),
            //     index:    0,
            //     dep_type: DepType::DepGroup.into(),
            // },
            // CellDep {
            //     tx_hash:  H256(
            //         h256!("0x636a786001f87cb615acfcf408be0f9a1f077001f0bbc75ca54eadfe7e221713").
            // 0,     ),
            //     index:    0,
            //     dep_type: DepType::DepGroup.into(),
            // },
        ],
        vec![],
    );

    // The following process is only for test
    let origin_tx = get_ckb_tx(JOYID_TEST_TX_HASH).await;
    let mock_tx = mock_tx
        .as_advanced_builder()
        .outputs(origin_tx.outputs())
        .outputs_data(origin_tx.outputs_data())
        .build();

    println!(
        "{:?}\n",
        serde_json::to_string(&ckb_jsonrpc_types::TransactionView::from(mock_tx.clone())).unwrap()
    );

    let r = InteroperationImpl::verify_by_ckb_vm(
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
