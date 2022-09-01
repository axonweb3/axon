use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

use crate::types::{HashWithDirection, RequestTxHashes, Requests, Transfer};

impl Encodable for Transfer {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(6)
            .append(&(self.direction as u8))
            .append(&self.tx_hash)
            .append(&self.address)
            .append(&self.erc20_address)
            .append(&self.ckb_amount)
            .append(&self.sudt_amount);
    }
}

impl Decodable for Transfer {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(Transfer {
            direction:     rlp
                .val_at::<u8>(0)?
                .try_into()
                .map_err(|_| DecoderError::Custom("Invalid transfer direction"))?,
            tx_hash:       rlp.val_at(1)?,
            address:       rlp.val_at(2)?,
            erc20_address: rlp.val_at(3)?,
            ckb_amount:    rlp.val_at(4)?,
            sudt_amount:   rlp.val_at(5)?,
        })
    }
}

impl Encodable for Requests {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_list(&self.0);
    }
}

impl Decodable for Requests {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(Requests(rlp.as_list()?))
    }
}

impl Encodable for RequestTxHashes {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(2)
            .append(&(self.direction as u8))
            .append_list(&self.tx_hashes);
    }
}

impl Decodable for RequestTxHashes {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(RequestTxHashes {
            direction: rlp
                .val_at::<u8>(0)?
                .try_into()
                .map_err(|_| DecoderError::Custom("Invalid transfer direction"))?,
            tx_hashes: rlp.list_at(1)?,
        })
    }
}

impl Encodable for HashWithDirection {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(2)
            .append(&self.tx_hash)
            .append(&(self.direction as u8));
    }
}

impl Decodable for HashWithDirection {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(HashWithDirection {
            tx_hash:   rlp.val_at(0)?,
            direction: rlp
                .val_at::<u8>(0)?
                .try_into()
                .map_err(|_| DecoderError::Custom("Invalid transfer direction"))?,
        })
    }
}

#[cfg(feature = "ibc")]
pub mod ibc {
    use cosmos_ibc::core::{
        ics02_client::{
            client_consensus::AnyConsensusState, client_state::AnyClientState,
            client_type::ClientType,
        },
        ics03_connection::connection::ConnectionEnd,
        ics04_channel::{
            channel::ChannelEnd,
            commitment::{AcknowledgementCommitment, PacketCommitment},
            packet::Sequence,
        },
        ics24_host::{
            identifier::ConnectionId,
            path::{
                AcksPath, ChannelEndsPath, ClientConnectionsPath, ClientConsensusStatePath,
                ClientStatePath, ClientTypePath, CommitmentsPath, ConnectionsPath, ReceiptsPath,
                SeqAcksPath, SeqRecvsPath, SeqSendsPath,
            },
        },
    };

    use crate::codec::ProtocolCodec;
    use crate::{ProtocolError, ProtocolErrorKind, ProtocolResult};

    #[derive(Clone)]
    pub struct IbcWrapper<T: Clone>(pub T);

    macro_rules! todo_codec {
        ($name: ty) => {
            impl ProtocolCodec for IbcWrapper<$name> {
                fn encode(&self) -> ProtocolResult<bytes::Bytes> {
                    todo!()
                }

                fn decode<B: AsRef<[u8]>>(_bytes: B) -> ProtocolResult<Self> {
                    todo!()
                }
            }
        };
    }

    macro_rules! json_codec_impl {
        ($name:ty) => {
            impl ProtocolCodec for IbcWrapper<$name> {
                fn encode(&self) -> ProtocolResult<bytes::Bytes> {
                    match serde_json::to_vec(&self.0) {
                        Ok(v) => Ok(v.into()),
                        Err(e) => Err(ProtocolError {
                            kind:  ProtocolErrorKind::CrossChain,
                            error: Box::new(e),
                        }),
                    }
                }

                fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self> {
                    match serde_json::from_slice(bytes.as_ref()) {
                        Ok(v) => Ok(IbcWrapper(v)),
                        Err(e) => Err(ProtocolError {
                            kind:  ProtocolErrorKind::CrossChain,
                            error: Box::new(e),
                        }),
                    }
                }
            }
        };
    }

    json_codec_impl!(ClientType);
    json_codec_impl!(());
    json_codec_impl!(Sequence);
    json_codec_impl!(Vec<ConnectionId>);
    json_codec_impl!(ChannelEnd);
    json_codec_impl!(ConnectionEnd);
    json_codec_impl!(PacketCommitment);
    json_codec_impl!(AcknowledgementCommitment);
    todo_codec!(AnyClientState);
    todo_codec!(AnyConsensusState);
    todo_codec!(ClientTypePath);
    todo_codec!(ClientStatePath);
    todo_codec!(ClientConsensusStatePath);
    todo_codec!(SeqSendsPath);
    todo_codec!(SeqRecvsPath);
    todo_codec!(SeqAcksPath);
    todo_codec!(CommitmentsPath);
    todo_codec!(AcksPath);
    todo_codec!(ReceiptsPath);
    todo_codec!(ChannelEndsPath);
    todo_codec!(ConnectionsPath);
    todo_codec!(ClientConnectionsPath);
}

#[cfg(test)]
mod tests {
    use crate::types::{Hash, H160};
    use rand::random;

    use super::*;

    fn random_transfer() -> Transfer {
        Transfer {
            direction:     0u8.try_into().unwrap(),
            tx_hash:       Hash::random(),
            address:       H160::random(),
            erc20_address: H160::random(),
            ckb_amount:    random(),
            sudt_amount:   random(),
        }
    }

    #[test]
    fn test_transfer_codec() {
        let origin = random_transfer();
        let raw = rlp::encode(&origin);
        let decode = rlp::decode::<Transfer>(&raw.freeze()).unwrap();
        assert_eq!(origin, decode);
    }

    #[test]
    fn test_requests_codec() {
        let origin = Requests((0..10).map(|_| random_transfer()).collect());
        let raw = rlp::encode(&origin);
        let decode = rlp::decode::<Requests>(&raw.freeze()).unwrap();
        assert_eq!(origin, decode);
    }
}
