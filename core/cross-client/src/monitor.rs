use std::sync::Arc;
use std::time::Duration;

use ckb_types::core::{BlockView, TransactionView};
use ckb_types::{packed, prelude::Pack};

use protocol::tokio::{sync::mpsc::UnboundedSender, time};
use protocol::traits::{CkbClient, Context};
use protocol::types::H256;

use crate::{CKB_BLOCK_INTERVAL, CKB_TIP, NON_FORK_BLOCK_GAP};

pub struct CrossChainMonitor<C> {
    ckb_client:        Arc<C>,
    req_tx:            UnboundedSender<Vec<TransactionView>>,
    handle_ckb_number: u64,
    acs_code_hash:     packed::Byte32,
    request_code_hash: packed::Byte32,
}

impl<C: CkbClient + 'static> CrossChainMonitor<C> {
    pub fn new(
        client: Arc<C>,
        sender: UnboundedSender<Vec<TransactionView>>,
        init_number: u64,
        acs_code_hash: H256,
        request_code_hash: H256,
    ) -> Self {
        CrossChainMonitor {
            ckb_client:        client,
            req_tx:            sender,
            handle_ckb_number: init_number,
            acs_code_hash:     acs_code_hash.0.pack(),
            request_code_hash: request_code_hash.0.pack(),
        }
    }

    pub async fn run(mut self) {
        let mut interval = time::interval(Duration::from_secs(CKB_BLOCK_INTERVAL));

        loop {
            self.update_tip_number().await;
            interval.tick().await;
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
        let non_fork_tip = **CKB_TIP.load() - NON_FORK_BLOCK_GAP;
        if self.handle_ckb_number > non_fork_tip {
            return;
        }

        let mut list = Vec::new();
        for i in self.handle_ckb_number..=non_fork_tip {
            loop {
                match self
                    .ckb_client
                    .get_block_by_number(Context::new(), i.into())
                    .await
                {
                    Ok(block) => {
                        list.append(&mut self.search_tx(block.into()));
                        break;
                    }
                    Err(e) => {
                        log::error!("[cross-chain]: get block from ckb node failed");
                        time::sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        }

        self.req_tx.send(list).unwrap();
        self.handle_ckb_number = non_fork_tip + 1;
    }

    fn search_tx(&self, block: BlockView) -> Vec<TransactionView> {
        block
            .transactions()
            .into_iter()
            .filter(|tx| {
                if tx.is_cellbase() || tx.output_pts().len() < 2 {
                    return false;
                }

                if tx.output(0).unwrap().lock().code_hash() == self.acs_code_hash {
                    if let Some(type_script) = tx.output(1).unwrap().type_().to_opt() {
                        if type_script.code_hash() == self.request_code_hash {
                            return true;
                        }
                    }
                }

                false
            })
            .collect()
    }
}
