use tentacle::{bytes::Bytes, secio::PeerId, SessionId};

#[derive(Debug)]
pub struct ReceivedMessage {
    pub session_id: SessionId,
    pub peer_id:    PeerId,
    pub data:       Bytes,
}
