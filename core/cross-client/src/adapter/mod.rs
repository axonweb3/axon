mod db;

use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use ckb_jsonrpc_types::OutputsValidator;
use ckb_types::{
    core::{BlockNumber, BlockView, TransactionView},
    packed,
    prelude::*,
};
use ethabi::RawLog;

use common_config_parser::types::{Config, ConfigCrossChain};
use common_crypto::{
    Crypto, PrivateKey, Secp256k1Recoverable, Secp256k1RecoverablePrivateKey, Signature,
    ToPublicKey, UncompressedPublicKey,
};
use core_executor::{AxonExecutor, AxonExecutorAdapter};
use protocol::traits::{
    CkbClient, Context, CrossAdapter, CrossChain, Executor, ExecutorAdapter, MemPool, Storage,
};
use protocol::types::{
    public_to_address, Account, Block, Bytes, CrossChainTransferPayload, ExecutorContext, Hash,
    Identity, Log, Proof, Proposal, Public, SignedTransaction, SubmitCheckpointPayload,
    Transaction, TransactionAction, UnverifiedTransaction, H160, H256, U256,
};
use protocol::{
    async_trait,
    codec::{hex_encode, ProtocolCodec},
    lazy::{CHAIN_ID, CURRENT_STATE_ROOT},
    tokio::{self, sync::mpsc},
    ProtocolResult,
};

ethabi_contract::use_contract!(asset, "./src/adapter/abi/asset.abi");

use asset::events as asset_events;
use asset::functions as asset_functions;
use asset::logs::Burned;

const TWO_THOUSAND: u64 = 2000;

trait CrossChainDB: Sync + Send {
    fn get(&self, key: &[u8]) -> ProtocolResult<Option<Vec<u8>>>;

    fn get_all(&self) -> ProtocolResult<Vec<(Vec<u8>, Vec<u8>)>>;

    fn insert(&self, key: &[u8], val: &[u8]) -> ProtocolResult<()>;

    fn remove(&self, key: &[u8]) -> ProtocolResult<()>;
}

pub struct DefaultCrossChainAdapter<M, S, TrieDB, DB> {
    mempool: Arc<M>,
    storage: Arc<S>,
    trie_db: Arc<TrieDB>,
    db:      Arc<DB>,
}

#[async_trait]
impl<M, S, TrieDB, DB> CrossAdapter for DefaultCrossChainAdapter<M, S, TrieDB, DB>
where
    M: MemPool + 'static,
    S: Storage + 'static,
    TrieDB: cita_trie::DB + 'static,
    DB: CrossChainDB + 'static,
{
    async fn send_axon_tx(&self, ctx: Context, stx: SignedTransaction) -> ProtocolResult<()> {
        self.mempool.insert(ctx, stx).await
    }

    async fn send_ckb_tx(
        &self,
        ctx: Context,
        tx: ckb_jsonrpc_types::TransactionView,
    ) -> ProtocolResult<()> {
        Ok(())
    }

    async fn insert_in_process(&self, ctx: Context, key: &[u8], val: &[u8]) -> ProtocolResult<()> {
        self.db.insert(key, val)
    }

    async fn get_all_in_process(&self, ctx: Context) -> ProtocolResult<Vec<(Vec<u8>, Vec<u8>)>> {
        self.db.get_all()
    }

    async fn remove_in_process(&self, ctx: Context, key: &[u8]) -> ProtocolResult<()> {
        self.db.remove(key)
    }

    async fn nonce(&self, address: H160) -> ProtocolResult<U256> {
        Ok(match self.evm_backend().await?.get(address.as_bytes()) {
            Some(bytes) => Account::decode(bytes)?.nonce,
            None => U256::zero(),
        })
    }
}

impl<M, S, TrieDB, DB> DefaultCrossChainAdapter<M, S, TrieDB, DB>
where
    M: MemPool + 'static,
    S: Storage + 'static,
    TrieDB: cita_trie::DB + 'static,
    DB: CrossChainDB + 'static,
{
    pub async fn evm_backend(&self) -> ProtocolResult<AxonExecutorAdapter<S, TrieDB>> {
        let block = self.storage.get_latest_block(Context::new()).await?;
        let state_root = block.header.state_root;
        let proposal: Proposal = block.into();

        AxonExecutorAdapter::from_root(
            state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            ExecutorContext::from(proposal),
        )
    }
}

pub struct DefaultCrossAdapter<M, S, DB, C> {
    priv_key:       Secp256k1RecoverablePrivateKey,
    config:         ConfigCrossChain,
    tip_number:     BlockNumber,
    current_number: BlockNumber,
    block_recv:     mpsc::Receiver<Vec<ProtocolResult<BlockView>>>,
    block_sender:   mpsc::Sender<Vec<ProtocolResult<BlockView>>>,
    start_fetch:    bool,
    backup_dir:     PathBuf,

    mempool:    Arc<M>,
    storage:    Arc<S>,
    trie_db:    Arc<DB>,
    ckb_client: Arc<C>,
}

#[async_trait]
impl<M, S, DB, C> CrossAdapter for DefaultCrossAdapter<M, S, DB, C>
where
    M: MemPool + 'static,
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
    C: CkbClient + 'static,
{
    async fn send_axon_tx(&self, ctx: Context, stx: SignedTransaction) -> ProtocolResult<()> {
        self.mempool.insert(ctx, stx).await
    }

    async fn send_ckb_tx(
        &self,
        ctx: Context,
        tx: ckb_jsonrpc_types::TransactionView,
    ) -> ProtocolResult<()> {
        Ok(())
    }

    async fn insert_in_process(&self, ctx: Context, key: &[u8], val: &[u8]) -> ProtocolResult<()> {
        Ok(())
    }

    async fn get_all_in_process(&self, ctx: Context) -> ProtocolResult<Vec<(Vec<u8>, Vec<u8>)>> {
        Ok(vec![])
    }

    async fn remove_in_process(&self, ctx: Context, key: &[u8]) -> ProtocolResult<()> {
        Ok(())
    }

    async fn nonce(&self, address: H160) -> ProtocolResult<U256> {
        Ok(U256::zero())
    }
}

impl<M, S, DB, C> DefaultCrossAdapter<M, S, DB, C>
where
    M: MemPool + 'static,
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
    C: CkbClient + 'static,
{
    pub fn new(
        config: Config,
        pk: Secp256k1RecoverablePrivateKey,
        mempool: Arc<M>,
        storage: Arc<S>,
        trie_db: Arc<DB>,
        ckb_client: Arc<C>,
    ) -> Self {
        let backup_dir = config.data_path.join("cross_client");
        let (sender, recv) = mpsc::channel(256);
        Self {
            priv_key: pk,
            tip_number: 0,
            current_number: std::cmp::max(
                load_current_number(backup_dir.as_path()),
                config.cross_client.start_block_number,
            ),
            config: config.cross_client,
            block_recv: recv,
            block_sender: sender,
            start_fetch: true,
            backup_dir,

            mempool,
            storage,
            trie_db,
            ckb_client,
        }
    }

    pub fn handle(&self) -> CrossAdapterHandle<C> {
        CrossAdapterHandle {
            client: Arc::<C>::clone(&self.ckb_client),
            config: self.config.clone(),
            pk:     Secp256k1RecoverablePrivateKey::try_from(self.config.pk.as_bytes().as_ref())
                .unwrap(),
        }
    }

    pub async fn run(mut self) {
        self.update_tip_number().await;

        let mut interval =
            tokio::time::interval_at(tokio::time::Instant::now(), Duration::from_secs(8));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.update_tip_number().await;
                },
                res = self.block_recv.recv() => {
                    if let Some(blocks) = res {
                        self.handle_blocks(blocks).await;
                    } else {
                        break
                    }
                }
                else => {
                    break
                }
            }
        }
    }

    async fn update_tip_number(&mut self) {
        let tip_header = self
            .ckb_client
            .get_tip_header(Context::new())
            .await
            .unwrap();

        self.tip_number = tip_header.inner.number.into();

        if self.tip_number < self.current_number {
            self.current_number = 0
        }

        self.fetch_block().await
    }

    async fn fetch_block(&mut self) {
        if self.start_fetch && self.tip_number - self.current_number > 24 {
            self.start_fetch = false;

            let mut tasks = Vec::new();
            for i in self.current_number + 1
                ..=std::cmp::min(
                    self.tip_number.saturating_sub(24),
                    self.current_number + 200,
                )
            {
                let task = self
                    .ckb_client
                    .get_block_by_number(Context::new(), i.into());
                let handle = tokio::spawn(task);
                tasks.push(handle)
            }
            let sender = self.block_sender.clone();
            tokio::spawn(async move {
                let mut list = Vec::with_capacity(tasks.len());
                for j in tasks {
                    let res = j.await.unwrap().map(|b| b.into());
                    list.push(res)
                }
                sender.send(list).await.unwrap();
            });
        } else {
            self.start_fetch = true;
        }
    }

    async fn handle_blocks(&mut self, blocks: Vec<ProtocolResult<BlockView>>) {
        for res in blocks {
            match res {
                Ok(block) => self.search_tx(block).await,
                Err(e) => {
                    let number = self.current_number + 1;
                    log::info!("get block {} error: {}", number, e);
                    loop {
                        if let Ok(block) = self
                            .ckb_client
                            .get_block_by_number(Context::new(), number.into())
                            .await
                        {
                            self.search_tx(block.into()).await
                        }
                    }
                }
            }
        }

        if let Err(e) = self.dump_current_number().await {
            log::debug!("dump current number error: {}", e);
        }
        self.fetch_block().await;
    }

    async fn search_tx(&mut self, block: BlockView) {
        self.current_number = block.number();

        log::info!("current block number : {:?}", block.number());

        let txs = block.transactions();
        for tx in txs {
            let inputs = tx.inputs();
            let outputs = tx.outputs();
            let witnesses = tx.witnesses();
            if outputs.len() != 2 || inputs.len() != 2 || witnesses.len() != 3 {
                continue;
            }

            let (output_1, _) = tx.output_with_data(0).unwrap();
            let (output_2, data_2) = tx.output_with_data(1).unwrap();

            if output_1.type_().is_none() || output_2.type_().is_none() {
                continue;
            }

            let output_1_type_hash = output_1.type_().to_opt().unwrap().calc_script_hash();
            let output_2_type_hash = output_2.type_().to_opt().unwrap().calc_script_hash();

            if output_1_type_hash.as_slice() != self.config.axon_udt_hash.as_bytes()
                && output_2_type_hash.as_slice() != self.config.axon_udt_hash.as_bytes()
            {
                continue;
            }

            let output_amount = get_amount(data_2);

            let input_point = inputs.get(1).unwrap().previous_output();

            let (hash, index) = (input_point.tx_hash(), input_point.index());

            let input_tx = self
                .ckb_client
                .get_transaction(Context::new(), &hash.unpack())
                .await
                .expect("get previous tx fail")
                .unwrap();

            let tx_view: TransactionView =
                Into::<packed::Transaction>::into(input_tx.transaction.unwrap().inner).into_view();

            let (_, data) = tx_view.output_with_data(index.unpack()).unwrap();

            let input_amount = get_amount(data);

            log::info!("search tx hash: {:?}", hex_encode(&tx.hash().raw_data()));

            self.send_axon_tx(
                input_amount.checked_sub(output_amount),
                witnesses.get(2).unwrap().raw_data(),
                tx.hash().raw_data().to_vec(),
            )
            .await;
        }
    }

    async fn send_axon_tx(&mut self, amount: Option<u128>, addr: Bytes, tx_hash: Vec<u8>) {
        let addr = H160::from_slice(&addr[0..20]);
        if amount.is_none() {
            return;
        }

        let distribution_amount: U256 = amount.unwrap().into();

        let input = asset_functions::mint::encode_input(distribution_amount, addr, tx_hash);

        let tx = Transaction {
            nonce:                    self.get_nonce(&addr),
            max_priority_fee_per_gas: TWO_THOUSAND.into(),
            gas_price:                TWO_THOUSAND.into(),
            gas_limit:                100000u64.into(),
            action:                   TransactionAction::Create,
            data:                     Bytes::from(input),
            value:                    Default::default(),
            access_list:              vec![],
        };

        let mut utx = UnverifiedTransaction {
            unsigned:  tx,
            signature: None,
            chain_id:  **CHAIN_ID.load(),
            hash:      Default::default(),
        };
        let raw = utx.signature_hash();
        let signature =
            Secp256k1Recoverable::sign_message(raw.as_bytes(), &self.priv_key.to_bytes())
                .unwrap()
                .to_bytes();
        utx.signature = Some(signature.into());
        let pub_key = Public::from_slice(&self.priv_key.pub_key().to_uncompressed_bytes()[1..65]);

        let stx = SignedTransaction {
            transaction: utx.calc_hash(),
            sender:      public_to_address(&pub_key),
            public:      Some(pub_key),
        };

        log::info!("axon tx hash: {:?}", stx.transaction.hash);

        if let Err(e) = self.mempool.insert(Context::new(), stx).await {
            log::info!("send tx hash err: {:?}", e);
        };
    }

    fn get_nonce(&self, addr: &H160) -> U256 {
        let backend = AxonExecutorAdapter::from_root(
            **CURRENT_STATE_ROOT.load(),
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            Default::default(),
        )
        .unwrap();

        AxonExecutor::default().get_account(&backend, addr).nonce + 1
    }

    async fn dump_current_number(&self) -> io::Result<()> {
        let path = self.backup_dir.clone();
        let current_number = self.current_number;

        tokio::task::spawn_blocking(move || {
            // create dir
            fs::create_dir_all(path.as_path())?;
            // dump file to a temporary sub-directory
            let tmp_dir = path.join("tmp");
            fs::create_dir_all(&tmp_dir)?;
            let tmp_current_number = tmp_dir.join("current_number.txt");

            let mut file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(false)
                .open(&tmp_current_number)?;

            file.set_len(0)?;
            file.write_all(&current_number.to_le_bytes())?;
            file.sync_all()?;

            move_file(tmp_current_number, path.join("current_number.txt"))
        })
        .await
        .unwrap()
    }
}

#[derive(Clone)]
pub struct CrossAdapterHandle<C> {
    client: Arc<C>,
    config: ConfigCrossChain,
    pk:     Secp256k1RecoverablePrivateKey,
}

#[async_trait]
impl<C> CrossChain for CrossAdapterHandle<C>
where
    C: CkbClient + 'static,
{
    async fn set_evm_log(
        &self,
        ctx: Context,
        block_number: BlockNumber,
        block_hash: H256,
        logs: &[Vec<Log>],
    ) {
        if !self.config.enable {
            return;
        }

        for inner_logs in logs {
            for log in inner_logs {
                if let Ok(burn) = asset_events::burned::parse_log(RawLog::from((
                    log.topics.clone(),
                    log.data.clone(),
                ))) {
                    let Burned {
                        amount,
                        recipient_ckb_address,
                    } = burn;

                    let payload = CrossChainTransferPayload {
                        sender: "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqdrhpvcu82numz73852ed45cdxn4kcn72cr4338a".to_string(),
                        receiver: String::from_utf8(recipient_ckb_address).unwrap(),
                        udt_hash: self.config.axon_udt_hash.0.into(),
                        direction: 1,
                        amount: amount.to_string(),
                        memo: [0;20].into(),
                    };

                    match self
                        .client
                        .build_cross_chain_transfer_transaction(Context::new(), payload)
                        .await
                    {
                        Ok(respond) => {
                            let tx = respond.sign(&self.pk);
                            match self
                                .client
                                .send_transaction(
                                    Context::new(),
                                    &tx,
                                    Some(OutputsValidator::Passthrough),
                                )
                                .await
                            {
                                Ok(tx_hash) => log::info!("set_evm_log send tx hash: {}", tx_hash),
                                Err(e) => {
                                    log::info!("set_evm_log send tx error: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            log::info!("build_cross_chain_transfer_transaction error: {}", e);
                        }
                    }
                }
            }
        }
    }

    async fn set_checkpoint(&self, ctx: Context, block: Block, proof: Proof) {
        if !self.config.enable {
            return;
        }

        let number = block.header.number;
        let mut proposal = Proposal::from(block).encode().unwrap().to_vec();
        let mut proof = proof.encode().unwrap().to_vec();
        proposal.append(&mut proof);

        let payload = SubmitCheckpointPayload {
            node_id:              Identity::new(0, self.config.node_address.0.to_vec()),
            admin_id:             Identity::new(0, self.config.admin_address.0.to_vec()),
            period_number:        number,
            checkpoint:           proposal.into(),
            selection_lock_hash:  self.config.selection_lock_hash.0.into(),
            checkpoint_type_hash: self.config.checkpoint_type_hash.0.into(),
        };

        match self
            .client
            .build_submit_checkpoint_transaction(Context::new(), payload)
            .await
        {
            Ok(respond) => {
                let tx = respond.sign(&self.pk);
                match self
                    .client
                    .send_transaction(Context::new(), &tx, Some(OutputsValidator::Passthrough))
                    .await
                {
                    Ok(tx_hash) => log::info!("set_checkpoint send tx hash: {}", tx_hash),
                    Err(e) => {
                        log::info!("set_checkpoint send tx error: {}", e);
                    }
                }
            }
            Err(e) => {
                log::info!("build_submit_checkpoint_transaction error: {}", e);
            }
        }
    }
}

fn get_amount(data: Bytes) -> u128 {
    let mut le = [0; 16];
    le.clone_from_slice(&data[0..16]);
    u128::from_le_bytes(le)
}

fn move_file<P: AsRef<Path>>(src: P, dst: P) -> Result<(), io::Error> {
    if fs::rename(&src, &dst).is_err() {
        fs::copy(&src, &dst)?;
        fs::remove_file(&src)?;
    }
    Ok(())
}

fn load_current_number<P: AsRef<Path>>(path: P) -> BlockNumber {
    fs::File::open(path.as_ref().join("current_number.txt"))
        .ok()
        .and_then(|mut f| {
            let mut buf = [0; 8];
            f.read_exact(&mut buf).map(|_| u64::from_le_bytes(buf)).ok()
        })
        .unwrap_or_default()
}
