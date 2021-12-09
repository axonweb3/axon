mod pubsub;
mod stream_codec;

pub use pubsub::Client;

use ckb_jsonrpc_types as ckb;

use protocol::tokio::io::{AsyncRead, AsyncWrite};
use protocol::tokio::net::TcpStream;
use protocol::traits::{CkbClient, Context};
use protocol::types::{Header, Transaction, Validator};
use protocol::{async_trait, ProtocolResult};

pub type ClientTcpImpl = Client<TcpStream>;

#[async_trait]
impl CkbClient for ClientTcpImpl {
    async fn get_validator_list(&self, ctx: Context) -> ProtocolResult<Vec<Validator>> {
        Ok(vec![])
    }

    async fn watch_cross_tx(&self, ctx: Context) -> ProtocolResult<Transaction> {
        todo!()
    }

    async fn verify_check_point(&self, ctx: Context, header: Header) -> ProtocolResult<()> {
        Ok(())
    }
}
