mod rpc_client;

use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use ckb_types::{
    core::{BlockNumber, BlockView, TransactionView},
    packed,
    prelude::*,
};

use common_config_parser::types::{Config, ConfigCrossClient};
use common_crypto::{
    Crypto, PrivateKey, Secp256k1PrivateKey, Secp256k1Recoverable, Signature, ToPublicKey,
    UncompressedPublicKey,
};
use core_executor::{EVMExecutorAdapter, EvmExecutor};
use protocol::traits::{Context, CrossAdapter, Executor, MemPool, Storage};
use protocol::types::{
    public_to_address, Bytes, Public, SignedTransaction, Transaction, TransactionAction,
    UnverifiedTransaction, H160, U256,
};
use protocol::{
    async_trait,
    lazy::{CHAIN_ID, CURRENT_STATE_ROOT},
    tokio::{self, sync::mpsc},
    ProtocolResult,
};

const TWO_THOUSAND: u64 = 2000;

pub struct DefaultCrossAdapter<M, S, DB> {
    priv_key:       Secp256k1PrivateKey,
    ckb_client:     rpc_client::RpcClient,
    config:         ConfigCrossClient,
    tip_number:     BlockNumber,
    current_number: BlockNumber,
    block_recv:     mpsc::Receiver<Vec<io::Result<BlockView>>>,
    block_sender:   mpsc::Sender<Vec<io::Result<BlockView>>>,
    start_fetch:    bool,
    backup_dir:     PathBuf,

    mempool: Arc<M>,
    storage: Arc<S>,
    trie_db: Arc<DB>,
}

#[async_trait]
impl<M, S, DB> CrossAdapter for DefaultCrossAdapter<M, S, DB>
where
    M: MemPool + 'static,
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    async fn watch_ckb_client(&self, ctx: Context) -> ProtocolResult<()> {
        Ok(())
    }

    async fn send_axon_tx(&self, ctx: Context, stx: SignedTransaction) -> ProtocolResult<()> {
        self.mempool.insert(ctx, stx).await
    }

    async fn send_ckb_tx(&self, ctx: Context) -> ProtocolResult<()> {
        Ok(())
    }
}

impl<M, S, DB> DefaultCrossAdapter<M, S, DB>
where
    M: MemPool + 'static,
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    pub fn new(
        config: Config,
        pk: Secp256k1PrivateKey,
        mempool: Arc<M>,
        storage: Arc<S>,
        trie_db: Arc<DB>,
    ) -> Self {
        let backup_dir = config.data_path.join("cross_client");
        let (sender, recv) = mpsc::channel(256);
        Self {
            priv_key: pk,
            ckb_client: rpc_client::RpcClient::new(&config.cross_client.ckb_uri),
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
                        self.handle(blocks).await;
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
        let tip_header = self.ckb_client.get_tip_header().await.unwrap();

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
            for i in
                self.current_number + 1..=std::cmp::min(self.tip_number - self.current_number, 24)
            {
                let task = self.ckb_client.get_block_by_number(i.into());
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

    async fn handle(&mut self, blocks: Vec<io::Result<BlockView>>) {
        for res in blocks {
            match res {
                Ok(block) => self.search_tx(block).await,
                Err(e) => {
                    let number = self.current_number + 1;
                    log::info!("get block {} error: {}", number, e);
                    loop {
                        if let Ok(block) = self.ckb_client.get_block_by_number(number.into()).await
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
                .get_transaction(&hash.unpack())
                .await
                .expect("get previous tx fail")
                .unwrap();

            let tx_view: TransactionView =
                Into::<packed::Transaction>::into(input_tx.transaction.unwrap().inner).into_view();

            let (_, data) = tx_view.output_with_data(index.unpack()).unwrap();

            let input_amount = get_amount(data);
            self.send_axon_tx(
                output_amount.checked_sub(input_amount),
                witnesses.get(2).unwrap().raw_data(),
            )
            .await;
        }
    }

    async fn send_axon_tx(&mut self, amount: Option<u128>, addr: Bytes) {
        let addr = H160::from_slice(&addr[0..20]);
        if amount.is_none() {
            return;
        }

        let tx = Transaction {
            nonce:                    self.get_nonce(&addr),
            max_priority_fee_per_gas: TWO_THOUSAND.into(),
            gas_price:                TWO_THOUSAND.into(),
            gas_limit:                100000u64.into(),
            action:                   TransactionAction::Call(addr),
            data:                     Default::default(),
            value:                    amount.unwrap().into(),
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
            transaction: utx.hash(),
            sender:      public_to_address(&pub_key),
            public:      Some(pub_key),
        };

        let _ = self.mempool.insert(Context::new(), stx).await;
    }

    fn get_nonce(&self, addr: &H160) -> U256 {
        let backend = EVMExecutorAdapter::from_root(
            **CURRENT_STATE_ROOT.load(),
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            Default::default(),
        )
        .unwrap();

        EvmExecutor::default().get_account(&backend, addr).nonce
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
        .map(|mut f| {
            let mut buf = [0; 8];
            f.read_exact(&mut buf).map(|_| u64::from_le_bytes(buf)).ok()
        })
        .flatten()
        .unwrap_or_default()
}
