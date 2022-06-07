#![allow(dead_code, unused_variables, clippy::derive_partial_eq_without_eq)]

mod adapter;
mod codec;
mod error;
mod generated;
mod monitor;
mod task;
mod types;

pub use adapter::DefaultCrossAdapter;

use std::sync::Arc;

use arc_swap::ArcSwap;
use ckb_types::{core::TransactionView, prelude::*};

use common_crypto::{
    PrivateKey, PublicKey, Secp256k1RecoverablePrivateKey, Signature, ToPublicKey,
};
use protocol::tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use protocol::traits::{Context, CrossAdapter, CrossChain};
use protocol::types::{
    Block, BlockNumber, Hash, Hasher, Log, Proof, SignedTransaction, Transaction,
    TransactionAction, TypesError, UnverifiedTransaction, H160, MAX_BLOCK_GAS_LIMIT, U256,
};
use protocol::{async_trait, lazy::CHAIN_ID, tokio, ProtocolResult};

use core_executor::CROSSCHAIN_CONTRACT_ADDRESS;

use crate::error::CrossChainError;
use crate::types::Requests;

pub const CKB_BLOCK_INTERVAL: u64 = 8; // second
pub const NON_FORK_BLOCK_GAP: u64 = 24;

lazy_static::lazy_static! {
    pub static ref CKB_TIP: ArcSwap<u64> = ArcSwap::from_pointee(0);
}

pub struct CrossChainImpl<Adapter> {
    priv_key: Secp256k1RecoverablePrivateKey,
    address:  H160,
    log_rx:   UnboundedReceiver<Vec<Log>>,
    req_rx:   UnboundedReceiver<Vec<TransactionView>>,

    adapter: Arc<Adapter>,
}

#[async_trait]
impl<Adapter: CrossAdapter + 'static> CrossChain for CrossChainImpl<Adapter> {
    async fn set_evm_log(
        &self,
        ctx: Context,
        block_number: BlockNumber,
        block_hash: Hash,
        logs: &[Vec<Log>],
    ) {
        for tx_logs in logs.iter() {
            if let Err(e) = self.log_tx.send(tx_logs.clone()) {
                log::error!("[cross-chain]: send log to process error {:?}", e);
            }
        }
    }

    async fn set_checkpoint(&self, ctx: Context, block: Block, proof: Proof) {}
}

impl<Adapter: CrossAdapter + 'static> CrossChainImpl<Adapter> {
    pub fn new(priv_key: Secp256k1RecoverablePrivateKey, adapter: Arc<Adapter>) -> Self {
        let address: H160 = Hasher::digest(priv_key.pub_key().to_bytes()).into();
        let (log_tx, log_rx) = unbounded_channel();
        let (req_tx, req_rx) = unbounded_channel();

        CrossChainImpl {
            priv_key,
            address,
            log_rx,
            req_rx,
            adapter,
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some(logs) = self.log_rx.recv() => {
                    let adapter_clone = Arc::clone(&self.adapter);

                    tokio::spawn(async move {
                        match build_ckb_txs(logs).await {
                            Ok((reqs, stx)) => {
                                let ctx = Context::new();
                                adapter_clone.insert_in_process(
                                    ctx.clone(),
                                    &rlp::encode(&reqs).freeze(),
                                    stx.pack().as_slice()
                                );
                                adapter_clone.send_ckb_tx(ctx, stx.into()).await;
                            },

                            Err(e) => log::error!("[crosschain]: crosschain error {:?}", e),
                        };

                    });
                }

                Some(reqs) = self.req_rx.recv() => {
                    let (reqs, stx) = match self.build_axon_txs(reqs).await {
                        Ok(res) => res,
                        Err(e) => {
                            log::error!("[crosschain]: crosschain error {:?}", e);
                            continue;
                        }
                    };

                    self.adapter.insert_in_process(
                        Context::new(),
                        &rlp::encode(&reqs).freeze(),
                        &rlp::encode(&stx).freeze()
                    );
                }
            }
        }
    }

    async fn build_axon_txs(
        &self,
        txs: Vec<TransactionView>,
    ) -> ProtocolResult<(Requests, SignedTransaction)> {
        let tx = Transaction {
            nonce:                    self.adapter.nonce(self.address).await?,
            max_priority_fee_per_gas: U256::one(),
            gas_price:                U256::one(),
            gas_limit:                MAX_BLOCK_GAS_LIMIT.into(),
            action:                   TransactionAction::Call(CROSSCHAIN_CONTRACT_ADDRESS),
            value:                    U256::zero(),
            data:                     Default::default(),
            access_list:              vec![],
        };

        let id = **CHAIN_ID.load();
        let signature = self.priv_key.sign_message(
            &Hasher::digest(tx.encode(id, None))
                .as_bytes()
                .try_into()
                .unwrap(),
        );

        let utx = UnverifiedTransaction {
            unsigned:  tx,
            signature: Some(signature.to_bytes().into()),
            chain_id:  id,
            hash:      Default::default(),
        }
        .calc_hash();

        let stx: SignedTransaction = utx
            .try_into()
            .map_err(|e: TypesError| CrossChainError::Adapter(e.to_string()))?;

        todo!()
    }
}

async fn build_ckb_txs(logs: Vec<Log>) -> ProtocolResult<(Requests, TransactionView)> {
    todo!()
}
