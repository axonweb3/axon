use super::TxContext;

pub mod message;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::{collections::HashMap, error::Error, marker::PhantomData, sync::Arc, time::Duration};

use ckb_types::core::cell::{CellProvider, CellStatus};
use ckb_types::{core::TransactionView, prelude::*};
use dashmap::DashMap;
use futures::{
    channel::mpsc::{unbounded, TrySendError, UnboundedReceiver, UnboundedSender},
    stream::StreamExt,
};
use log::{debug, error};
use parking_lot::Mutex;

use protocol::traits::{
    Context, Executor, Gossip, Interoperation, MemPoolAdapter, MetadataControl, PeerTrust,
    Priority, Rpc, Storage, TrustFeedback,
};
use protocol::types::{
    recover_intact_pub_key, AddressSource, BatchSignedTxs, CellDepWithPubKey, CellWithData, Hash,
    Hasher, MerkleRoot, SignatureComponents, SignatureR, SignatureS, SignedTransaction, H160, U256,
};
use protocol::{
    async_trait, ckb_blake2b_256, codec::ProtocolCodec, lazy::CURRENT_STATE_ROOT, tokio, trie,
    Display, ProtocolError, ProtocolErrorKind, ProtocolResult,
};

use common_apm_derive::trace_span;
use common_crypto::{Crypto, Secp256k1Recoverable};
use core_executor::{
    is_call_system_script, system_contract::DataProvider, AxonExecutor, AxonExecutorAdapter,
};
use core_interoperation::{utils::is_dummy_out_point, InteroperationImpl};

use crate::adapter::message::{MsgPullTxs, END_GOSSIP_NEW_TXS, RPC_PULL_TXS};
use crate::MemPoolError;

const MAX_VERIFY_CKB_VM_CYCLES: u64 = 50_000_000;

struct IntervalTxsBroadcaster;

impl IntervalTxsBroadcaster {
    pub async fn broadcast<G>(
        stx_rx: UnboundedReceiver<(Option<usize>, SignedTransaction)>,
        interval_ms: u64,
        tx_size: usize,
        gossip: G,
        err_tx: UnboundedSender<ProtocolError>,
    ) where
        G: Gossip + Clone + Unpin + 'static,
    {
        let mut stx_rx = stx_rx;
        let mut txs_cache: HashMap<_, Vec<SignedTransaction>> = HashMap::with_capacity(10);
        let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                opt_stx = stx_rx.next() => {
                    if let Some((origin, stx)) = opt_stx {
                        txs_cache.entry(origin).or_default().push(stx);

                        let len: usize = {
                            txs_cache.values().map(|v| v.len()).sum()
                        };

                        if len == tx_size {
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
        txs_cache: &mut HashMap<Option<usize>, Vec<SignedTransaction>>,
        gossip: &G,
        err_tx: UnboundedSender<ProtocolError>,
    ) where
        G: Gossip + Unpin,
    {
        if txs_cache.is_empty() {
            return;
        }

        let report_if_err = move |ret: ProtocolResult<()>| {
            if let Err(err) = ret {
                if err_tx.unbounded_send(err).is_err() {
                    error!("mempool: default mempool adapter dropped");
                }
            }
        };

        for (origin, batch_stxs) in txs_cache.drain() {
            let gossip_msg = BatchSignedTxs(batch_stxs);

            let ctx = Context::new();
            let end = END_GOSSIP_NEW_TXS;

            report_if_err(
                gossip
                    .gossip(ctx, origin, end, gossip_msg, Priority::Normal)
                    .await,
            )
        }
    }
}

pub struct DefaultMemPoolAdapter<C, N, S, DB, M, I> {
    network:  N,
    storage:  Arc<S>,
    trie_db:  Arc<DB>,
    metadata: Arc<M>,

    addr_nonce:  DashMap<H160, (U256, U256)>,
    gas_limit:   AtomicU64,
    max_tx_size: AtomicUsize,
    chain_id:    u64,

    stx_tx: UnboundedSender<(Option<usize>, SignedTransaction)>,
    err_rx: Mutex<UnboundedReceiver<ProtocolError>>,

    pin_c: PhantomData<C>,
    pin_i: PhantomData<I>,
}

impl<C, N, S, DB, M, I> DefaultMemPoolAdapter<C, N, S, DB, M, I>
where
    C: Crypto,
    N: Rpc + PeerTrust + Gossip + Clone + Unpin + 'static,
    S: Storage,
    DB: trie::DB + 'static,
    M: MetadataControl + 'static,
    I: Interoperation + 'static,
{
    pub fn new(
        network: N,
        storage: Arc<S>,
        trie_db: Arc<DB>,
        metadata: Arc<M>,
        chain_id: u64,
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

            addr_nonce: DashMap::new(),
            gas_limit: AtomicU64::new(gas_limit),
            max_tx_size: AtomicUsize::new(max_tx_size),
            chain_id,

            stx_tx,
            err_rx: Mutex::new(err_rx),

            pin_c: PhantomData,
            pin_i: PhantomData,
        }
    }

    async fn check_system_script_tx_authorization(
        &self,
        ctx: Context,
        stx: &SignedTransaction,
    ) -> ProtocolResult<U256> {
        let addr = &stx.sender;
        let block = self.storage.get_latest_block(ctx.clone()).await?;
        let metadata = self
            .metadata
            .get_metadata_unchecked(ctx, block.header.number + 1);

        if metadata.verifier_list.iter().any(|ve| &ve.address == addr) {
            return Ok(U256::zero());
        }

        Err(MemPoolError::CheckAuthorization {
            tx_hash:  stx.transaction.hash,
            err_info: "Invalid system script transaction".to_string(),
        }
        .into())
    }

    fn verify_cell_mapping_sender(
        &self,
        sender: H160,
        ckb_tx_view: &TransactionView,
        dummy_input: Option<CellWithData>,
        address_source: AddressSource,
    ) -> ProtocolResult<()> {
        let input = ckb_tx_view
            .inputs()
            .get(address_source.index as usize)
            .ok_or(MemPoolError::InvalidAddressSource(address_source))?;

        log::debug!("[mempool]: verify interoperation tx sender \ntx view \n{:?}\ndummy input\n {:?}\naddress source\n{:?}\n", ckb_tx_view, dummy_input, address_source);

        if is_dummy_out_point(&input.previous_output()) {
            log::debug!("[mempool]: verify interoperation tx dummy input mode.");

            if let Some(cell) = dummy_input {
                if address_source.type_ == 1 && cell.type_script.is_none() {
                    return Err(MemPoolError::InvalidAddressSource(address_source).into());
                }

                let script_hash = if address_source.type_ == 0 {
                    cell.lock_script_hash()
                } else {
                    cell.type_script_hash().unwrap()
                };

                let expect_sender: H160 = Hasher::digest(script_hash).into();
                if expect_sender != sender {
                    return Err(MemPoolError::InvalidSender {
                        expect: expect_sender,
                        actual: sender,
                    }
                    .into());
                }

                return Ok(());
            }

            return Err(MemPoolError::InvalidDummyInput.into());
        }

        log::debug!("[mempool]: verify interoperation tx reality input mode.");
        match DataProvider.cell(&input.previous_output(), true) {
            CellStatus::Live(cell) => {
                let script_hash = if address_source.type_ == 0 {
                    ckb_blake2b_256(cell.cell_output.lock().as_slice())
                } else if let Some(type_script) = cell.cell_output.type_().to_opt() {
                    ckb_blake2b_256(type_script.as_slice())
                } else {
                    return Err(MemPoolError::InvalidAddressSource(address_source).into());
                };

                let expect_sender: H160 = Hasher::digest(script_hash).into();
                if expect_sender != sender {
                    return Err(MemPoolError::InvalidSender {
                        expect: expect_sender,
                        actual: sender,
                    }
                    .into());
                }

                Ok(())
            }
            _ => Err(MemPoolError::InvalidAddressSource(address_source).into()),
        }
    }

    fn verify_chain_id(&self, ctx: Context, stx: &SignedTransaction) -> ProtocolResult<()> {
        if self.chain_id != stx.transaction.chain_id {
            if ctx.is_network_origin_txs() {
                self.network.report(
                    ctx,
                    TrustFeedback::Worse(format!(
                        "Mempool wrong chain of tx {:?}",
                        stx.transaction.hash
                    )),
                );
            }
            let wrong_chain_id = MemPoolError::WrongChain(stx.transaction.hash);

            return Err(wrong_chain_id.into());
        }

        Ok(())
    }

    fn verify_tx_size(&self, ctx: Context, stx: &SignedTransaction) -> ProtocolResult<()> {
        let fixed_bytes = stx.transaction.encode()?;
        if fixed_bytes.len() > self.max_tx_size.load(Ordering::Acquire) {
            if ctx.is_network_origin_txs() {
                self.network.report(
                    ctx,
                    TrustFeedback::Bad(format!(
                        "Mempool exceed size limit of tx {:?}",
                        stx.transaction.hash
                    )),
                );
            }
            return Err(MemPoolError::ExceedSizeLimit {
                tx_hash:     stx.transaction.hash,
                max_tx_size: self.max_tx_size.load(Ordering::Acquire),
                size:        fixed_bytes.len(),
            }
            .into());
        }

        Ok(())
    }

    fn verify_gas_price(&self, stx: &SignedTransaction) -> ProtocolResult<()> {
        let gas_price = stx.transaction.unsigned.gas_price();
        if gas_price == U256::zero() || gas_price >= U256::from(u64::MAX) {
            return Err(MemPoolError::InvalidGasPrice(gas_price).into());
        }

        Ok(())
    }

    fn verify_gas_limit(&self, ctx: Context, stx: &SignedTransaction) -> ProtocolResult<()> {
        let gas_limit_tx = stx.transaction.unsigned.gas_limit();
        if gas_limit_tx > &U256::from(self.gas_limit.load(Ordering::Acquire)) {
            if ctx.is_network_origin_txs() {
                self.network.report(
                    ctx,
                    TrustFeedback::Bad(format!(
                        "Mempool exceed cycle limit of tx {:?}",
                        stx.transaction.hash
                    )),
                );
            }
            return Err(MemPoolError::ExceedGasLimit {
                tx_hash:          stx.transaction.hash,
                gas_limit_tx:     gas_limit_tx.as_u64(),
                gas_limit_config: self.gas_limit.load(Ordering::Acquire),
            }
            .into());
        }

        Ok(())
    }
}

#[async_trait]
impl<C, N, S, DB, M, I> MemPoolAdapter for DefaultMemPoolAdapter<C, N, S, DB, M, I>
where
    C: Crypto + Send + Sync + 'static,
    N: Rpc + PeerTrust + Gossip + Clone + Unpin + 'static,
    S: Storage + 'static,
    DB: trie::DB + 'static,
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
            .call::<MsgPullTxs, BatchSignedTxs>(ctx, RPC_PULL_TXS, pull_msg, Priority::High)
            .await?;

        Ok(resp_msg.inner())
    }

    async fn broadcast_tx(
        &self,
        _ctx: Context,
        origin: Option<usize>,
        stx: SignedTransaction,
    ) -> ProtocolResult<()> {
        self.stx_tx
            .unbounded_send((origin, stx))
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
    ) -> ProtocolResult<U256> {
        if is_call_system_script(tx.transaction.unsigned.action()) {
            return self.check_system_script_tx_authorization(ctx, tx).await;
        }

        let addr = &tx.sender;
        if let Some(res) = self.addr_nonce.get(addr) {
            if tx.transaction.unsigned.nonce() < &res.value().0 {
                return Err(MemPoolError::InvalidNonce {
                    current:  res.value().0.as_u64(),
                    tx_nonce: tx.transaction.unsigned.nonce().as_u64(),
                }
                .into());
            } else if res.value().1 < tx.transaction.unsigned.may_cost() {
                return Err(MemPoolError::ExceedBalance {
                    tx_hash:         tx.transaction.hash,
                    account_balance: res.value().1,
                    tx_gas_limit:    *tx.transaction.unsigned.gas_limit(),
                }
                .into());
            } else {
                return Ok(tx.transaction.unsigned.nonce() - res.value().0);
            }
        }

        let backend = AxonExecutorAdapter::from_root(
            **CURRENT_STATE_ROOT.load(),
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            Default::default(),
        )?;

        let account = AxonExecutor::default().get_account(&backend, addr);
        self.addr_nonce
            .insert(*addr, (account.nonce, account.balance));

        if &account.nonce > tx.transaction.unsigned.nonce() {
            return Err(MemPoolError::InvalidNonce {
                current:  account.nonce.as_u64(),
                tx_nonce: tx.transaction.unsigned.nonce().as_u64(),
            }
            .into());
        }

        if account.balance < tx.transaction.unsigned.may_cost() {
            return Err(MemPoolError::ExceedBalance {
                tx_hash:         tx.transaction.hash,
                account_balance: account.balance,
                tx_gas_limit:    *tx.transaction.unsigned.gas_limit(),
            }
            .into());
        }

        Ok(tx.transaction.unsigned.nonce() - account.nonce)
    }

    async fn check_transaction(&self, ctx: Context, stx: &SignedTransaction) -> ProtocolResult<()> {
        if stx.transaction.signature.is_none() {
            return Err(AdapterError::VerifySignature("missing signature".to_string()).into());
        }

        if stx.public.is_none() {
            return Err(AdapterError::VerifySignature("missing public key".to_string()).into());
        }

        self.verify_chain_id(ctx.clone(), stx)?;
        self.verify_tx_size(ctx.clone(), stx)?;
        self.verify_gas_price(stx)?;
        self.verify_gas_limit(ctx, stx)?;

        // Verify signature
        let signature = stx.transaction.signature.clone().unwrap();
        if signature.len() == SignatureComponents::ETHEREUM_TX_LEN {
            // use original Secp256k1 library to verify
            Secp256k1Recoverable::verify_signature(
                stx.transaction.signature_hash(true).as_bytes(),
                signature.as_bytes().as_ref(),
                recover_intact_pub_key(&stx.public.unwrap()).as_bytes(),
            )
            .map_err(|err| AdapterError::VerifySignature(err.to_string()))?;

            return Ok(());
        }

        match signature.r[0] {
            0u8 => {
                let r = rlp::decode::<CellDepWithPubKey>(&signature.r[1..])
                    .map_err(AdapterError::Rlp)?;

                InteroperationImpl::call_ckb_vm(
                    Default::default(),
                    &DataProvider::default(),
                    r.cell_dep,
                    &[r.pub_key, signature.s],
                    u64::MAX,
                )
                .map_err(|e| AdapterError::VerifySignature(e.to_string()))?;
            }
            _ => {
                let r = SignatureR::decode(&signature.r)?;
                let s = SignatureS::decode(&signature.s)?;

                if r.inputs_len() != s.witnesses.len() {
                    return Err(AdapterError::VerifySignature(
                        "signature item mismatch".to_string(),
                    )
                    .into());
                }

                let dummy_ckb_tx = InteroperationImpl::dummy_transaction(r.clone(), s);
                let dummy_input = r.dummy_input();

                self.verify_cell_mapping_sender(
                    stx.sender,
                    &dummy_ckb_tx,
                    dummy_input.clone(),
                    r.address_source(),
                )?;

                InteroperationImpl::verify_by_ckb_vm(
                    Default::default(),
                    &DataProvider::default(),
                    &dummy_ckb_tx,
                    dummy_input,
                    MAX_VERIFY_CKB_VM_CYCLES,
                )
                .map_err(|e| AdapterError::VerifySignature(e.to_string()))?;
            }
        }

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
        self.gas_limit.store(cycles_limit, Ordering::Release);
        self.max_tx_size
            .store(max_tx_size as usize, Ordering::Release);
        self.addr_nonce.clear();
    }

    fn clear_nonce_cache(&self) {
        self.addr_nonce.clear()
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

    #[display(fmt = "adapter: rlp decode error {:?}", _0)]
    Rlp(rlp::DecoderError),
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
    use super::*;

    use std::sync::Arc;

    use futures::{
        channel::mpsc::{unbounded, UnboundedSender},
        stream::StreamExt,
    };
    use parking_lot::Mutex;

    use protocol::{traits::MessageCodec, types::Bytes};

    use crate::tests::default_mock_txs;

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

        async fn gossip<M>(
            &self,
            _: Context,
            _: Option<usize>,
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
            BatchSignedTxs::decode_msg(msg).expect("decode MsgNewTxs fail")
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
            stx_tx.unbounded_send((None, stx)).expect("send stx fail");
        }

        broadcast_signal_rx.next().await;
        let mut msgs = gossip.msgs.lock().drain(..).collect::<Vec<_>>();
        assert_eq!(msgs.len(), 1, "should only have one message");

        let msg = pop_msg!(msgs);
        assert_eq!(msg.0.len(), 10, "should only have 10 stx");
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
            stx_tx.unbounded_send((None, stx)).expect("send stx fail");
        }

        broadcast_signal_rx.next().await;
        let mut msgs = gossip.msgs.lock().drain(..).collect::<Vec<_>>();
        assert_eq!(msgs.len(), 1, "should only have one message");

        let msg = pop_msg!(msgs);
        assert_eq!(msg.0.len(), 9, "should only have 9 stx");
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
            stx_tx.unbounded_send((None, stx)).expect("send stx fail");
        }

        // Should got two broadcast
        broadcast_signal_rx.next().await;
        broadcast_signal_rx.next().await;

        let mut msgs = gossip.msgs.lock().drain(..).collect::<Vec<_>>();
        assert_eq!(msgs.len(), 2, "should only have two messages");

        let msg = pop_msg!(msgs);
        assert_eq!(msg.0.len(), 9, "last message should only have 9 stx");

        let msg = pop_msg!(msgs);
        assert_eq!(msg.0.len(), 10, "first message should only have 10 stx");
    }
}
