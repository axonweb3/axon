use ckb_types::core::TransactionView;

use common_crypto::{BlsPublicKey, BlsSignature};

use crate::traits::MessageTarget;
use crate::types::{
    Block, BlockNumber, Hash, Log, Metadata, Proof, RequestTxHashes, SignedTransaction, Transfer,
    TxResp, H160, U256,
};
use crate::{async_trait, traits::Context, ProtocolResult};

#[async_trait]
pub trait CrossAdapter: Send + Sync {
    async fn send_axon_tx(&self, ctx: Context, stx: SignedTransaction) -> ProtocolResult<()>;

    async fn send_ckb_tx(
        &self,
        ctx: Context,
        tx: ckb_jsonrpc_types::TransactionView,
    ) -> ProtocolResult<()>;

    async fn insert_in_process(&self, ctx: Context, key: &[u8], val: &[u8]) -> ProtocolResult<()>;

    async fn get_all_in_process(&self, ctx: Context) -> ProtocolResult<Vec<(Vec<u8>, Vec<u8>)>>;

    async fn remove_in_process(&self, ctx: Context, key: &[u8]) -> ProtocolResult<()>;

    async fn update_monitor_ckb_number(&self, ctx: Context, number: u64) -> ProtocolResult<()>;

    async fn get_monitor_ckb_number(&self, ctx: Context) -> ProtocolResult<u64>;

    async fn nonce(&self, ctx: Context, address: H160) -> ProtocolResult<U256>;

    async fn call_evm(&self, ctx: Context, addr: H160, data: Vec<u8>) -> ProtocolResult<TxResp>;

    async fn insert_record(
        &self,
        ctx: Context,
        reqs: RequestTxHashes,
        block_hash: Hash,
    ) -> ProtocolResult<()>;

    async fn get_record(&self, ctx: Context, reqs: RequestTxHashes)
        -> ProtocolResult<Option<Hash>>;

    async fn current_metadata(&self, ctx: Context) -> Metadata;

    async fn calc_to_ckb_tx(
        &self,
        ctx: Context,
        transfers: &[Transfer],
    ) -> ProtocolResult<TransactionView>;

    fn build_to_ckb_tx(
        &self,
        ctx: Context,
        digest: Hash,
        bls_signature: &BlsSignature,
        bls_pubkey_list: &[BlsPublicKey],
    ) -> ProtocolResult<TransactionView>;

    async fn transmit(
        &self,
        ctx: Context,
        msg: Vec<u8>,
        end: &str,
        target: MessageTarget,
    ) -> ProtocolResult<()>;
}

#[async_trait]
pub trait Crosschain: Send + Sync {
    async fn set_evm_log(
        &self,
        ctx: Context,
        block_number: BlockNumber,
        block_hash: Hash,
        logs: &[Vec<Log>],
    );

    async fn set_checkpoint(&self, ctx: Context, block: Block, proof: Proof);
}
