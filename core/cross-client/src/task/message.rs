use ckb_types::{core::TransactionView, packed, prelude::*};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

use common_crypto::{
    BlsPublicKey, BlsSignature, BlsSignatureVerify, HashValue, PublicKey, Signature,
};
use protocol::async_trait;
use protocol::tokio::sync::mpsc::UnboundedSender;
use protocol::traits::{Context, MessageHandler, TrustFeedback};
use protocol::types::{Bytes, H256};

pub const END_GOSSIP_BUILD_CKB_TX: &str = "/gossip/crosschain/ckb_tx";
pub const END_GOSSIP_CKB_TX_SIGNATURE: &str = "/gossip/crosschain/signature";

pub struct CrossChainMessageHandler(UnboundedSender<CrossChainMessage>);

#[async_trait]
impl MessageHandler for CrossChainMessageHandler {
    type Message = CrossChainMessage;

    async fn process(&self, _ctx: Context, msg: Self::Message) -> TrustFeedback {
        if let Err(e) = self.0.send(msg) {
            log::warn!("set crosschain message {:?}", e);
            return TrustFeedback::Worse(e.to_string());
        }

        TrustFeedback::Good
    }
}

impl CrossChainMessageHandler {
    pub fn new(sender: UnboundedSender<CrossChainMessage>) -> Self {
        CrossChainMessageHandler(sender)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum CrossChainMessage {
    TxView(TxViewWrapper),
    Signature(CrossChainSignature),
}

impl Encodable for CrossChainMessage {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(2);
        match self {
            CrossChainMessage::TxView(hash) => s.append(&0u8).append(hash),
            CrossChainMessage::Signature(sig) => s.append(&1u8).append(sig),
        };
    }
}

impl Decodable for CrossChainMessage {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let ty: u8 = rlp.val_at(0)?;
        match ty {
            0 => Ok(CrossChainMessage::TxView(rlp.val_at(1)?)),
            1 => Ok(CrossChainMessage::Signature(rlp.val_at(1)?)),
            _ => Err(DecoderError::Custom("Invalid message type")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TxViewWrapper {
    pub req_hash: H256,
    pub tx_view:  TransactionView,
}

impl TxViewWrapper {
    pub fn new(req_hash: H256, tx_view: TransactionView) -> Self {
        TxViewWrapper { req_hash, tx_view }
    }
}

impl Encodable for TxViewWrapper {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(2)
            .append(&self.req_hash)
            .append(&self.tx_view.pack().as_bytes());
    }
}

impl Decodable for TxViewWrapper {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(TxViewWrapper {
            req_hash: rlp.val_at(0)?,
            tx_view:  packed::TransactionView::new_unchecked(rlp.val_at(1)?).unpack(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct CrossChainSignature {
    pub req_hash:   H256,
    pub tx_hash:    H256,
    pub bls_pubkey: BlsPublicKey,
    pub signature:  BlsSignature,
}

impl CrossChainSignature {
    pub fn verify(&self) -> bool {
        self.signature
            .verify(
                &HashValue::from_bytes_unchecked(self.tx_hash.0),
                &self.bls_pubkey,
                &String::new(),
            )
            .is_ok()
    }
}

impl Encodable for CrossChainSignature {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(4)
            .append(&self.req_hash)
            .append(&self.tx_hash)
            .append(&self.bls_pubkey.to_bytes())
            .append(&self.signature.to_bytes());
    }
}

impl Decodable for CrossChainSignature {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(CrossChainSignature {
            req_hash:   rlp.val_at(0)?,
            tx_hash:    rlp.val_at(1)?,
            bls_pubkey: {
                let raw: Bytes = rlp.val_at(2)?;
                raw.as_ref()
                    .try_into()
                    .map_err(|_| DecoderError::Custom("decode public key"))?
            },
            signature:  {
                let raw: Bytes = rlp.val_at(3)?;
                raw.as_ref()
                    .try_into()
                    .map_err(|_| DecoderError::Custom("decode public key"))?
            },
        })
    }
}
