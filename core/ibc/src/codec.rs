use ibc::core::{
    ics02_client::{
        client_consensus::AnyConsensusState, client_state::AnyClientState, client_type::ClientType,
    },
    ics03_connection::connection::ConnectionEnd,
    ics04_channel::{
        channel::ChannelEnd,
        commitment::{AcknowledgementCommitment, PacketCommitment},
        packet::Sequence,
    },
    ics24_host::identifier::ConnectionId,
};
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::channel::v1::Channel as RawChannelEnd;
use ibc_proto::ibc::core::connection::v1::ConnectionEnd as RawConnectionEnd;
use tendermint_proto::Protobuf;

pub trait Codec: Sized {
    fn encode(&self) -> Option<Vec<u8>>;
    fn decode(raw: &[u8]) -> Option<Self>;
}

macro_rules! json_codec {
    ($name:ty) => {
        impl Codec for $name {
            fn encode(&self) -> Option<Vec<u8>> {
                serde_json::to_string(&self).ok().map(|v| v.into_bytes())
            }

            fn decode(raw: &[u8]) -> Option<Self> {
                let json_string = String::from_utf8(raw.to_vec()).ok()?;
                serde_json::from_str(&json_string).ok()
            }
        }
    };
}

macro_rules! protobuf_codec {
    ($name:ty, $raw:ty) => {
        impl Codec for $name {
            fn encode(&self) -> Option<Vec<u8>> {
                self.encode_vec().ok()
            }

            fn decode(raw: &[u8]) -> Option<Self> {
                <Self as Protobuf<$raw>>::decode(raw).ok()
            }
        }
    };
}

macro_rules! bin_codec {
    ($name:ty) => {
        impl Codec for $name {
            fn encode(&self) -> Option<Vec<u8>> {
                Some(self.as_ref().to_vec())
            }

            fn decode(raw: &[u8]) -> Option<Self> {
                Some(raw.to_vec().into())
            }
        }
    };
}

json_codec!(());
json_codec!(ClientType);
json_codec!(Sequence);
json_codec!(Vec<ConnectionId>);
protobuf_codec!(AnyClientState, Any);
protobuf_codec!(AnyConsensusState, Any);
protobuf_codec!(ChannelEnd, RawChannelEnd);
protobuf_codec!(ConnectionEnd, RawConnectionEnd);
bin_codec!(PacketCommitment);
bin_codec!(AcknowledgementCommitment);
