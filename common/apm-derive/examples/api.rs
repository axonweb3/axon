use async_trait::async_trait;
use common_apm_derive::metrics_rpc;
use jsonrpsee::core::Error;

use protocol::types::{Hash, SignedTransaction};

#[async_trait]
pub trait Rpc {
    async fn send_transaction(&self, tx: SignedTransaction) -> Result<Hash, Error>;

    fn listening(&self) -> Result<bool, Error>;
}

pub struct RpcExample;

#[async_trait]
impl Rpc for RpcExample {
    #[metrics_rpc("eth_sendRawTransaction")]
    async fn send_transaction(&self, tx: SignedTransaction) -> Result<Hash, Error> {
        Ok(tx.transaction.hash)
    }

    #[metrics_rpc("net_listening")]
    fn listening(&self) -> Result<bool, Error> {
        Ok(false)
    }
}

fn main() {}
