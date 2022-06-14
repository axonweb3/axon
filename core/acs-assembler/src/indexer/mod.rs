mod types;
use types::{ckb, Cell, JsonBytes, Order, Pagination, ScriptType, SearchKey, Uint32};

use std::collections::HashMap;

use ckb_types::bytes::Bytes;
use ckb_types::core::{Capacity, ScriptHashType, TransactionView};
use ckb_types::packed::{CellInput, CellOutput, OutPoint, Script};
use ckb_types::prelude::{Builder, Entity, Pack, Unpack};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::types::ParamsSer;
use protocol::types::{H160, H256};
use protocol::ProtocolResult;
use serde_json::json;

use crate::molecule::Metadata;
use crate::util;
use crate::AcsAssemblerError;

async fn fetch_live_cells(
    rpc_client: &HttpClient,
    search_key: SearchKey,
    limit: u32,
    cursor: Option<JsonBytes>,
) -> ProtocolResult<Pagination<ckb::Cell>> {
    let live_cells: Pagination<Cell> = rpc_client
        .request(
            "get_cells",
            Some(ParamsSer::Array(vec![
                json!(search_key),
                json!(Order::Asc),
                json!(Uint32::from(limit)),
                json!(cursor),
            ])),
        )
        .await
        .map_err(|err| AcsAssemblerError::IndexerRpcError(err.to_string()))?;
    let ckb_cells = live_cells
        .objects
        .iter()
        .map(ckb::Cell::from)
        .collect::<Vec<ckb::Cell>>();
    Ok(Pagination::<ckb::Cell> {
        objects:     ckb_cells,
        last_cursor: live_cells.last_cursor,
    })
}

pub async fn fetch_crosschain_metdata(
    rpc_client: &HttpClient,
    metadata_typeid_args: H160,
    axon_chain_id: u8,
) -> ProtocolResult<(Metadata, H256, OutPoint)> {
    let metadata_typescript = Script::new_builder()
        .code_hash(util::TYPE_ID_CODE_HASH.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(metadata_typeid_args.as_bytes().pack())
        .build();

    let search_key = SearchKey::new(metadata_typescript.clone().into(), ScriptType::Type);
    let metadata_cell = fetch_live_cells(rpc_client, search_key, 1, None).await?;

    let ckb_metadata_cell = {
        if let Some(cell) = metadata_cell.objects.first() {
            cell
        } else {
            return Err(AcsAssemblerError::MetadataTypeIdError(
                metadata_typeid_args,
                "no metadata found".into(),
            )
            .into());
        }
    };

    let metadata =
        Metadata::from_slice(&ckb_metadata_cell.output_data).map_err(|err| {
            AcsAssemblerError::MetadataTypeIdError(metadata_typeid_args, err.to_string())
        })?;

    if u8::from(metadata.chain_id()) != axon_chain_id {
        return Err(AcsAssemblerError::MetadataChainIdError(metadata.chain_id().into()).into());
    }

    let hash = metadata_typescript.calc_script_hash().unpack();
    let mut metadata_typeid = [0u8; 32];
    metadata_typeid.copy_from_slice(hash.as_bytes());

    Ok((
        metadata,
        H256::from(metadata_typeid),
        ckb_metadata_cell.out_point.clone(),
    ))
}

pub async fn fill_transaction_with_inputs(
    rpc_client: &HttpClient,
    tx: TransactionView,
    metadata_typeid: H256,
    fee: Capacity,
) -> ProtocolResult<TransactionView> {
    let acs_lock_script = Script::new_builder()
        .code_hash(util::ACS_LOCK_CODE_HASH.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(Bytes::from(metadata_typeid.as_bytes().to_vec()).pack())
        .build();

    let (required_ckb, required_sudt_set, sudt_scripts) =
        util::compute_required_ckb_and_sudt(&tx, fee);
    let mut offerred_ckb = Capacity::zero();
    let mut offerred_sudt_set = HashMap::new();

    // fetch live vault cells to match crosschain requests
    let mut real_inputs_capacity = Capacity::zero();
    let mut tx_inputs = vec![];
    let mut cursor = None;
    while !util::is_offerred_match_required(
        &offerred_ckb,
        &required_ckb,
        &offerred_sudt_set,
        &required_sudt_set,
    ) {
        let search_key = SearchKey::new(acs_lock_script.clone().into(), ScriptType::Lock);
        let lock_cells = fetch_live_cells(rpc_client, search_key, 20, cursor).await?;
        let mut inputs = lock_cells
            .objects
            .iter()
            .filter(|cell| {
                // for sUDT request
                if let Some(sudt_script) = cell.output.type_().to_opt() {
                    let hash = sudt_script.calc_script_hash().unpack();
                    if let Some(required_sudt) = required_sudt_set.get(&hash) {
                        let mut uint128 = [0u8; 16];
                        uint128.copy_from_slice(&cell.output_data.to_vec()[..16]);
                        let offerred_sudt = &u128::from_le_bytes(uint128);
                        if offerred_sudt < required_sudt {
                            // record total inputs capacity
                            let capacity = Capacity::shannons(cell.output.capacity().unpack());
                            real_inputs_capacity = real_inputs_capacity.safe_add(capacity).unwrap();
                            // record offerred sudt amount
                            let value = offerred_sudt_set.entry(hash).or_insert(0);
                            *value += offerred_sudt;
                            return true;
                        }
                    }
                // for CKB request
                } else if offerred_ckb.as_u64() < required_ckb.as_u64() {
                    let capacity = Capacity::shannons(cell.output.capacity().unpack());
                    offerred_ckb = offerred_ckb.safe_add(capacity).unwrap();
                    return true;
                }
                false
            })
            .map(|cell| {
                CellInput::new_builder()
                    .previous_output(cell.out_point.clone())
                    .build()
            })
            .collect::<Vec<CellInput>>();
        tx_inputs.append(&mut inputs);
        if lock_cells.last_cursor.is_empty() {
            break;
        }
        cursor = Some(lock_cells.last_cursor);
    }

    if util::is_offerred_match_required(
        &offerred_ckb,
        &required_ckb,
        &offerred_sudt_set,
        &required_sudt_set,
    ) {
        return Err(AcsAssemblerError::InsufficientCrosschainCell.into());
    }

    println!(
        "offerred_ckb = {:?}, required_ckb = {:?}, offerred_sudt = {:?}, required_sudt = {:?}",
        offerred_ckb, required_ckb, offerred_sudt_set, required_sudt_set
    );

    // fill transaction inputs and build sUDT change outputs
    let mut tx = tx.as_advanced_builder().inputs(tx_inputs).build();
    for (hash, sudt_script) in sudt_scripts {
        let offerred_sudt = offerred_sudt_set.get(&hash).unwrap();
        let required_sudt = required_sudt_set.get(&hash).unwrap();
        assert!(offerred_sudt >= required_sudt, "internal error");
        if offerred_sudt > required_sudt {
            let change_sudt = offerred_sudt - required_sudt;
            let sudt_output = CellOutput::new_builder()
                .lock(acs_lock_script.clone())
                .type_(Some(sudt_script).pack())
                .build_exact_capacity(Capacity::bytes(16).unwrap())
                .unwrap();
            tx = tx
                .as_advanced_builder()
                .output(sudt_output)
                .output_data(Bytes::from(change_sudt.to_le_bytes().to_vec()).pack())
                .build();
        }
    }

    // build CKB change output
    real_inputs_capacity = real_inputs_capacity.safe_add(offerred_ckb).unwrap();
    let real_outputs_capacity = tx.outputs_capacity().unwrap();
    let mut acs_lock_output = CellOutput::new_builder()
        .lock(acs_lock_script.clone())
        .build_exact_capacity(Capacity::zero())
        .unwrap();
    let extra_capacity = acs_lock_output.occupied_capacity(Capacity::zero()).unwrap();
    assert!(
        real_inputs_capacity.as_u64() > real_outputs_capacity.as_u64() + extra_capacity.as_u64(),
        "internal error"
    );
    acs_lock_output = acs_lock_output
        .as_builder()
        .capacity((real_inputs_capacity.as_u64() - real_outputs_capacity.as_u64()).pack())
        .build();
    tx = tx
        .as_advanced_builder()
        .output(acs_lock_output)
        .output_data(Bytes::new().pack())
        .build();

    Ok(tx)
}
