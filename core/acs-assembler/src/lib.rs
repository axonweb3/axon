mod indexer;
mod molecule;
#[cfg(test)]
mod tests;
mod util;

use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};

use arc_swap::ArcSwap;
use ckb_types::core::{Capacity, TransactionView};
use ckb_types::packed::OutPoint;
use ckb_types::prelude::Entity;
use common_crypto::{BlsPublicKey, BlsSignature, PublicKey, Signature};
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};

use protocol::traits::{AcsAssembler, Context};
use protocol::types::{Direction, Transfer, H160, H256};
use protocol::{async_trait, Display, ProtocolError, ProtocolErrorKind, ProtocolResult};

lazy_static::lazy_static! {
    static ref ACS_TRANSACTIONS: RwLock<HashMap<H256, TransactionView>> = RwLock::new(HashMap::new());
    static ref ACS_METADATA: ArcSwap<CkbMetadata> = ArcSwap::from_pointee(CkbMetadata::default());
}

#[derive(Default)]
struct CkbMetadata {
    metadata_outpoint: OutPoint,
    metadata_typeid:   H256,
    erc20_config:      HashMap<H160, H256>,
}

#[derive(Clone)]
pub struct AcsAssemblerImpl {
    rpc_client: HttpClient,
}

impl AcsAssemblerImpl {
    pub fn new(indexer_url: String) -> AcsAssemblerImpl {
        let rpc_client = HttpClientBuilder::default().build(indexer_url).unwrap();
        AcsAssemblerImpl { rpc_client }
    }

    pub async fn update_metadata(
        &mut self,
        metadata_typeid_args: H160,
        chain_id: u8,
    ) -> ProtocolResult<()> {
        let (metadata, typeid, outpoint) =
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
        let ckb_metadata = {
            CkbMetadata {
                metadata_outpoint: outpoint,
                metadata_typeid:   typeid,
                erc20_config:      erc20_token_config,
            }
        };
        ACS_METADATA.swap(Arc::new(ckb_metadata));
        Ok(())
    }
}

#[async_trait]
impl AcsAssembler for AcsAssemblerImpl {
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
                let sudt_lockhash = erc20_tokens.erc20_config.get(&value.erc20_address).unwrap();
                util::build_transfer_output_cell(
                    value.address,
                    value.ckb_amount,
                    value.sudt_amount,
                    *sudt_lockhash,
                )
            })
            .collect::<Vec<_>>();
        let tx = util::build_transaction_with_outputs(&ckb_transfers);
        let tx = indexer::fill_transaction_with_inputs(
            &self.rpc_client,
            tx,
            ACS_METADATA.load().metadata_typeid,
            Capacity::one(),
        )
        .await?;
        let mut hash = [0u8; 32];
        hash.copy_from_slice(tx.hash().as_slice());
        ACS_TRANSACTIONS
            .write()
            .unwrap()
            .insert(H256::from(hash), tx);
        Ok(H256::from(hash))
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
                return Err(AcsAssemblerError::NoTransactionFound(digest).into());
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
        let metadata_outpoint = &ACS_METADATA.load().metadata_outpoint;
        let tx = util::complete_transaction_with_witnesses_and_celldeps(
            tx,
            &signature,
            &pubkey_list,
            metadata_outpoint,
        );
        Ok(tx)
    }
}

#[derive(Debug, Display)]
pub enum AcsAssemblerError {
    #[display(fmt = "Indexer RPC request error = {}", _0)]
    IndexerRpcError(String),

    #[display(fmt = "Cannot get metadata of type_id_args = {:?}, error = {}", _0, _1)]
    MetadataTypeIdError(H160, String),

    #[display(fmt = "ChainId = {} from metadata isn't equal to Axon", _0)]
    MetadataChainIdError(u8),

    #[display(fmt = "Not enough cells to response current crosschain requests")]
    InsufficientCrosschainCell,

    #[display(fmt = "No transaction found with Hash({})", _0)]
    NoTransactionFound(H256),
}

impl Error for AcsAssemblerError {}

impl From<AcsAssemblerError> for ProtocolError {
    fn from(error: AcsAssemblerError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::AcsAssember, Box::new(error))
    }
}
