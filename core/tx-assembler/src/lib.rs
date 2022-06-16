mod indexer;
#[allow(clippy::all)]
mod molecule;
#[cfg(test)]
mod tests;
mod util;

use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};

use arc_swap::ArcSwap;
use ckb_jsonrpc_types::TransactionView as JsonTxView;
use ckb_types::core::{Capacity, TransactionView};
use ckb_types::packed::OutPoint;
use ckb_types::prelude::Entity;
use common_crypto::{BlsPublicKey, BlsSignature, PublicKey, Signature};
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};

use protocol::traits::{Context, TxAssembler};
use protocol::types::{Direction, Transfer, H160, H256};
use protocol::{async_trait, Display, ProtocolError, ProtocolErrorKind, ProtocolResult};

lazy_static::lazy_static! {
    static ref ACS_TRANSACTIONS: RwLock<HashMap<H256, TransactionView>> = RwLock::new(HashMap::new());
    static ref ACS_METADATA: ArcSwap<CkbMetadata> = ArcSwap::from_pointee(CkbMetadata::default());
}

#[derive(Default, Debug)]
struct CkbMetadata {
    stake_outpoint:    OutPoint,
    metadata_outpoint: OutPoint,
    metadata_typeid:   H256,
    erc20_config:      HashMap<H160, H256>,
}

#[derive(Clone)]
pub struct TxAssemblerImpl {
    rpc_client: HttpClient,
}

impl TxAssemblerImpl {
    pub fn new(indexer_url: String) -> TxAssemblerImpl {
        let rpc_client = HttpClientBuilder::default().build(indexer_url).unwrap();
        TxAssemblerImpl { rpc_client }
    }

    pub async fn update_metadata(
        &self,
        metadata_typeid_args: H256,
        stake_typeid_args: H256,
        chain_id: u8,
    ) -> ProtocolResult<H256> {
        let (metadata, metadata_typeid, metadata_outpoint) =
            indexer::fetch_crosschain_metdata(&self.rpc_client, metadata_typeid_args, chain_id)
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
        let stake_outpoint =
            indexer::fetch_axon_stake_outpoint(&self.rpc_client, stake_typeid_args).await?;
        let ckb_metadata = CkbMetadata {
            stake_outpoint,
            metadata_outpoint,
            metadata_typeid,
            erc20_config: erc20_token_config,
        };
        ACS_METADATA.swap(Arc::new(ckb_metadata));
        Ok(metadata_typeid)
    }
}

#[async_trait]
impl TxAssembler for TxAssemblerImpl {
    async fn generate_crosschain_transaction_digest(
        &self,
        _ctx: Context,
        transfers: &[Transfer],
    ) -> ProtocolResult<H256> {
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
            .collect::<Vec<_>>();
        let metadata = ACS_METADATA.load();
        let tx = util::build_transaction_with_outputs_and_celldeps(&ckb_transfers, &[
            &metadata.metadata_outpoint,
            &metadata.stake_outpoint,
        ]);
        println!(
            "[with outputs] tx = {}",
            serde_json::to_string_pretty(&JsonTxView::from(tx.clone())).unwrap()
        );
        let tx = indexer::fill_transaction_with_inputs_and_changes(
            &self.rpc_client,
            tx,
            ACS_METADATA.load().metadata_typeid,
            Capacity::bytes(1).unwrap(),
        )
        .await?;
        println!(
            "[with inputs] tx = {}",
            serde_json::to_string_pretty(&JsonTxView::from(tx.clone())).unwrap()
        );
        let hash = H256::from_slice(tx.hash().as_slice());
        ACS_TRANSACTIONS.write().unwrap().insert(hash, tx);
        Ok(hash)
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
    MetadataChainIdError(u8),

    #[display(fmt = "Not enough cells to response current crosschain requests")]
    InsufficientCrosschainCell,

    #[display(fmt = "No transaction found with Hash({})", _0)]
    NoTransactionFound(H256),
}

impl Error for TxAssemblerError {}

impl From<TxAssemblerError> for ProtocolError {
    fn from(error: TxAssemblerError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::TxAssember, Box::new(error))
    }
}
