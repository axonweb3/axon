use serde::{Deserialize, Serialize};

use protocol::types::{Direction, H256};

use crate::jsonrpc::web3_types::{Web3Receipt, Web3Transaction};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CrossChainTransaction {
    pub request_tx_hash: H256,
    pub relay_tx_hash:   H256,
    pub direction:       Direction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub axon_tx:         Option<Web3Transaction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt:         Option<Web3Receipt>,
}
