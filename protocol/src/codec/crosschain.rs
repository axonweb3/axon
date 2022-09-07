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
    use std::str::FromStr;

    use bincode;
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
            Path,
        },
    };
    use ibc_proto::google::protobuf::Any;
    use ibc_proto::ibc::core::{
        channel::v1::Channel as RawChannelEnd, connection::v1::ConnectionEnd as RawConnectionEnd,
    };
    use tendermint_proto::Protobuf;

    use crate::codec::error::CodecError;
    use crate::codec::ProtocolCodec;
    use crate::{ProtocolError, ProtocolResult};

    #[derive(Clone)]
    pub struct IbcWrapper<T: Clone>(pub T);

    macro_rules! protobuf_codec_impl {
        ($name:ty, $raw:ident) => {
            impl ProtocolCodec for IbcWrapper<$name> {
                fn encode(&self) -> ProtocolResult<bytes::Bytes> {
                    let res = self
                        .0
                        .encode_vec()
                        .map_err(|e| ProtocolError::from(CodecError::Ibc(e.to_string())))?;
                    Ok(res.into())
                }

                fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self> {
                    let res = <$name as Protobuf<$raw>>::decode_vec(bytes.as_ref())
                        .map_err(|e| ProtocolError::from(CodecError::Ibc(e.to_string())))?;
                    Ok(IbcWrapper(res.into()))
                }
            }
        };
    }

    macro_rules! path_codec_impl {
        ($name:ty, $variant:ident) => {
            impl ProtocolCodec for IbcWrapper<$name> {
                fn encode(&self) -> ProtocolResult<bytes::Bytes> {
                    let path: Path = self.0.clone().into();
                    let string = path.to_string();
                    ProtocolCodec::encode(&string)
                }

                fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self> {
                    let raw = <String as ProtocolCodec>::decode(bytes.as_ref())?;
                    let path = Path::from_str(&raw).unwrap();
                    if let Path::$variant(p) = path {
                        Ok(IbcWrapper(p))
                    } else {
                        Err(ProtocolError::from(CodecError::Ibc(raw)))
                    }
                }
            }
        };
    }

    macro_rules! raw_codec_impl {
        ($name:ty) => {
            impl ProtocolCodec for IbcWrapper<$name> {
                fn encode(&self) -> ProtocolResult<bytes::Bytes> {
                    Ok(self.0.as_ref().to_vec().into())
                }

                fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self> {
                    Ok(IbcWrapper(bytes.as_ref().to_vec().into()))
                }
            }
        };
    }

    macro_rules! bincode_codec_impl {
        ($name:ty) => {
            impl ProtocolCodec for IbcWrapper<$name> {
                fn encode(&self) -> ProtocolResult<bytes::Bytes> {
                    let r = bincode::serialize(&self.0)
                        .map_err(|e| ProtocolError::from(CodecError::Ibc(e.to_string())))?;
                    Ok(r.into())
                }

                fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self> {
                    let r = bincode::deserialize(bytes.as_ref())
                        .map_err(|e| ProtocolError::from(CodecError::Ibc(e.to_string())))?;
                    Ok(IbcWrapper(r))
                }
            }
        };
    }

    bincode_codec_impl!(());
    bincode_codec_impl!(ClientType);
    bincode_codec_impl!(Sequence);
    bincode_codec_impl!(Vec<ConnectionId>);
    raw_codec_impl!(PacketCommitment);
    raw_codec_impl!(AcknowledgementCommitment);
    protobuf_codec_impl!(AnyClientState, Any);
    protobuf_codec_impl!(AnyConsensusState, Any);
    protobuf_codec_impl!(ChannelEnd, RawChannelEnd);
    protobuf_codec_impl!(ConnectionEnd, RawConnectionEnd);
    path_codec_impl!(ClientTypePath, ClientType);
    path_codec_impl!(ClientStatePath, ClientState);
    path_codec_impl!(ClientConsensusStatePath, ClientConsensusState);
    path_codec_impl!(SeqSendsPath, SeqSends);
    path_codec_impl!(SeqRecvsPath, SeqRecvs);
    path_codec_impl!(SeqAcksPath, SeqAcks);
    path_codec_impl!(CommitmentsPath, Commitments);
    path_codec_impl!(AcksPath, Acks);
    path_codec_impl!(ReceiptsPath, Receipts);
    path_codec_impl!(ChannelEndsPath, ChannelEnds);
    path_codec_impl!(ConnectionsPath, Connections);
    path_codec_impl!(ClientConnectionsPath, ClientConnections);

    #[test]
    fn test_ibc_path_codec() {
        use cosmos_ibc::core::ics24_host::identifier::ClientId;
        let path = ClientTypePath(ClientId::new(ClientType::Tendermint, 0).unwrap());
        let wrap = IbcWrapper(path.clone());
        let encoded = wrap.encode().unwrap();
        let actual = IbcWrapper::<ClientTypePath>::decode(encoded).unwrap();
        assert_eq!(path, actual.0);
    }
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
