use overlord::Codec;
use rlp_derive::{RlpDecodable, RlpEncodable};

use protocol::codec::{ProtocolCodec, ProtocolListCodec};
use protocol::types::{Block, Bytes, Hash, SignedTransaction};
use protocol::{traits::MessageCodec, ProtocolResult};

use crate::{ConsensusError, ConsensusType};

#[derive(Clone, Debug)]
pub enum ConsensusRpcRequest {
    PullBlocks(u64),
    PullTxs(PullTxsRequest),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsensusRpcResponse {
    PullBlocks(Box<Block>),
    PullTxs(Box<Vec<SignedTransaction>>),
}

impl MessageCodec for ConsensusRpcResponse {
    fn encode_msg(&mut self) -> ProtocolResult<Bytes> {
        let bytes = match self {
            ConsensusRpcResponse::PullBlocks(ep) => {
                let mut tmp = ep.as_ref().encode()?.as_mut();
                tmp.extend_from_slice(b"a");
                tmp
            }

            ConsensusRpcResponse::PullTxs(txs) => {
                let mut tmp = txs.encode_list()?.as_mut();
                tmp.extend_from_slice(b"b");
                tmp
            }
        };
        Ok(bytes.freeze())
    }

    fn decode_msg(mut bytes: Bytes) -> ProtocolResult<Self> {
        let len = bytes.len();
        let flag = bytes.split_off(len - 1);

        match flag.as_ref() {
            b"a" => {
                let res = Block::decode_fixed(bytes)?;
                Ok(ConsensusRpcResponse::PullBlocks(Box::new(res)))
            }

            b"b" => {
                let res = Vec::<SignedTransaction>::decode_list(&bytes)
                    .map_err(|_| ConsensusError::DecodeErr(ConsensusType::RpcPullTxs))?;
                Ok(ConsensusRpcResponse::PullTxs(Box::new(res)))
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, RlpEncodable, RlpDecodable)]
pub struct PullTxsRequest {
    pub height: u64,
    pub inner:  Vec<Hash>,
}

impl PullTxsRequest {
    pub fn new(height: u64, inner: Vec<Hash>) -> Self {
        PullTxsRequest { height, inner }
    }
}
