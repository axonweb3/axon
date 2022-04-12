use crate::types::{
    CrossChainTransferPayload, SubmitCheckpointPayload, TransactionCompletionResponse,
};
use crate::{async_trait, traits::Context as OtherContext, ProtocolResult};

use ckb_jsonrpc_types::{
    BlockNumber, BlockView, HeaderView, OutputsValidator, Transaction, TransactionWithStatus,
};
use ckb_types::H256;

use std::{future::Future, pin::Pin};

pub type RPC<T> = Pin<Box<dyn Future<Output = ProtocolResult<T>> + Send + 'static>>;

#[async_trait]
pub trait CkbClient: Send + Sync {
    // async fn get_validator_list(&self, ctx: Context) ->
    // ProtocolResult<Vec<Validator>>;

    // async fn watch_cross_tx(&self, ctx: Context) -> ProtocolResult<Transaction>;

    // async fn verify_check_point(&self, ctx: Context, header: Header) ->
    // ProtocolResult<()>;
    fn get_block_by_number(&self, ctx: OtherContext, number: BlockNumber) -> RPC<BlockView>;

    fn get_tip_header(&self, ctx: OtherContext) -> RPC<HeaderView>;

    fn get_transaction(&self, ctx: OtherContext, hash: &H256)
        -> RPC<Option<TransactionWithStatus>>;

    fn send_transaction(
        &self,
        ctx: OtherContext,
        tx: &Transaction,
        outputs_validator: Option<OutputsValidator>,
    ) -> RPC<H256>;

    fn get_txs_by_hashes(
        &self,
        ctx: OtherContext,
        hash: Vec<H256>,
    ) -> RPC<Vec<Option<TransactionWithStatus>>>;

    // mercury api

    fn build_cross_chain_transfer_transaction(
        &self,
        ctx: OtherContext,
        paylod: CrossChainTransferPayload,
    ) -> RPC<TransactionCompletionResponse>;

    fn build_submit_checkpoint_transaction(
        &self,
        ctx: OtherContext,
        paylod: SubmitCheckpointPayload,
    ) -> RPC<TransactionCompletionResponse>;
}
