use std::sync::Arc;

use futures::TryFutureExt;
use log::warn;
use overlord::types::{AggregatedVote, SignedChoke, SignedProposal, SignedVote};
use overlord::Codec;
use rlp::Encodable;

use common_apm::muta_apm;
use protocol::traits::{
    Consensus, Context, MessageHandler, Priority, Rpc, Storage, Synchronization, TrustFeedback,
};
use protocol::types::BatchSignedTxs;
use protocol::{async_trait, types::BlockNumber, ProtocolError};

use core_storage::StorageError;

pub use crate::types::PullTxsRequest;

pub const END_GOSSIP_SIGNED_PROPOSAL: &str = "/gossip/consensus/signed_proposal";
pub const END_GOSSIP_SIGNED_VOTE: &str = "/gossip/consensus/signed_vote";
pub const END_GOSSIP_AGGREGATED_VOTE: &str = "/gossip/consensus/qc";
pub const END_GOSSIP_SIGNED_CHOKE: &str = "/gossip/consensus/signed_choke";
pub const RPC_SYNC_PULL_BLOCK: &str = "/rpc_call/consensus/sync_pull_block";
pub const RPC_RESP_SYNC_PULL_BLOCK: &str = "/rpc_resp/consensus/sync_pull_block";
pub const RPC_SYNC_PULL_TXS: &str = "/rpc_call/consensus/sync_pull_txs";
pub const RPC_RESP_SYNC_PULL_TXS: &str = "/rpc_resp/consensus/sync_pull_txs";
pub const BROADCAST_HEIGHT: &str = "/gossip/consensus/broadcast_height";
pub const RPC_SYNC_PULL_PROOF: &str = "/rpc_call/consensus/sync_pull_proof";
pub const RPC_RESP_SYNC_PULL_PROOF: &str = "/rpc_resp/consensus/sync_pull_proof";

macro_rules! overlord_message {
    ($msg_name: ident, $overlord_type_name: ident) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $msg_name(pub protocol::types::Bytes);

        impl From<$overlord_type_name> for $msg_name {
            fn from(overlord_ty: $overlord_type_name) -> Self {
                Self(overlord_ty.rlp_bytes().freeze())
            }
        }

        impl protocol::codec::ProtocolCodec for $msg_name {
            fn encode(&self) -> protocol::ProtocolResult<protocol::types::Bytes> {
                Ok(self.0.clone())
            }

            fn decode<B: AsRef<[u8]>>(bytes: B) -> protocol::ProtocolResult<Self> {
                Ok(Self(protocol::types::Bytes::from(bytes.as_ref().to_vec())))
            }
        }

        impl $msg_name {
            pub fn to_vec(&self) -> Vec<u8> {
                self.0.to_vec()
            }
        }
    };

    ($msg_name: ident, $overlord_type_name: ident, $other_type: ident) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $msg_name(pub protocol::types::Bytes);

        impl<$other_type: Codec> From<$overlord_type_name<$other_type>> for $msg_name {
            fn from(overlord_ty: $overlord_type_name<$other_type>) -> Self {
                Self(overlord_ty.rlp_bytes().freeze())
            }
        }

        impl protocol::codec::ProtocolCodec for $msg_name {
            fn encode(&self) -> protocol::ProtocolResult<protocol::types::Bytes> {
                Ok(self.0.clone())
            }

            fn decode<B: AsRef<[u8]>>(bytes: B) -> protocol::ProtocolResult<Self> {
                Ok(Self(protocol::types::Bytes::from(bytes.as_ref().to_vec())))
            }
        }

        impl $msg_name {
            pub fn to_vec(&self) -> Vec<u8> {
                self.0.to_vec()
            }
        }
    };
}

overlord_message!(Proposal, SignedProposal, C);
overlord_message!(Vote, SignedVote);
overlord_message!(QC, AggregatedVote);
overlord_message!(Choke, SignedChoke);

pub struct ProposalMessageHandler<C> {
    consensus: Arc<C>,
}

impl<C: Consensus + 'static> ProposalMessageHandler<C> {
    pub fn new(consensus: Arc<C>) -> Self {
        Self { consensus }
    }
}

#[async_trait]
impl<C: Consensus + 'static> MessageHandler for ProposalMessageHandler<C> {
    type Message = Proposal;

    // #[muta_apm::derive::tracing_span(name = "handle_proposal", kind =
    // "consensus.message")]
    async fn process(&self, ctx: Context, msg: Self::Message) -> TrustFeedback {
        if let Err(e) = self.consensus.set_proposal(ctx, msg.to_vec()).await {
            warn!("set proposal {}", e);
            return TrustFeedback::Worse(e.to_string());
        }

        TrustFeedback::Good
    }
}

pub struct VoteMessageHandler<C> {
    consensus: Arc<C>,
}

impl<C: Consensus + 'static> VoteMessageHandler<C> {
    pub fn new(consensus: Arc<C>) -> Self {
        Self { consensus }
    }
}

#[async_trait]
impl<C: Consensus + 'static> MessageHandler for VoteMessageHandler<C> {
    type Message = Vote;

    #[muta_apm::derive::tracing_span(name = "handle_vote", kind = "consensus.message")]
    async fn process(&self, ctx: Context, msg: Self::Message) -> TrustFeedback {
        if let Err(e) = self.consensus.set_vote(ctx, msg.to_vec()).await {
            warn!("set vote {}", e);
            return TrustFeedback::Worse(e.to_string());
        }

        TrustFeedback::Good
    }
}

pub struct QCMessageHandler<C> {
    consensus: Arc<C>,
}

impl<C: Consensus + 'static> QCMessageHandler<C> {
    pub fn new(consensus: Arc<C>) -> Self {
        Self { consensus }
    }
}

#[async_trait]
impl<C: Consensus + 'static> MessageHandler for QCMessageHandler<C> {
    type Message = QC;

    #[muta_apm::derive::tracing_span(name = "handle_qc", kind = "consensus.message")]
    async fn process(&self, ctx: Context, msg: Self::Message) -> TrustFeedback {
        if let Err(e) = self.consensus.set_qc(ctx, msg.to_vec()).await {
            warn!("set qc {}", e);
            return TrustFeedback::Worse(e.to_string());
        }

        TrustFeedback::Good
    }
}

pub struct ChokeMessageHandler<C> {
    consensus: Arc<C>,
}

impl<C: Consensus + 'static> ChokeMessageHandler<C> {
    pub fn new(consensus: Arc<C>) -> Self {
        Self { consensus }
    }
}

#[async_trait]
impl<C: Consensus + 'static> MessageHandler for ChokeMessageHandler<C> {
    type Message = Choke;

    #[muta_apm::derive::tracing_span(name = "handle_choke", kind = "consensus.message")]
    async fn process(&self, ctx: Context, msg: Self::Message) -> TrustFeedback {
        if let Err(e) = self.consensus.set_choke(ctx, msg.to_vec()).await {
            warn!("set choke {}", e);
            return TrustFeedback::Worse(e.to_string());
        }

        TrustFeedback::Good
    }
}

pub struct RemoteHeightMessageHandler<Sy> {
    synchronization: Arc<Sy>,
}

impl<Sy: Synchronization + 'static> RemoteHeightMessageHandler<Sy> {
    pub fn new(synchronization: Arc<Sy>) -> Self {
        Self { synchronization }
    }
}

#[async_trait]
impl<Sy: Synchronization + 'static> MessageHandler for RemoteHeightMessageHandler<Sy> {
    type Message = BlockNumber;

    #[muta_apm::derive::tracing_span(name = "handle_remote_height", kind = "consensus.message")]
    async fn process(&self, ctx: Context, remote_height: Self::Message) -> TrustFeedback {
        if let Err(e) = self
            .synchronization
            .receive_remote_block(ctx, remote_height)
            .await
        {
            warn!("sync: receive remote block {}", e);
            if e.to_string().contains("timeout") {
                return TrustFeedback::Bad("sync block timeout".to_owned());
            } else {
                // Just in case, don't use worse here
                return TrustFeedback::Bad(e.to_string());
            }
        }

        TrustFeedback::Good
    }
}

#[derive(Debug)]
pub struct PullBlockRpcHandler<R, S> {
    rpc:     Arc<R>,
    storage: Arc<S>,
}

impl<R, S> PullBlockRpcHandler<R, S>
where
    R: Rpc + 'static,
    S: Storage + 'static,
{
    pub fn new(rpc: Arc<R>, storage: Arc<S>) -> Self {
        PullBlockRpcHandler { rpc, storage }
    }
}

#[async_trait]
impl<R: Rpc + 'static, S: Storage + 'static> MessageHandler for PullBlockRpcHandler<R, S> {
    type Message = BlockNumber;

    #[muta_apm::derive::tracing_span(name = "pull_block_rpc", kind = "consensus.message")]
    async fn process(&self, ctx: Context, msg: BlockNumber) -> TrustFeedback {
        let ret = match self.storage.get_block(ctx.clone(), msg).await {
            Ok(Some(block)) => Ok(block),
            Ok(None) => Err(StorageError::GetNone.into()),
            Err(e) => Err(e),
        };
        self.rpc
            .response(ctx, RPC_RESP_SYNC_PULL_BLOCK, ret, Priority::High)
            .unwrap_or_else(move |e: ProtocolError| warn!("[core_consensus] push block {}", e))
            .await;

        TrustFeedback::Neutral
    }
}

#[derive(Debug)]
pub struct PullProofRpcHandler<R, S> {
    rpc:     Arc<R>,
    storage: Arc<S>,
}

impl<R, S> PullProofRpcHandler<R, S>
where
    R: Rpc + 'static,
    S: Storage + 'static,
{
    pub fn new(rpc: Arc<R>, storage: Arc<S>) -> Self {
        PullProofRpcHandler { rpc, storage }
    }
}

#[async_trait]
impl<R: Rpc + 'static, S: Storage + 'static> MessageHandler for PullProofRpcHandler<R, S> {
    type Message = BlockNumber;

    #[muta_apm::derive::tracing_span(name = "pull_proof_rpc", kind = "consensus.message")]
    async fn process(&self, ctx: Context, msg: BlockNumber) -> TrustFeedback {
        let latest_proof = self.storage.get_latest_proof(ctx.clone()).await;

        let ret = match latest_proof {
            Ok(latest_proof) => match msg {
                number if number < latest_proof.number => {
                    match self.storage.get_block_header(ctx.clone(), number + 1).await {
                        Ok(Some(next_header)) => Ok(next_header.proof),
                        Ok(None) => Err(StorageError::GetNone.into()),
                        Err(_) => Err(StorageError::GetNone.into()),
                    }
                }
                number if number == latest_proof.number => Ok(latest_proof),
                _ => Err(StorageError::GetNone.into()),
            },
            Err(_) => Err(StorageError::GetNone.into()),
        };

        self.rpc
            .response(ctx, RPC_RESP_SYNC_PULL_PROOF, ret, Priority::High)
            .unwrap_or_else(move |e: ProtocolError| warn!("[core_consensus] push proof {}", e))
            .await;

        TrustFeedback::Neutral
    }
}

#[derive(Debug)]
pub struct PullTxsRpcHandler<R, S> {
    rpc:     Arc<R>,
    storage: Arc<S>,
}

impl<R, S> PullTxsRpcHandler<R, S>
where
    R: Rpc + 'static,
    S: Storage + 'static,
{
    pub fn new(rpc: Arc<R>, storage: Arc<S>) -> Self {
        PullTxsRpcHandler { rpc, storage }
    }
}

#[async_trait]
impl<R: Rpc + 'static, S: Storage + 'static> MessageHandler for PullTxsRpcHandler<R, S> {
    type Message = PullTxsRequest;

    // #[muta_apm::derive::tracing_span(name = "pull_txs_rpc", kind =
    // "consensus.message")]
    async fn process(&self, ctx: Context, msg: PullTxsRequest) -> TrustFeedback {
        let PullTxsRequest { height, inner } = msg;

        let ret = self
            .storage
            .get_transactions(ctx.clone(), height, &inner)
            .await
            .map(|txs| BatchSignedTxs(txs.into_iter().flatten().collect::<Vec<_>>()));

        self.rpc
            .response(ctx, RPC_RESP_SYNC_PULL_TXS, ret, Priority::High)
            .unwrap_or_else(move |e: ProtocolError| warn!("[core_consensus] push txs {}", e))
            .await;

        TrustFeedback::Neutral
    }
}
