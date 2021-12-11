use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use common_merkle::Merkle;
use protocol::codec::ProtocolCodec;
use protocol::traits::Context;
use protocol::types::{
    Block, Bloom, ExecResponse, Hash, Hasher, MerkleRoot, Proof, Validator, U256,
};
use protocol::Display;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct CurrentStatus {
    pub prev_hash:        Hash,
    pub state_root:       MerkleRoot,
    pub receipts_root:    MerkleRoot,
    pub log_bloom:        Bloom,
    pub gas_used:         U256,
    pub gas_limit:        U256,
    pub base_fee_per_gas: Option<U256>,
    pub proof:            Proof,
}

#[derive(Clone, Debug, Display)]
#[display(
    fmt = "exec height {}, cycles used {}, state root {:?}, receipt root {:?}, confirm root {:?}",
    exec_height,
    gas_used,
    state_root,
    receipts_root,
    state_root,
)]
pub struct ExecutedInfo {
    pub ctx:           Context,
    pub exec_height:   u64,
    pub gas_used:      u64,
    pub state_root:    MerkleRoot,
    pub receipts_root: MerkleRoot,
}

impl ExecutedInfo {
    pub fn new(ctx: Context, height: u64, state_root: MerkleRoot, resp: Vec<ExecResponse>) -> Self {
        let gas_sum = resp.iter().map(|r| r.remain_gas).sum();

        let receipt =
            Merkle::from_hashes(resp.iter().map(|r| Hasher::digest(r.ret)).collect::<Vec<_>>())
                .get_root_hash()
                .unwrap_or_default();

        Self {
            ctx,
            exec_height: height,
            gas_used: gas_sum,
            receipts_root: receipt,
            state_root: resp.state_root,
        }
    }
}
