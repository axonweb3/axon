use std::sync::Arc;

use crate::{
    status::{CurrentStatus, StatusAgent},
    synchronization::{OverlordSynchronization, RichBlock},
    tests::MockSyncAdapter,
    util::time_now,
};
use creep::Context;
use protocol::{
    tokio::{self, sync::Mutex as AsyncMutex},
    traits::Synchronization,
    types::{Block, Header},
};

pub fn get_mock_synchronization() -> OverlordSynchronization<MockSyncAdapter> {
    let sync_txs_chunk_size = 50;
    let consensus_adapter = Arc::new(MockSyncAdapter::default());
    let status_agent = StatusAgent::new(CurrentStatus::default());
    let lock = Arc::new(AsyncMutex::new(()));

    OverlordSynchronization::<_>::new(sync_txs_chunk_size, consensus_adapter, status_agent, lock)
}

pub fn get_mock_rick_block() -> RichBlock {
    RichBlock {
        txs:   vec![],
        block: Block {
            tx_hashes: vec![],
            header:    Header {
                prev_hash:                  Default::default(),
                proposer:                   Default::default(),
                state_root:                 Default::default(),
                transactions_root:          Default::default(),
                signed_txs_hash:            Default::default(),
                receipts_root:              Default::default(),
                log_bloom:                  Default::default(),
                difficulty:                 Default::default(),
                timestamp:                  time_now(),
                number:                     1,
                gas_used:                   Default::default(),
                gas_limit:                  Default::default(),
                extra_data:                 Default::default(),
                mixed_hash:                 Default::default(),
                nonce:                      Default::default(),
                base_fee_per_gas:           Default::default(),
                proof:                      Default::default(),
                last_checkpoint_block_hash: Default::default(),
                call_system_script_count:   0,
                chain_id:                   0,
            },
        },
    }
}

#[tokio::test]
async fn test_new() {
    let synchronization = get_mock_synchronization();
    tokio::spawn(async move {
        if let Err(e) = synchronization.polling_broadcast().await {
            println!("synchronization: {:?}", e);
        }
    });
    println!("test_new end");
}

#[tokio::test]
async fn test_receive_remote_block() {
    let sync_txs_chunk_size = 50;
    let status_agent = StatusAgent::new(CurrentStatus::default());
    let lock = Arc::new(AsyncMutex::new(()));

    // let network_service = mock_network_service();
    let consensus_adapter = MockSyncAdapter::default();
    let consensus_adapter = Arc::new(consensus_adapter);

    let synchronization = Arc::new(OverlordSynchronization::<_>::new(
        sync_txs_chunk_size,
        consensus_adapter,
        status_agent.clone(),
        lock,
    ));

    let result = synchronization
        .receive_remote_block(Context::new(), 1)
        .await;
    assert!(result.is_err());
    println!("{:?}", result.err());
}
