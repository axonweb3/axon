mod discovery;
mod feeler;
mod identify;
mod ping;
mod transmitter;

pub use self::{
    discovery::{DiscoveryAddressManager, DiscoveryProtocol},
    feeler::Feeler,
    identify::IdentifyProtocol,
    ping::PingHandler,
    transmitter::{protocol::ReceivedMessage, TransmitterProtocol},
};
use crate::compress::{compress, decompress};
use tentacle::{
    builder::MetaBuilder,
    service::{ProtocolHandle, ProtocolMeta},
    traits::ServiceProtocol,
    ProtocolId,
};

#[derive(Clone, Debug, Copy)]
pub enum SupportProtocols {
    Ping,
    Identify,
    Discovery,
    Transmitter,
    Feeler,
}

impl SupportProtocols {
    pub fn protocol_id(&self) -> ProtocolId {
        match self {
            SupportProtocols::Ping => 1,
            SupportProtocols::Identify => 2,
            SupportProtocols::Discovery => 3,
            SupportProtocols::Transmitter => 4,
            SupportProtocols::Feeler => 5,
        }
        .into()
    }

    pub fn name(&self) -> String {
        match self {
            SupportProtocols::Ping => "/axon/ping",
            SupportProtocols::Identify => "/axon/identify",
            SupportProtocols::Discovery => "/axon/discovery",
            SupportProtocols::Transmitter => "/axon/transmitter",
            SupportProtocols::Feeler => "/axon/feeler",
        }
        .to_owned()
    }

    pub fn support_versions(&self) -> Vec<String> {
        match self {
            SupportProtocols::Ping => vec!["1".to_owned()],
            SupportProtocols::Identify => vec!["1".to_owned()],
            SupportProtocols::Discovery => vec!["1".to_owned()],
            SupportProtocols::Transmitter => vec!["1".to_owned()],
            SupportProtocols::Feeler => vec!["1".to_owned()],
        }
    }

    pub fn max_frame_length(&self) -> usize {
        match self {
            SupportProtocols::Ping => 1024,
            SupportProtocols::Identify => 2 * 1024,
            SupportProtocols::Discovery => 512 * 1024,
            SupportProtocols::Transmitter => 4 * 1024 * 1024,
            SupportProtocols::Feeler => 1024,
        }
    }

    pub fn build_meta_with_service_handle<
        SH: FnOnce() -> ProtocolHandle<Box<dyn ServiceProtocol + Send + 'static + Unpin>>,
    >(
        self,
        service_handle: SH,
    ) -> ProtocolMeta {
        let meta_builder: MetaBuilder = self.into();
        meta_builder.service_handle(service_handle).build()
    }
}

impl From<SupportProtocols> for MetaBuilder {
    fn from(p: SupportProtocols) -> Self {
        let max_frame_length = p.max_frame_length();
        MetaBuilder::default()
            .id(p.protocol_id())
            .support_versions(p.support_versions())
            .name(move |_| p.name())
            .before_send(compress)
            .before_receive(|| Some(Box::new(decompress)))
            .codec(move || {
                Box::new(
                    tokio_util::codec::length_delimited::Builder::new()
                        .max_frame_length(max_frame_length)
                        .new_codec(),
                )
            })
    }
}
