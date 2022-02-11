use std::io::Error;

use metrics_derive::metrics_rpc;

use protocol::types::{Hash, SignedTransaction};

pub struct RpcExample;

impl RpcExample {
    #[metrics_rpc("eth_sendRawTransaction")]
    fn send_raw_transaction(&self, tx: SignedTransaction) -> Result<Hash, Error> {
        Ok(tx.transaction.hash)
    }
}

fn main() {}
