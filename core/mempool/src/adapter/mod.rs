use super::TxContext;

pub mod message;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::{error::Error, marker::PhantomData, sync::Arc, time::Duration};

use dashmap::DashMap;
use futures::{
    channel::mpsc::{unbounded, TrySendError, UnboundedReceiver, UnboundedSender},
    stream::StreamExt,
};
use log::{debug, error};
use parking_lot::Mutex;

use common_apm_derive::trace_span;
use common_crypto::{Crypto, Secp256k1Recoverable};
use core_executor::{is_call_system_script, AxonExecutor, AxonExecutorAdapter};
use core_interoperation::{get_ckb_transaction_hash, BlockchainType};
use protocol::traits::{
    Context, Executor, Gossip, Interoperation, MemPoolAdapter, MetadataControl, PeerTrust,
    Priority, Rpc, Storage, TrustFeedback,
};
use protocol::types::{
    recover_intact_pub_key, Bytes, Hash, MerkleRoot, SignedTransaction, H160, U256,
};
use protocol::{
    async_trait, codec::ProtocolCodec, lazy::CURRENT_STATE_ROOT, tokio, Display, ProtocolError,
    ProtocolErrorKind, ProtocolResult,
};

use crate::adapter::message::{
    MsgNewTxs, MsgPullTxs, MsgPushTxs, END_GOSSIP_NEW_TXS, RPC_PULL_TXS,
};
use crate::MemPoolError;

struct IntervalTxsBroadcaster;

impl IntervalTxsBroadcaster {
    pub async fn broadcast<G>(
        stx_rx: UnboundedReceiver<SignedTransaction>,
        interval_ms: u64,
        tx_size: usize,
        gossip: G,
        err_tx: UnboundedSender<ProtocolError>,
    ) where
        G: Gossip + Clone + Unpin + 'static,
    {
        let mut stx_rx = stx_rx;
        let mut txs_cache = Vec::with_capacity(tx_size);
        let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                opt_stx = stx_rx.next() => {
                    if let Some(stx) = opt_stx {
                        txs_cache.push(stx);

                        if txs_cache.len() == tx_size {
                            Self::do_broadcast(&mut txs_cache, &gossip, err_tx.clone()).await
                        }
                    } else {
                        debug!("mempool: default mempool adapter dropped")
                    }
                },
                _ = interval.tick() => {
                        Self::do_broadcast(&mut txs_cache, &gossip, err_tx.clone()).await
                },
                else => {
                    break
                }
            };
        }
    }

    async fn do_broadcast<G>(
        txs_cache: &mut Vec<SignedTransaction>,
        gossip: &G,
        err_tx: UnboundedSender<ProtocolError>,
    ) where
        G: Gossip + Unpin,
    {
        if txs_cache.is_empty() {
            return;
        }

        let batch_stxs = txs_cache.drain(..).collect::<Vec<_>>();
        let gossip_msg = MsgNewTxs { batch_stxs };

        let ctx = Context::new();
        let end = END_GOSSIP_NEW_TXS;

        let report_if_err = move |ret: ProtocolResult<()>| {
            if let Err(err) = ret {
                if err_tx.unbounded_send(err).is_err() {
                    error!("mempool: default mempool adapter dropped");
                }
            }
        };

        report_if_err(
            gossip
                .broadcast(ctx, end, gossip_msg, Priority::Normal)
                .await,
        )
    }
}

pub struct DefaultMemPoolAdapter<C, N, S, DB, M, I> {
    network:        N,
    storage:        Arc<S>,
    trie_db:        Arc<DB>,
    metadata:       Arc<M>,
    interoperation: Arc<I>,

    addr_nonce:   DashMap<H160, U256>,
    _timeout_gap: AtomicU64,
    gas_limit:    AtomicU64,
    max_tx_size:  AtomicUsize,
    chain_id:     u64,

    stx_tx: UnboundedSender<SignedTransaction>,
    err_rx: Mutex<UnboundedReceiver<ProtocolError>>,

    pin_c: PhantomData<C>,
}

impl<C, N, S, DB, M, I> DefaultMemPoolAdapter<C, N, S, DB, M, I>
where
    C: Crypto,
    N: Rpc + PeerTrust + Gossip + Clone + Unpin + 'static,
    S: Storage,
    DB: cita_trie::DB + 'static,
    M: MetadataControl + 'static,
    I: Interoperation + 'static,
{
    pub fn new(
        network: N,
        storage: Arc<S>,
        trie_db: Arc<DB>,
        metadata: Arc<M>,
        interoperation: Arc<I>,
        chain_id: u64,
        timeout_gap: u64,
        gas_limit: u64,
        max_tx_size: usize,
        broadcast_txs_size: usize,
        broadcast_txs_interval: u64,
    ) -> Self {
        let (stx_tx, stx_rx) = unbounded();
        let (err_tx, err_rx) = unbounded();

        tokio::spawn(IntervalTxsBroadcaster::broadcast(
            stx_rx,
            broadcast_txs_interval,
            broadcast_txs_size,
            network.clone(),
            err_tx,
        ));

        DefaultMemPoolAdapter {
            network,
            storage,
            trie_db,
            metadata,
            interoperation,

            addr_nonce: DashMap::new(),
            _timeout_gap: AtomicU64::new(timeout_gap),
            gas_limit: AtomicU64::new(gas_limit),
            max_tx_size: AtomicUsize::new(max_tx_size),
            chain_id,

            stx_tx,
            err_rx: Mutex::new(err_rx),

            pin_c: PhantomData,
        }
    }

    async fn check_system_script_tx_authorization(
        &self,
        ctx: Context,
        stx: &SignedTransaction,
    ) -> ProtocolResult<()> {
        let addr = &stx.sender;
        let block = self.storage.get_latest_block(ctx.clone()).await?;
        let metadata = self
            .metadata
            .get_metadata_unchecked(ctx, block.header.number + 1);

        if metadata.verifier_list.iter().any(|ve| &ve.address == addr) {
            return Ok(());
        }

        Err(MemPoolError::CheckAuthorization {
            tx_hash:  stx.transaction.hash,
            err_info: "Invalid system script transaction".to_string(),
        }
        .into())
    }
}

#[async_trait]
impl<C, N, S, DB, M, I> MemPoolAdapter for DefaultMemPoolAdapter<C, N, S, DB, M, I>
where
    C: Crypto + Send + Sync + 'static,
    N: Rpc + PeerTrust + Gossip + Clone + Unpin + 'static,
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
    M: MetadataControl + 'static,
    I: Interoperation + 'static,
{
    #[trace_span(kind = "mempool.adapter", logs = "{txs_len: tx_hashes.len()}")]
    async fn pull_txs(
        &self,
        ctx: Context,
        height: Option<u64>,
        tx_hashes: Vec<Hash>,
    ) -> ProtocolResult<Vec<SignedTransaction>> {
        let pull_msg = MsgPullTxs {
            height,
            hashes: tx_hashes,
        };

        let resp_msg = self
            .network
            .call::<MsgPullTxs, MsgPushTxs>(ctx, RPC_PULL_TXS, pull_msg, Priority::High)
            .await?;

        Ok(resp_msg.sig_txs)
    }

    async fn broadcast_tx(&self, _ctx: Context, stx: SignedTransaction) -> ProtocolResult<()> {
        self.stx_tx
            .unbounded_send(stx)
            .map_err(AdapterError::from)?;

        if let Some(mut err_rx) = self.err_rx.try_lock() {
            match err_rx.try_next() {
                Ok(Some(err)) => return Err(err),
                // Error means receiver channel is empty, is ok here
                Ok(None) | Err(_) => return Ok(()),
            }
        }

        Ok(())
    }

    async fn check_authorization(
        &self,
        ctx: Context,
        tx: &SignedTransaction,
    ) -> ProtocolResult<()> {
        if is_call_system_script(&tx.transaction.unsigned.action) {
            return self.check_system_script_tx_authorization(ctx, tx).await;
        }

        let addr = &tx.sender;
        if let Some(res) = self.addr_nonce.get(addr) {
            if res.value() >= &tx.transaction.unsigned.nonce {
                return Err(MemPoolError::InvalidNonce {
                    current:  res.value().as_u64(),
                    tx_nonce: tx.transaction.unsigned.nonce.as_u64(),
                }
                .into());
            } else {
                return Ok(());
            }
        }

        let backend = AxonExecutorAdapter::from_root(
            **CURRENT_STATE_ROOT.load(),
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            Default::default(),
        )?;

        let account = AxonExecutor::default().get_account(&backend, addr);
        self.addr_nonce.insert(*addr, account.nonce);

        if account.nonce >= tx.transaction.unsigned.nonce {
            return Err(MemPoolError::InvalidNonce {
                current:  account.nonce.as_u64(),
                tx_nonce: tx.transaction.unsigned.nonce.as_u64(),
            }
            .into());
        }

        Ok(())
    }

    async fn check_transaction(&self, ctx: Context, stx: &SignedTransaction) -> ProtocolResult<()> {
        if stx.transaction.signature.is_none() {
            return Err(AdapterError::VerifySignature("missing signature".to_string()).into());
        }

        if stx.public.is_none() {
            return Err(AdapterError::VerifySignature("missing public key".to_string()).into());
        }

        let fixed_bytes = stx.transaction.encode()?;
        let tx_hash = stx.transaction.hash;

        // check tx size
        if fixed_bytes.len() > self.max_tx_size.load(Ordering::SeqCst) {
            if ctx.is_network_origin_txs() {
                self.network.report(
                    ctx,
                    TrustFeedback::Bad(format!("Mempool exceed size limit of tx {:?}", tx_hash)),
                );
            }
            return Err(MemPoolError::ExceedSizeLimit {
                tx_hash,
                max_tx_size: self.max_tx_size.load(Ordering::SeqCst),
                size: fixed_bytes.len(),
            }
            .into());
        }

        // check gas limit
        let gas_limit_tx = stx.transaction.unsigned.gas_limit;
        if gas_limit_tx.as_u64() > self.gas_limit.load(Ordering::SeqCst) {
            if ctx.is_network_origin_txs() {
                self.network.report(
                    ctx,
                    TrustFeedback::Bad(format!("Mempool exceed cycle limit of tx {:?}", tx_hash)),
                );
            }
            return Err(MemPoolError::ExceedGasLimit {
                tx_hash,
                gas_limit_tx: gas_limit_tx.as_u64(),
                gas_limit_config: self.gas_limit.load(Ordering::SeqCst),
            }
            .into());
        }

        // Verify chain id
        if self.chain_id != stx.transaction.chain_id {
            if ctx.is_network_origin_txs() {
                self.network.report(
                    ctx,
                    TrustFeedback::Worse(format!("Mempool wrong chain of tx {:?}", tx_hash)),
                );
            }
            let wrong_chain_id = MemPoolError::WrongChain(tx_hash);

            return Err(wrong_chain_id.into());
        }

        // Verify signature
        let signature = stx.transaction.signature.clone().unwrap();
        match BlockchainType::from(signature.standard_v) {
            BlockchainType::Ethereum => {
                // use original Secp256k1 library to verify
                Secp256k1Recoverable::verify_signature(
                    stx.transaction.signature_hash().as_bytes(),
                    signature.as_bytes().as_ref(),
                    recover_intact_pub_key(&stx.public.unwrap()).as_bytes(),
                )
                .map_err(|err| AdapterError::VerifySignature(err.to_string()))?;
            }
            BlockchainType::Other(blockchain_id) => {
                let tx_hash = get_ckb_transaction_hash(blockchain_id)?;
                let args = [
                    Bytes::from(Vec::from(stx.transaction.signature_hash().to_fixed_bytes())),
                    signature.r,
                    signature.s,
                ];
                self.interoperation
                    .call_ckb_vm(Default::default(), tx_hash, &args, u64::MAX)
                    .map_err(|err| AdapterError::VerifySignature(err.to_string()))?;
            }
        };

        Ok(())
    }

    async fn check_storage_exist(&self, ctx: Context, tx_hash: &Hash) -> ProtocolResult<()> {
        match self.storage.get_transaction_by_hash(ctx, tx_hash).await {
            Ok(Some(_)) => Err(MemPoolError::CommittedTx(*tx_hash).into()),
            Ok(None) => Ok(()),
            Err(err) => Err(err),
        }
    }

    async fn get_latest_height(&self, ctx: Context) -> ProtocolResult<u64> {
        let height = self.storage.get_latest_block_header(ctx).await?.number;
        Ok(height)
    }

    async fn get_transactions_from_storage(
        &self,
        ctx: Context,
        block_height: Option<u64>,
        tx_hashes: &[Hash],
    ) -> ProtocolResult<Vec<Option<SignedTransaction>>> {
        if let Some(height) = block_height {
            self.storage.get_transactions(ctx, height, tx_hashes).await
        } else {
            let futs = tx_hashes
                .iter()
                .map(|tx_hash| self.storage.get_transaction_by_hash(ctx.clone(), tx_hash))
                .collect::<Vec<_>>();
            futures::future::try_join_all(futs).await
        }
    }

    fn set_args(
        &self,
        _context: Context,
        _state_root: MerkleRoot,
        cycles_limit: u64,
        max_tx_size: u64,
    ) {
        self.gas_limit.store(cycles_limit, Ordering::Relaxed);
        self.max_tx_size
            .store(max_tx_size as usize, Ordering::Relaxed);
        self.addr_nonce.clear();
    }

    fn report_good(&self, ctx: Context) {
        if ctx.is_network_origin_txs() {
            self.network.report(ctx, TrustFeedback::Good);
        }
    }
}

#[derive(Debug, Display)]
pub enum AdapterError {
    #[display(fmt = "adapter: interval broadcaster drop")]
    IntervalBroadcasterDrop,

    #[display(fmt = "adapter: internal error")]
    Internal,

    #[display(fmt = "adapter: verify signature error {:?}", _0)]
    VerifySignature(String),
}

impl Error for AdapterError {}

impl<T> From<TrySendError<T>> for AdapterError {
    fn from(_error: TrySendError<T>) -> AdapterError {
        AdapterError::IntervalBroadcasterDrop
    }
}

impl From<AdapterError> for ProtocolError {
    fn from(error: AdapterError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Mempool, Box::new(error))
    }
}

#[cfg(test)]
mod tests {

    use futures::{
        channel::mpsc::{unbounded, UnboundedSender},
        stream::StreamExt,
    };
    use std::sync::Arc;

    use parking_lot::Mutex;

    use protocol::{traits::MessageCodec, types::Bytes};

    use super::*;
    use crate::{adapter::message::MsgNewTxs, tests::default_mock_txs};

    #[derive(Clone)]
    struct MockGossip {
        msgs:      Arc<Mutex<Vec<Bytes>>>,
        signal_tx: UnboundedSender<()>,
    }

    impl MockGossip {
        pub fn new(signal_tx: UnboundedSender<()>) -> Self {
            MockGossip {
                msgs: Default::default(),
                signal_tx,
            }
        }
    }

    #[async_trait]
    impl Gossip for MockGossip {
        async fn broadcast<M>(
            &self,
            _: Context,
            _: &str,
            mut msg: M,
            _: Priority,
        ) -> ProtocolResult<()>
        where
            M: MessageCodec,
        {
            let bytes = msg.encode_msg().expect("encode message fail");
            self.msgs.lock().push(bytes);

            self.signal_tx
                .unbounded_send(())
                .expect("send broadcast signal fail");

            Ok(())
        }

        async fn multicast<'a, M, P>(
            &self,
            _: Context,
            _: &str,
            _: P,
            _: M,
            _: Priority,
        ) -> ProtocolResult<()>
        where
            M: MessageCodec,
            P: AsRef<[Bytes]> + Send + 'a,
        {
            unreachable!()
        }
    }

    macro_rules! pop_msg {
        ($msgs:expr) => {{
            let msg = $msgs.pop().expect("should have one message");
            MsgNewTxs::decode(msg).expect("decode MsgNewTxs fail")
        }};
    }

    #[tokio::test]
    async fn test_interval_broadcast_reach_cache_size() {
        let (stx_tx, stx_rx) = unbounded();
        let (err_tx, _err_rx) = unbounded();
        let tx_size = 10;
        let (broadcast_signal_tx, mut broadcast_signal_rx) = unbounded();
        let gossip = MockGossip::new(broadcast_signal_tx);

        tokio::spawn(IntervalTxsBroadcaster::broadcast(
            stx_rx,
            1000000,
            tx_size,
            gossip.clone(),
            err_tx,
        ));

        for stx in default_mock_txs(11).into_iter() {
            stx_tx.unbounded_send(stx).expect("send stx fail");
        }

        broadcast_signal_rx.next().await;
        let mut msgs = gossip.msgs.lock().drain(..).collect::<Vec<_>>();
        assert_eq!(msgs.len(), 1, "should only have one message");

        let msg = pop_msg!(msgs);
        assert_eq!(msg.batch_stxs.len(), 10, "should only have 10 stx");
    }

    #[tokio::test]
    async fn test_interval_broadcast_reach_interval() {
        let (stx_tx, stx_rx) = unbounded();
        let (err_tx, _err_rx) = unbounded();
        let tx_size = 10;
        let (broadcast_signal_tx, mut broadcast_signal_rx) = unbounded();
        let gossip = MockGossip::new(broadcast_signal_tx);

        tokio::spawn(IntervalTxsBroadcaster::broadcast(
            stx_rx,
            200,
            tx_size,
            gossip.clone(),
            err_tx,
        ));

        for stx in default_mock_txs(9).into_iter() {
            stx_tx.unbounded_send(stx).expect("send stx fail");
        }

        broadcast_signal_rx.next().await;
        let mut msgs = gossip.msgs.lock().drain(..).collect::<Vec<_>>();
        assert_eq!(msgs.len(), 1, "should only have one message");

        let msg = pop_msg!(msgs);
        assert_eq!(msg.batch_stxs.len(), 9, "should only have 9 stx");
    }

    #[tokio::test]
    async fn test_interval_broadcast() {
        let (stx_tx, stx_rx) = unbounded();
        let (err_tx, _err_rx) = unbounded();
        let tx_size = 10;
        let (broadcast_signal_tx, mut broadcast_signal_rx) = unbounded();
        let gossip = MockGossip::new(broadcast_signal_tx);

        tokio::spawn(IntervalTxsBroadcaster::broadcast(
            stx_rx,
            200,
            tx_size,
            gossip.clone(),
            err_tx,
        ));

        for stx in default_mock_txs(19).into_iter() {
            stx_tx.unbounded_send(stx).expect("send stx fail");
        }

        // Should got two broadcast
        broadcast_signal_rx.next().await;
        broadcast_signal_rx.next().await;

        let mut msgs = gossip.msgs.lock().drain(..).collect::<Vec<_>>();
        assert_eq!(msgs.len(), 2, "should only have two messages");

        let msg = pop_msg!(msgs);
        assert_eq!(
            msg.batch_stxs.len(),
            9,
            "last message should only have 9 stx"
        );

        let msg = pop_msg!(msgs);
        assert_eq!(
            msg.batch_stxs.len(),
            10,
            "first message should only have 10 stx"
        );
    }
}
