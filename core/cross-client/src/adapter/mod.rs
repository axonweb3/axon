mod rpc_client;

use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use ckb_types::prelude::{Entity, Unpack};
use common_config_parser::types::{Config, ConfigCrossClient};
use protocol::traits::{Context, CrossAdapter, MemPool};
use protocol::types::{Bytes, SignedTransaction};
use protocol::{
    async_trait,
    tokio::{self, sync::mpsc},
    ProtocolResult,
};

use ckb_types::{
    core::{BlockNumber, BlockView, TransactionView},
    packed::Transaction,
};

pub struct DefaultCrossAdapter<M> {
    mempool:        Arc<M>,
    ckb_client:     rpc_client::RpcClient,
    config:         ConfigCrossClient,
    tip_number:     BlockNumber,
    current_number: BlockNumber,
    block_recv:     mpsc::Receiver<Vec<io::Result<BlockView>>>,
    block_sender:   mpsc::Sender<Vec<io::Result<BlockView>>>,
    start_fetch:    bool,
    backup_dir:     PathBuf,
}

#[async_trait]
impl<M> CrossAdapter for DefaultCrossAdapter<M>
where
    M: MemPool + 'static,
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

impl<M> DefaultCrossAdapter<M>
where
    M: MemPool + 'static,
{
    pub fn new(config: Config, mempool: Arc<M>) -> Self {
        let backup_dir = config.data_path.join("cross_client");
        let (sender, recv) = mpsc::channel(256);
        Self {
            mempool,
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
                Into::<Transaction>::into(input_tx.transaction.unwrap().inner).into_view();

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
        if amount.is_none() {
            return;
        }

        // TODO: insert axon tx to tx pool
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
