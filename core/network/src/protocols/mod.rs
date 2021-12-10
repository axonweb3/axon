mod discovery;
mod identify;
mod ping;
mod transmitter;

pub use self::{
    discovery::{DiscoveryAddressManager, DiscoveryProtocol},
    identify::IdentifyProtocol,
    ping::PingHandler,
    transmitter::{protocol::ReceivedMessage, TransmitterProtocol},
};

pub const PING_PROTOCOL_ID: usize = 1;
pub const IDENTIFY_PROTOCOL_ID: usize = 2;
pub const DISCOVERY_PROTOCOL_ID: usize = 3;
pub const TRANSMITTER_PROTOCOL_ID: usize = 4;
