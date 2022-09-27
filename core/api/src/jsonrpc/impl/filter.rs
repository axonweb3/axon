use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use jsonrpsee::core::Error;
use serde::{Deserialize, Serialize};

use protocol::async_trait;
use protocol::tokio::sync::mpsc::{channel, Receiver, Sender};
use protocol::tokio::{self, select, sync::oneshot, time::interval};
use protocol::traits::{APIAdapter, Context};
use protocol::types::{BlockNumber, Hash, Receipt, H160, H256, U256};

use crate::jsonrpc::web3_types::{BlockId, FilterChanges, RawLoggerFilter, Web3Log};
use crate::jsonrpc::{r#impl::from_receipt_to_web3_log, RpcResult, Web3FilterServer};

pub fn filter_module<Adapter>(adapter: Arc<Adapter>) -> AxonWeb3RpcFilter
where
    Adapter: APIAdapter + 'static,
{
    let (tx, rx) = channel(128);

    tokio::spawn(FilterHub::new(adapter, rx).run());

    AxonWeb3RpcFilter { sender: tx }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LoggerFilter {
    pub from_block: Option<BlockId>,
    pub to_block:   Option<BlockId>,
    pub address:    Option<Vec<H160>>,
    pub topics:     Vec<Option<Vec<Option<Hash>>>>,
}

impl From<RawLoggerFilter> for LoggerFilter {
    fn from(src: RawLoggerFilter) -> Self {
        LoggerFilter {
            from_block: src.from_block,
            to_block:   src.to_block,
            address:    src.address.into(),
            topics:     src
                .topics
                .unwrap_or_default()
                .into_iter()
                .take(4)
                .map(Into::<Option<Vec<Option<H256>>>>::into)
                .collect(),
        }
    }
}

pub struct AxonWeb3RpcFilter {
    sender: Sender<Command>,
}

#[async_trait]
impl Web3FilterServer for AxonWeb3RpcFilter {
    async fn new_filter(&self, filter: RawLoggerFilter) -> RpcResult<U256> {
        if let Some(BlockId::Pending) = filter.from_block {
            return Err(Error::Custom(
                "Invalid from_block and to_block union".to_string(),
            ));
        }
        match filter.to_block {
            Some(BlockId::Earliest) | Some(BlockId::Num(0)) => {
                return Err(Error::Custom("Invalid to_block".to_string()))
            }
            _ => (),
        }
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(Command::NewLogs((filter.into(), tx)))
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(rx.await.unwrap())
    }

    async fn block_filter(&self) -> RpcResult<U256> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(Command::NewBlocks(tx))
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(rx.await.unwrap())
    }

    async fn get_filter_logs(&self, id: U256) -> RpcResult<FilterChanges> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(Command::FilterRequest((id, tx)))
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        rx.await.unwrap()
    }

    async fn get_filter_changes(&self, id: U256) -> RpcResult<FilterChanges> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(Command::FilterRequest((id, tx)))
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        rx.await.unwrap()
    }

    async fn uninstall_filter(&self, id: U256) -> RpcResult<bool> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(Command::Uninstall((id, tx)))
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(rx.await.unwrap())
    }
}

pub enum Command {
    NewLogs((LoggerFilter, oneshot::Sender<U256>)),
    NewBlocks(oneshot::Sender<U256>),
    FilterRequest((U256, oneshot::Sender<RpcResult<FilterChanges>>)),
    Uninstall((U256, oneshot::Sender<bool>)),
}

pub struct FilterHub<Adapter> {
    logs_hub:   HashMap<U256, (LoggerFilter, Instant)>,
    blocks_hub: HashMap<U256, (BlockNumber, Instant)>,
    id:         u64,
    recv:       Receiver<Command>,
    adapter:    Arc<Adapter>,
}

impl<Adapter> FilterHub<Adapter>
where
    Adapter: APIAdapter + 'static,
{
    pub fn new(adapter: Arc<Adapter>, recv: Receiver<Command>) -> Self {
        Self {
            logs_hub: HashMap::new(),
            blocks_hub: HashMap::new(),
            id: 0,
            recv,
            adapter,
        }
    }

    async fn run(mut self) {
        let mut time_internal = interval(Duration::from_secs(20));
        loop {
            select! {
                event = self.recv.recv() => {
                    match event {
                        Some(cmd) => {
                            self.handle(cmd).await
                        },
                        None => {
                            break
                        }
                    }
                }
                _ = time_internal.tick() => {
                    self.check_hubs();
                }
                else => {
                    break
                }
            }
        }
    }

    fn check_hubs(&mut self) {
        let now = Instant::now();
        self.blocks_hub
            .retain(|_, (_, time)| now.saturating_duration_since(*time) < Duration::from_secs(40));
        self.logs_hub
            .retain(|_, (_, time)| now.saturating_duration_since(*time) < Duration::from_secs(40))
    }

    async fn handle(&mut self, cmd: Command) {
        match cmd {
            Command::NewLogs((mut filter, sender)) => {
                self.id += 1;
                let header = self
                    .adapter
                    .get_block_header_by_number(Context::new(), None)
                    .await
                    .unwrap()
                    .unwrap();
                let from = filter.from_block.as_ref().unwrap_or(&BlockId::Latest);

                match from {
                    BlockId::Num(n) => {
                        if n < &header.number {
                            filter.from_block = Some(BlockId::Num(header.number + 1));
                        }
                    }
                    _ => filter.from_block = Some(BlockId::Num(header.number + 1)),
                }

                self.logs_hub
                    .insert(self.id.into(), (filter, Instant::now()));
                sender.send(self.id.into()).unwrap()
            }
            Command::NewBlocks(sender) => {
                self.id += 1;
                let header = self
                    .adapter
                    .get_block_header_by_number(Context::new(), None)
                    .await
                    .unwrap()
                    .unwrap();
                self.blocks_hub
                    .insert(self.id.into(), (header.number, Instant::now()));
                sender.send(self.id.into()).unwrap()
            }
            Command::FilterRequest((id, sender)) => self.impl_filter(id, sender).await,
            Command::Uninstall((id, sender)) => {
                let removed =
                    self.blocks_hub.remove(&id).is_some() || self.logs_hub.remove(&id).is_some();
                sender.send(removed).unwrap()
            }
        }
    }

    async fn impl_filter(&mut self, id: U256, sender: oneshot::Sender<RpcResult<FilterChanges>>) {
        if self.blocks_hub.contains_key(&id) {
            let res = Ok(FilterChanges::Blocks(self.filter_block(&id).await));
            sender.send(res).unwrap()
        } else if self.logs_hub.contains_key(&id) {
            let res = self.filter_logs(&id).await.map(FilterChanges::Logs);
            if res.is_err() {
                self.logs_hub.remove(&id);
            }
            sender.send(res).unwrap()
        } else {
            sender
                .send(Err(Error::Custom(format!(
                    "Can't find this filter id: {}",
                    id
                ))))
                .unwrap()
        }
    }

    async fn filter_block(&mut self, id: &U256) -> Vec<H256> {
        let (start, time) = self.blocks_hub.get_mut(id).unwrap();
        let latest = self
            .adapter
            .get_block_by_number(Context::new(), None)
            .await
            .unwrap()
            .unwrap();
        if *start >= latest.header.number {
            return Vec::new();
        }

        let mut block_hashes = Vec::with_capacity((latest.header.number - *start) as usize);

        for number in *start + 1..latest.header.number {
            let block = self
                .adapter
                .get_block_by_number(Context::new(), Some(number))
                .await
                .unwrap()
                .unwrap();

            block_hashes.push(block.hash());
        }

        block_hashes.push(latest.hash());

        *start = latest.header.number;
        *time = Instant::now();

        block_hashes
    }

    async fn filter_logs(&mut self, id: &U256) -> RpcResult<Vec<Web3Log>> {
        let (filter, time) = self.logs_hub.get_mut(id).unwrap();

        let topics = filter.topics.as_slice();

        let mut all_logs = Vec::new();

        let latest_block = self
            .adapter
            .get_block_by_number(Context::new(), None)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
            .unwrap();

        let latest_number = latest_block.header.number;
        let (start, end) = {
            let convert = |id: &BlockId| -> BlockNumber {
                match id {
                    BlockId::Num(n) => *n,
                    BlockId::Earliest => 0,
                    _ => latest_number,
                }
            };

            (
                filter
                    .from_block
                    .as_ref()
                    .map(convert)
                    .unwrap_or(latest_number),
                std::cmp::min(
                    filter
                        .to_block
                        .as_ref()
                        .map(convert)
                        .unwrap_or(latest_number),
                    latest_number,
                ),
            )
        };

        if start > latest_number {
            return Ok(Vec::new());
        }

        let extend_logs = |logs: &mut Vec<Web3Log>, receipts: Vec<Option<Receipt>>| {
            for (index, receipt) in receipts.into_iter().flatten().enumerate() {
                from_receipt_to_web3_log(
                    index,
                    topics,
                    filter.address.as_ref().unwrap_or(&Vec::new()),
                    &receipt,
                    logs,
                )
            }
        };

        let mut visiter_last_block = false;
        for n in start..=end {
            if n == latest_number {
                visiter_last_block = true;
            } else {
                let block = self
                    .adapter
                    .get_block_by_number(Context::new(), Some(n))
                    .await
                    .map_err(|e| Error::Custom(e.to_string()))?
                    .unwrap();
                let receipts = self
                    .adapter
                    .get_receipts_by_hashes(Context::new(), block.header.number, &block.tx_hashes)
                    .await
                    .map_err(|e| Error::Custom(e.to_string()))?;

                extend_logs(&mut all_logs, receipts);
            }
        }

        if visiter_last_block {
            let receipts = self
                .adapter
                .get_receipts_by_hashes(
                    Context::new(),
                    latest_block.header.number,
                    &latest_block.tx_hashes,
                )
                .await
                .map_err(|e| Error::Custom(e.to_string()))?;

            extend_logs(&mut all_logs, receipts);
        }

        if let Some(BlockId::Num(ref mut n)) = filter.from_block {
            *n = end + 1
        }
        *time = Instant::now();
        Ok(all_logs)
    }
}
