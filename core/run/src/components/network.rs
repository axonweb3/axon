//! Configure the network service.

use std::sync::Arc;

use core_consensus::message::{
    ChokeMessageHandler, ProposalMessageHandler, PullBlockRpcHandler, PullProofRpcHandler,
    PullTxsRpcHandler, QCMessageHandler, RemoteHeightMessageHandler, VoteMessageHandler,
    BROADCAST_HEIGHT, END_GOSSIP_AGGREGATED_VOTE, END_GOSSIP_SIGNED_CHOKE,
    END_GOSSIP_SIGNED_PROPOSAL, END_GOSSIP_SIGNED_VOTE, RPC_RESP_SYNC_PULL_BLOCK,
    RPC_RESP_SYNC_PULL_PROOF, RPC_RESP_SYNC_PULL_TXS, RPC_SYNC_PULL_BLOCK, RPC_SYNC_PULL_PROOF,
    RPC_SYNC_PULL_TXS,
};
use core_consensus::OverlordSynchronization;
use core_db::RocksAdapter;
use core_mempool::{
    NewTxsHandler, PullTxsHandler, END_GOSSIP_NEW_TXS, RPC_PULL_TXS, RPC_RESP_PULL_TXS,
    RPC_RESP_PULL_TXS_SYNC,
};
use core_network::{KeyProvider, NetworkService, PeerId, PeerIdExt};
use core_storage::ImplStorage;
use protocol::{
    traits::{Consensus, Context, MemPool, Network, SynchronizationAdapter},
    types::ValidatorExtend,
    ProtocolResult,
};

pub(crate) trait NetworkServiceExt {
    fn tag_consensus(&self, validators: &[ValidatorExtend]) -> ProtocolResult<()>;

    fn register_mempool_endpoint(
        &mut self,
        mempool: &Arc<impl MemPool + 'static>,
    ) -> ProtocolResult<()>;

    fn register_consensus_endpoint(
        &mut self,
        overlord_consensus: &Arc<impl Consensus + 'static>,
    ) -> ProtocolResult<()>;

    fn register_synchronization_endpoint(
        &mut self,
        synchronization: &Arc<OverlordSynchronization<impl SynchronizationAdapter + 'static>>,
    ) -> ProtocolResult<()>;

    fn register_storage_endpoint(
        &mut self,
        storage: &Arc<ImplStorage<RocksAdapter>>,
    ) -> ProtocolResult<()>;

    fn register_rpc(&mut self) -> ProtocolResult<()>;
}

impl<K> NetworkServiceExt for NetworkService<K>
where
    K: KeyProvider,
{
    fn tag_consensus(&self, validators: &[ValidatorExtend]) -> ProtocolResult<()> {
        let peer_ids = validators
            .iter()
            .map(|v| PeerId::from_pubkey_bytes(v.pub_key.as_bytes()).map(PeerIdExt::into_bytes_ext))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        self.handle().tag_consensus(Context::new(), peer_ids)
    }

    fn register_mempool_endpoint(
        &mut self,
        mempool: &Arc<impl MemPool + 'static>,
    ) -> ProtocolResult<()> {
        // register broadcast new transaction
        self.register_endpoint_handler(
            END_GOSSIP_NEW_TXS,
            NewTxsHandler::new(Arc::clone(mempool)),
        )?;
        // register pull txs from other node
        self.register_endpoint_handler(
            RPC_PULL_TXS,
            PullTxsHandler::new(Arc::new(self.handle()), Arc::clone(mempool)),
        )?;
        Ok(())
    }

    fn register_consensus_endpoint(
        &mut self,
        overlord_consensus: &Arc<impl Consensus + 'static>,
    ) -> ProtocolResult<()> {
        // register consensus
        self.register_endpoint_handler(
            END_GOSSIP_SIGNED_PROPOSAL,
            ProposalMessageHandler::new(Arc::clone(overlord_consensus)),
        )?;
        self.register_endpoint_handler(
            END_GOSSIP_AGGREGATED_VOTE,
            QCMessageHandler::new(Arc::clone(overlord_consensus)),
        )?;
        self.register_endpoint_handler(
            END_GOSSIP_SIGNED_VOTE,
            VoteMessageHandler::new(Arc::clone(overlord_consensus)),
        )?;
        self.register_endpoint_handler(
            END_GOSSIP_SIGNED_CHOKE,
            ChokeMessageHandler::new(Arc::clone(overlord_consensus)),
        )?;
        Ok(())
    }

    fn register_synchronization_endpoint(
        &mut self,
        synchronization: &Arc<OverlordSynchronization<impl SynchronizationAdapter + 'static>>,
    ) -> ProtocolResult<()> {
        self.register_endpoint_handler(
            BROADCAST_HEIGHT,
            RemoteHeightMessageHandler::new(Arc::clone(synchronization)),
        )?;
        Ok(())
    }

    fn register_storage_endpoint(
        &mut self,
        storage: &Arc<ImplStorage<RocksAdapter>>,
    ) -> ProtocolResult<()> {
        let handle = Arc::new(self.handle());
        // register storage
        self.register_endpoint_handler(
            RPC_SYNC_PULL_BLOCK,
            PullBlockRpcHandler::new(Arc::clone(&handle), Arc::clone(storage)),
        )?;
        self.register_endpoint_handler(
            RPC_SYNC_PULL_PROOF,
            PullProofRpcHandler::new(Arc::clone(&handle), Arc::clone(storage)),
        )?;
        self.register_endpoint_handler(
            RPC_SYNC_PULL_TXS,
            PullTxsRpcHandler::new(Arc::clone(&handle), Arc::clone(storage)),
        )?;
        Ok(())
    }

    fn register_rpc(&mut self) -> ProtocolResult<()> {
        self.register_rpc_response(RPC_RESP_PULL_TXS)?;
        self.register_rpc_response(RPC_RESP_PULL_TXS_SYNC)?;
        self.register_rpc_response(RPC_RESP_SYNC_PULL_BLOCK)?;
        self.register_rpc_response(RPC_RESP_SYNC_PULL_PROOF)?;
        self.register_rpc_response(RPC_RESP_SYNC_PULL_TXS)?;
        Ok(())
    }
}
