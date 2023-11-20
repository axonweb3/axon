pub const END_GOSSIP_NEW_TXS: &str = "/gossip/mempool/new_txs";
pub const RPC_PULL_TXS: &str = "/rpc_call/mempool/pull_txs";
pub const RPC_RESP_PULL_TXS: &str = "/rpc_resp/mempool/pull_txs";
pub const RPC_RESP_PULL_TXS_SYNC: &str = "/rpc_resp/mempool/pull_txs_sync";

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
