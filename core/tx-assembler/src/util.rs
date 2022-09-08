use std::collections::HashMap;

use ckb_sdk::constants::TYPE_ID_CODE_HASH;
use ckb_types::bytes::Bytes;
use ckb_types::core::{Capacity, DepType, ScriptHashType, TransactionView};
use ckb_types::h256;
use ckb_types::packed::{CellDep, CellOutput, OutPoint, Script, WitnessArgs};
use ckb_types::prelude::{Builder, Entity, Pack, Unpack};
use protocol::types::H256;

use crate::molecule;

pub const ACS_LOCK_CODE_HASH: ckb_types::H256 =
    h256!("0xe6716305da395dbd3dc094695b2a6dc9160e186e41102cceac377d78a350c962");
pub const SUDT_CODE_HASH: ckb_types::H256 =
    h256!("0xc5e5dcf215925f7ef4dfaf5f4b4f105bc321c02776d6e7d52a1db3fcd9d011a4");

const ACS_LOCK_TX_HASH: ckb_types::H256 =
    h256!("0x65bb542bab44a1769379cca21c70e031d753942b272f7b6ac1b87e95d4325d24");
const SUDT_TX_HASH: ckb_types::H256 =
    h256!("0xe12877ebd2c3c364dc46c5c992bcfaf4fee33fa13eebdf82c591fc9825aab769");
const ACS_REQUEST_TX_HASH: ckb_types::H256 =
    h256!("0x272168c3d7c4576398f6cb82a15490ad07b11602c0af8810f7dc0fd4252717fb");

fn build_script(code_hash: ckb_types::H256, args: &[u8]) -> Script {
    Script::new_builder()
        .code_hash(code_hash.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(args.pack())
        .build()
}

pub fn build_typeid_script(typeid_args: H256) -> Script {
    build_script(TYPE_ID_CODE_HASH, typeid_args.as_bytes())
}

pub fn build_acs_lock_script(metadata_typeid: H256) -> Script {
    build_script(ACS_LOCK_CODE_HASH, metadata_typeid.as_bytes())
}

pub fn build_transfer_output_cell(
    lock_script: Script,
    ckb_amount: u64,
    sudt_amount: u128,
    sudt_lockhash: H256,
) -> Result<(CellOutput, Bytes), (u64, u64)> {
    let mut cell = CellOutput::new_builder()
        .lock(lock_script)
        .build_exact_capacity(Capacity::zero())
        .unwrap();
    let mut output_data = Bytes::new();
    if sudt_amount > 0 {
        let type_script = build_script(SUDT_CODE_HASH, sudt_lockhash.as_bytes());
        cell = cell
            .as_builder()
            .type_(Some(type_script).pack())
            .build_exact_capacity(Capacity::zero())
            .unwrap();
        output_data = Bytes::from(sudt_amount.to_le_bytes().to_vec());
    }
    let required_ckb: Capacity = cell.capacity().unpack();
    if ckb_amount >= required_ckb.as_u64() {
        cell = cell
            .as_builder()
            .capacity(Capacity::shannons(ckb_amount).pack())
            .build();
        Ok((cell, output_data))
    } else {
        Err((ckb_amount, required_ckb.as_u64()))
    }
}

pub fn build_transaction_with_outputs_and_celldeps(
    output_cell_and_data: &Vec<(CellOutput, Bytes)>,
    extra_outpoints: &[&OutPoint],
) -> TransactionView {
    let mut tx = TransactionView::new_advanced_builder().build();
    for (cell, data) in output_cell_and_data {
        tx = tx
            .as_advanced_builder()
            .output(cell.clone())
            .output_data(data.pack())
            .build();
    }
    let mut celldeps = vec![ACS_LOCK_TX_HASH, SUDT_TX_HASH, ACS_REQUEST_TX_HASH]
        .into_iter()
        .map(|tx_hash| {
            CellDep::new_builder()
                .out_point(
                    OutPoint::new_builder()
                        .tx_hash(tx_hash.pack())
                        .index(0u32.pack())
                        .build(),
                )
                .dep_type(DepType::Code.into())
                .build()
        })
        .collect::<Vec<_>>();
    extra_outpoints.iter().for_each(|outpoint| {
        celldeps.push(
            CellDep::new_builder()
                .out_point((*outpoint).clone())
                .dep_type(DepType::Code.into())
                .build(),
        );
    });
    tx.as_advanced_builder().cell_deps(celldeps).build()
}

pub fn complete_transaction_with_witnesses(
    tx: TransactionView,
    signature: &[u8; 96],
    pubkey_list: &[[u8; 48]],
) -> TransactionView {
    let signature = molecule::Signautre::new_unchecked(Bytes::from(signature.to_vec()));
    let bls_pubkeys = {
        let pubkey_list = pubkey_list
            .iter()
            .map(|pubkey| molecule::BlsPubkey::new_unchecked(Bytes::from(pubkey.to_vec())))
            .collect::<Vec<_>>();
        molecule::BlsPubkeyList::new_builder()
            .set(pubkey_list)
            .build()
    };
    let acs_witness = molecule::Witness::new_builder()
        .signature(signature)
        .bls_pubkeys(bls_pubkeys)
        .build();
    let witness = WitnessArgs::new_builder()
        .lock(Some(acs_witness.as_bytes()).pack())
        .build();
    tx.as_advanced_builder()
        .witness(witness.as_bytes().pack())
        .build()
}

pub fn compute_required_ckb_and_sudt(
    tx: &TransactionView,
    fee: Capacity,
    change: Capacity,
) -> (
    Capacity,
    HashMap<ckb_types::H256, u128>,
    HashMap<ckb_types::H256, Script>,
) {
    let mut required_sudt_set = HashMap::new();
    let mut sudt_scripts = HashMap::new();
    for i in 0..tx.outputs().len() {
        if let Some(output) = tx.outputs().get(i) {
            if let Some(sudt_script) = output.type_().to_opt() {
                let data = tx.outputs_data().get(i).unwrap();
                let mut uint128 = [0u8; 16];
                uint128.copy_from_slice(data.as_slice());
                let required_sudt = u128::from_le_bytes(uint128);
                let hash = sudt_script.calc_script_hash().unpack();
                *required_sudt_set.entry(hash.clone()).or_default() += required_sudt;
                sudt_scripts.insert(hash, sudt_script);
            }
        }
    }
    let required_ckb = {
        let mut ckb = tx.outputs_capacity().unwrap();
        ckb = ckb.safe_add(fee).unwrap();
        ckb.safe_add(change).unwrap()
    };
    (required_ckb, required_sudt_set, sudt_scripts)
}

pub fn is_offered_match_required(
    offerred_ckb: &Capacity,
    required_ckb: &Capacity,
    offerred_sudt: &HashMap<ckb_types::H256, u128>,
    required_sudt: &HashMap<ckb_types::H256, u128>,
) -> bool {
    if offerred_ckb.as_u64() >= required_ckb.as_u64() {
        let sudt_not_enough = required_sudt.iter().any(|(hash, required)| {
            if let Some(offerred) = offerred_sudt.get(hash) {
                offerred < required
            } else {
                true
            }
        });
        !sudt_not_enough
    } else {
        false
    }
}
