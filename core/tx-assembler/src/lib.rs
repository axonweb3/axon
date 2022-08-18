mod indexer;
#[allow(clippy::all)]
mod molecule;
#[cfg(test)]
mod tests;
mod util;

pub use crate::indexer::IndexerAdapter;

use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};

use arc_swap::ArcSwap;
use ckb_jsonrpc_types::JsonBytes;
use ckb_sdk::rpc::ckb_indexer::{Cell, Pagination, ScriptType, SearchKey};
use ckb_types::core::{Capacity, TransactionView};
use ckb_types::{bytes::Bytes, packed, prelude::*};

use common_crypto::{BlsPublicKey, BlsSignature, PublicKey, Signature};
use protocol::traits::{Context, TxAssembler, TxAssemblerAdapter};
use protocol::types::{Direction, Transfer, H160, H256};
use protocol::{async_trait, Display, ProtocolError, ProtocolErrorKind, ProtocolResult};

use crate::indexer::types as ckb;
use crate::molecule::Metadata;

lazy_static::lazy_static! {
    static ref ACS_TRANSACTIONS: RwLock<HashMap<H256, TransactionView>> = RwLock::new(HashMap::new());
    static ref ACS_METADATA: ArcSwap<CkbMetadata> = ArcSwap::from_pointee(CkbMetadata::default());
}

#[derive(Default, Debug)]
struct CkbMetadata {
    // stake_outpoint:    packed::OutPoint,
    metadata_outpoint: packed::OutPoint,
    metadata_typeid:   H256,
    erc20_config:      HashMap<H160, H256>,
}

#[derive(Clone)]
pub struct TxAssemblerImpl<Adapter> {
    adapter: Arc<Adapter>,
}

impl<Adapter: TxAssemblerAdapter + 'static> TxAssemblerImpl<Adapter> {
    pub fn new(adapter: Arc<Adapter>) -> Self {
        TxAssemblerImpl { adapter }
    }

    async fn fetch_live_cells(
        &self,
        search_key: SearchKey,
        limit: u32,
        cursor: Option<JsonBytes>,
    ) -> ProtocolResult<Pagination<ckb::Cell>> {
        let live_cells: Pagination<Cell> = self
            .adapter
            .fetch_live_cells(Context::new(), search_key, limit, cursor)
            .await?;

        Ok(Pagination {
            objects:     live_cells.objects.into_iter().map(From::from).collect(),
            last_cursor: live_cells.last_cursor,
        })
    }

    pub async fn fetch_axon_stake_outpoint(
        &self,
        stake_typeid_args: H256,
    ) -> ProtocolResult<packed::OutPoint> {
        let stake_typescript = util::build_typeid_script(stake_typeid_args);
        let search_key = SearchKey {
            script:      stake_typescript.into(),
            script_type: ScriptType::Type,
            filter:      None,
        };
        let stake_cell = self.fetch_live_cells(search_key, 1, None).await?;

        if let Some(cell) = stake_cell.objects.first() {
            Ok(cell.out_point.clone())
        } else {
            Err(TxAssemblerError::StakeTypeIdError(stake_typeid_args).into())
        }
    }

    pub async fn fetch_crosschain_metadata(
        &self,
        metadata_typeid_args: H256,
        axon_chain_id: u16,
    ) -> ProtocolResult<(Metadata, H256, packed::OutPoint)> {
        let metadata_typescript = util::build_typeid_script(metadata_typeid_args);
        let search_key = SearchKey {
            script:      metadata_typescript.clone().into(),
            script_type: ScriptType::Type,
            filter:      None,
        };

        let metadata_cell = self.fetch_live_cells(search_key, 1, None).await?;

        let ckb_metadata_cell = {
            if let Some(cell) = metadata_cell.objects.first() {
                cell
            } else {
                return Err(TxAssemblerError::MetadataTypeIdError(
                    metadata_typeid_args,
                    "no metadata found".into(),
                )
                .into());
            }
        };

        let metadata = Metadata::from_slice(&ckb_metadata_cell.output_data).map_err(|err| {
            TxAssemblerError::MetadataTypeIdError(metadata_typeid_args, err.to_string())
        })?;

        let chain_id = {
            let mut bytes = [0u8; 2];
            bytes.copy_from_slice(metadata.chain_id().as_slice());
            u16::from_le_bytes(bytes)
        };
        if chain_id != axon_chain_id {
            return Err(TxAssemblerError::MetadataChainIdError(chain_id).into());
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

    pub async fn fill_transaction_with_inputs_and_changes(
        &self,
        tx: TransactionView,
        metadata_typeid: H256,
        fee: Capacity,
    ) -> ProtocolResult<TransactionView> {
        let acs_lock_script = util::build_acs_lock_script(metadata_typeid);
        let mut acs_lock_output = packed::CellOutput::new_builder()
            .lock(acs_lock_script.clone())
            .build_exact_capacity(Capacity::zero())
            .unwrap();

        // prepare offer and require ckb
        let minimal_change_ckb = acs_lock_output.occupied_capacity(Capacity::zero()).unwrap();
        let (required_ckb, required_sudt_set, sudt_scripts) =
            util::compute_required_ckb_and_sudt(&tx, fee, minimal_change_ckb);
        let mut offered_ckb = Capacity::zero();
        let mut offered_sudt_set = HashMap::new();

        // fetch live vault cells to match crosschain requests
        let mut real_inputs_capacity = Capacity::zero();
        let mut tx_inputs = vec![];
        let mut cursor = None;
        while !util::is_offered_match_required(
            &offered_ckb,
            &required_ckb,
            &offered_sudt_set,
            &required_sudt_set,
        ) {
            let search_key = SearchKey {
                script:      acs_lock_script.clone().into(),
                script_type: ScriptType::Lock,
                filter:      None,
            };

            let lock_cells = self.fetch_live_cells(search_key, 20, cursor).await?;
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
                            let offered_sudt = &u128::from_le_bytes(uint128);
                            if offered_sudt < required_sudt {
                                // record total inputs capacity
                                let capacity = Capacity::shannons(cell.output.capacity().unpack());
                                real_inputs_capacity =
                                    real_inputs_capacity.safe_add(capacity).unwrap();
                                // record offered sudt amount
                                let value = offered_sudt_set.entry(hash).or_insert(0);
                                *value += offered_sudt;
                                return true;
                            }
                        }
                    // for CKB request
                    } else if offered_ckb.as_u64() < required_ckb.as_u64() {
                        let capacity = Capacity::shannons(cell.output.capacity().unpack());
                        offered_ckb = offered_ckb.safe_add(capacity).unwrap();
                        return true;
                    }
                    false
                })
                .map(|cell| {
                    packed::CellInput::new_builder()
                        .previous_output(cell.out_point.clone())
                        .build()
                })
                .collect::<Vec<packed::CellInput>>();
            tx_inputs.append(&mut inputs);
            if lock_cells.last_cursor.is_empty() {
                break;
            }
            cursor = Some(lock_cells.last_cursor);
        }

        log::info!(
            "[cross-chain] offered_ckb = {:?}, required_ckb = {:?}, offered_sudt = {:?}, required_sudt = {:?}",
            offered_ckb,
            required_ckb,
            offered_sudt_set,
            required_sudt_set
        );

        if !util::is_offered_match_required(
            &offered_ckb,
            &required_ckb,
            &offered_sudt_set,
            &required_sudt_set,
        ) {
            return Err(TxAssemblerError::InsufficientCrossChainCell.into());
        }

        // fill transaction inputs and build sUDT change outputs
        let mut tx = tx.as_advanced_builder().inputs(tx_inputs).build();
        for (hash, sudt_script) in sudt_scripts {
            let offered_sudt = offered_sudt_set.get(&hash).unwrap();
            let required_sudt = required_sudt_set.get(&hash).unwrap();
            assert!(offered_sudt >= required_sudt, "internal error");
            if offered_sudt > required_sudt {
                let change_sudt = offered_sudt - required_sudt;
                let sudt_output = packed::CellOutput::new_builder()
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
        real_inputs_capacity = real_inputs_capacity.safe_add(offered_ckb).unwrap();
        let real_outputs_capacity = tx.outputs_capacity().unwrap();

        log::info!(
            "[cross-chain] real_inputs_capacity = {:?}, real_outputs_capacity = {:?}, change_capacity = {:?}, fee = {:?}",
            real_inputs_capacity,
            real_outputs_capacity,
            minimal_change_ckb,
            fee
        );

        assert!(
            real_inputs_capacity.as_u64()
                >= real_outputs_capacity.as_u64() + minimal_change_ckb.as_u64() + fee.as_u64(),
            "internal error"
        );
        let real_change_ckb =
            real_inputs_capacity.as_u64() - real_outputs_capacity.as_u64() - fee.as_u64();
        acs_lock_output = acs_lock_output
            .as_builder()
            .capacity(real_change_ckb.pack())
            .build();
        tx = tx
            .as_advanced_builder()
            .output(acs_lock_output)
            .output_data(Bytes::new().pack())
            .build();

        Ok(tx)
    }

    #[allow(unused_variables)]
    pub async fn update_metadata(
        &self,
        metadata_typeid_args: H256,
        stake_typeid_args: H256,
        chain_id: u16,
        enable: bool,
    ) -> ProtocolResult<H256> {
        if !enable {
            return Ok(Default::default());
        }

        let (metadata, metadata_typeid, metadata_outpoint) = self
            .fetch_crosschain_metadata(metadata_typeid_args, chain_id)
            .await?;
        let erc20_token_config = {
            let mut config = HashMap::new();
            for i in 0..metadata.token_config().len() {
                let token = metadata.token_config().get(i).unwrap();
                let address = H160::from_slice(&token.erc20_address().raw_data());
                let hash = H256::from_slice(&token.sudt_typehash().raw_data());
                config.insert(address, hash);
            }
            config
        };
        // let stake_outpoint =
        // self.fetch_axon_stake_outpoint(stake_typeid_args).await?;
        let ckb_metadata = CkbMetadata {
            // stake_outpoint,
            metadata_outpoint,
            metadata_typeid,
            erc20_config: erc20_token_config,
        };
        ACS_METADATA.swap(Arc::new(ckb_metadata));
        Ok(metadata_typeid)
    }
}

#[async_trait]
impl<Adapter: TxAssemblerAdapter + 'static> TxAssembler for TxAssemblerImpl<Adapter> {
    async fn generate_crosschain_transaction_digest(
        &self,
        _ctx: Context,
        transfers: &[Transfer],
    ) -> ProtocolResult<TransactionView> {
        let erc20_tokens = ACS_METADATA.load();
        let ckb_transfers = transfers
            .iter()
            .filter(|value| {
                if value.direction == Direction::FromAxon {
                    if value.sudt_amount > 0 {
                        erc20_tokens.erc20_config.contains_key(&value.erc20_address)
                    } else {
                        value.ckb_amount > 0
                    }
                } else {
                    false
                }
            })
            .map(|value| {
                let mut sudt_lockhash = H256::default();
                if let Some(lockhash) = erc20_tokens.erc20_config.get(&value.erc20_address) {
                    sudt_lockhash = *lockhash;
                }
                util::build_transfer_output_cell(
                    value.address,
                    value.ckb_amount,
                    value.sudt_amount,
                    sudt_lockhash,
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|(offerred_ckb, required_ckb)| {
                TxAssemblerError::InsufficientWCKB(offerred_ckb, required_ckb)
            })?;
        let metadata = ACS_METADATA.load();
        let tx = util::build_transaction_with_outputs_and_celldeps(&ckb_transfers, &[
            &metadata.metadata_outpoint,
            // &metadata.stake_outpoint,
        ]);

        // log::info!(
        //     "[with outputs] tx = {}",
        //     serde_json::to_string_pretty(&JsonTxView::from(tx.clone())).unwrap()
        // );

        let tx = self
            .fill_transaction_with_inputs_and_changes(
                tx,
                ACS_METADATA.load().metadata_typeid,
                Capacity::bytes(1).unwrap(),
            )
            .await?;

        // log::info!(
        //     "[with inputs] tx = {}",
        //     serde_json::to_string_pretty(&JsonTxView::from(tx.clone())).unwrap()
        // );

        let hash = H256::from_slice(tx.hash().as_slice());
        ACS_TRANSACTIONS.write().unwrap().insert(hash, tx.clone());

        Ok(tx)
    }

    fn complete_crosschain_transaction(
        &self,
        _ctx: Context,
        digest: H256,
        bls_signature: &BlsSignature,
        bls_pubkey_list: &[BlsPublicKey],
    ) -> ProtocolResult<TransactionView> {
        let tx = {
            if let Some(tx) = ACS_TRANSACTIONS.read().unwrap().get(&digest) {
                tx.clone()
            } else {
                return Err(TxAssemblerError::NoTransactionFound(digest).into());
            }
        };
        let signature = {
            let mut signature = [0u8; 96];
            signature.copy_from_slice(&bls_signature.to_bytes());
            signature
        };
        let pubkey_list = bls_pubkey_list
            .iter()
            .map(|bls_pubkey| {
                let mut pubkey = [0u8; 48];
                pubkey.copy_from_slice(&bls_pubkey.to_bytes());
                pubkey
            })
            .collect::<Vec<_>>();
        let tx = util::complete_transaction_with_witnesses(tx, &signature, &pubkey_list);
        ACS_TRANSACTIONS
            .write()
            .unwrap()
            .insert(H256::from_slice(tx.hash().as_slice()), tx.clone());
        Ok(tx)
    }
}

#[derive(Debug, Display)]
pub enum TxAssemblerError {
    #[display(fmt = "Indexer RPC request error = {}", _0)]
    IndexerRpcError(String),

    #[display(fmt = "Cannot get stake cell by type_id_args = {:?}", _0)]
    StakeTypeIdError(H256),

    #[display(fmt = "Cannot get metadata by type_id_args = {:?}, error = {}", _0, _1)]
    MetadataTypeIdError(H256, String),

    #[display(fmt = "ChainId = {} from metadata isn't equal to Axon", _0)]
    MetadataChainIdError(u16),

    #[display(fmt = "Not enough cells to response current crosschain requests")]
    InsufficientCrossChainCell,

    #[display(fmt = "No transaction found with Hash({})", _0)]
    NoTransactionFound(H256),

    #[display(fmt = "Not enough wCKB to wrap a new generated Cell ({}/{})", _0, _1)]
    InsufficientWCKB(u64, u64),
}

impl Error for TxAssemblerError {}

impl From<TxAssemblerError> for ProtocolError {
    fn from(error: TxAssemblerError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::TxAssembler, Box::new(error))
    }
}
