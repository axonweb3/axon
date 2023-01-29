use std::sync::Arc;
use std::time::Duration;

use ckb_types::core::{BlockView, TransactionView};
use ckb_types::{packed, prelude::Pack};

use protocol::tokio::{sync::mpsc::UnboundedSender, time};
use protocol::traits::{CkbClient, Context, CrossAdapter};
use protocol::types::H256;
use protocol::ProtocolResult;

use crate::{error::CrossChainError, CKB_BLOCK_INTERVAL, CKB_TIP, NON_FORK_BLOCK_GAP};

pub struct CrossChainMonitor<C, Adapter> {
    ckb_client:        Arc<C>,
    req_tx:            UnboundedSender<Vec<TransactionView>>,
    handle_ckb_number: u64,
    acs_code_hash:     packed::Byte32,
    request_code_hash: packed::Byte32,

    adapter: Arc<Adapter>,
}

impl<C, Adapter> CrossChainMonitor<C, Adapter>
where
    C: CkbClient + 'static,
    Adapter: CrossAdapter + 'static,
{
    pub async fn new(
        client: Arc<C>,
        sender: UnboundedSender<Vec<TransactionView>>,
        init_ckb_number: u64,
        acs_code_hash: H256,
        request_code_hash: H256,
        cross_adapter: Arc<Adapter>,
    ) -> Self {
        let init_number = cross_adapter
            .get_monitor_ckb_number(Context::new())
            .await
            .unwrap_or(init_ckb_number);

        CKB_TIP.swap(Arc::new(init_number + NON_FORK_BLOCK_GAP));

        CrossChainMonitor {
            ckb_client:        client,
            req_tx:            sender,
            handle_ckb_number: init_number,
            acs_code_hash:     acs_code_hash.0.pack(),
            request_code_hash: request_code_hash.0.pack(),
            adapter:           cross_adapter,
        }
    }

    pub async fn run(mut self) {
        let mut interval = time::interval(Duration::from_secs(CKB_BLOCK_INTERVAL));

        loop {
            if let Err(e) = self.update_tip_number().await {
                log::error!("[cross-chain]: monitor error {:?}", e);
            }

            interval.tick().await;
        }
    }

    async fn update_tip_number(&mut self) -> ProtocolResult<()> {
        let tip_header = self.ckb_client.get_tip_header(Context::new()).await?;

        let tip_number: u64 = tip_header.inner.number.into();
        let current_tip = **CKB_TIP.load();

        if current_tip < tip_number {
            CKB_TIP.swap(Arc::new(tip_number));
            self.fetch_block().await?;
        }

        Ok(())
    }

    async fn fetch_block(&mut self) -> ProtocolResult<()> {
        let non_fork_tip = **CKB_TIP.load() - NON_FORK_BLOCK_GAP;
        if self.handle_ckb_number > non_fork_tip {
            return Ok(());
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
                        list.append(&mut search_tx(
                            block.into(),
                            &self.acs_code_hash,
                            &self.request_code_hash,
                        ));
                        break;
                    }
                    Err(_e) => {
                        log::error!("[cross-chain]: get block from ckb node failed");
                        time::sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        }

        if !list.is_empty() {
            log::info!(
                "[cross-chain]: Cross from CKB block {:?}, request tx count {:?}",
                (self.handle_ckb_number..=non_fork_tip),
                list.len()
            );

            self.req_tx
                .send(list)
                .map_err(|e| CrossChainError::Sender(e.to_string()))?;
        }

        self.handle_ckb_number = non_fork_tip + 1;
        self.adapter
            .update_monitor_ckb_number(Context::new(), self.handle_ckb_number)
            .await?;
        Ok(())
    }
}

pub fn search_tx(
    block: BlockView,
    acs_code_hash: &packed::Byte32,
    request_code_hash: &packed::Byte32,
) -> Vec<TransactionView> {
    block
        .transactions()
        .into_iter()
        .filter(|tx| {
            if tx.is_cellbase() || tx.output_pts().len() < 2 {
                return false;
            }

            if &tx.output(0).unwrap().lock().code_hash() == acs_code_hash {
                if let Some(type_script) = tx.output(1).unwrap().type_().to_opt() {
                    if &type_script.code_hash() == request_code_hash {
                        return true;
                    }
                }
            }

            false
        })
        .collect()
}
