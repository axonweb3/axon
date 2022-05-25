use std::sync::Arc;
use std::time::Duration;

use ckb_types::core::{BlockView, TransactionView};

use protocol::tokio::{self, sync::mpsc::UnboundedSender, time};
use protocol::traits::{CkbClient, Context};

use crate::{CKB_BLOCK_INTERVAL, CKB_TIP, NON_FORK_BLOCK_GAP};

pub struct CrossChainMonitor<C> {
    ckb_client:        Arc<C>,
    req_tx:            UnboundedSender<Vec<TransactionView>>,
    handle_ckb_number: u64,
}

impl<C: CkbClient + 'static> CrossChainMonitor<C> {
    pub fn new(ckb_client: Arc<C>, req_tx: UnboundedSender<Vec<TransactionView>>) -> Self {
        CrossChainMonitor {
            ckb_client,
            req_tx,
            handle_ckb_number: 0,
        }
    }

    pub async fn run(mut self) {
        let mut interval = time::interval(Duration::from_secs(CKB_BLOCK_INTERVAL));

        loop {
            interval.tick().await;
            self.update_tip_number().await
        }
    }

    async fn update_tip_number(&mut self) {
        let tip_header = self
            .ckb_client
            .get_tip_header(Context::new())
            .await
            .unwrap();

        let tip_number: u64 = tip_header.inner.number.into();
        let current_tip = **CKB_TIP.load();
        if current_tip < tip_number {
            CKB_TIP.swap(Arc::new(tip_number));
        }

        self.fetch_block().await
    }

    async fn fetch_block(&mut self) {
        let sender = self.req_tx.clone();
        let start_number = self.handle_ckb_number + 1;
        let ckb_client = Arc::clone(&self.ckb_client);

        tokio::spawn(async move {
            let mut list = Vec::new();
            for i in start_number..=(**CKB_TIP.load() - NON_FORK_BLOCK_GAP) {
                loop {
                    match ckb_client
                        .get_block_by_number(Context::new(), i.into())
                        .await
                    {
                        Ok(block) => {
                            list.append(&mut search_tx(block.into()));
                            break;
                        }
                        Err(e) => {
                            log::error!("[cross-chain]: get block from ckb node failed");
                            time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
            }
            sender.send(list).unwrap();
        });
    }
}

fn search_tx(block: BlockView) -> Vec<TransactionView> {
    block.transactions().into_iter().filter(|tx| true).collect()
}
