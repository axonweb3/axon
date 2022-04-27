use protocol::traits::Context;
use tentacle::{secio::PeerId, SessionId};

use crate::common::ConnectedAddr;
use crate::error::{ErrorKind, NetworkError};

pub trait NetworkContext: Sized {
    fn session_id(&self) -> Result<SessionId, NetworkError>;
    fn set_session_id(&mut self, sid: SessionId) -> Self;
    fn remote_peer_id(&self) -> Result<PeerId, NetworkError>;
    fn set_remote_peer_id(&mut self, pid: PeerId) -> Self;
    // This connected address is for debug purpose, so soft failure is ok.
    fn remote_connected_addr(&self) -> Option<ConnectedAddr>;
    fn set_remote_connected_addr(&mut self, addr: ConnectedAddr) -> Self;
    fn rpc_id(&self) -> Result<u64, NetworkError>;
    fn set_rpc_id(&mut self, rid: u64) -> Self;
    fn url(&self) -> Result<&str, NetworkError>;
    fn set_url(&mut self, url: String) -> Self;
}

#[derive(Debug, Clone)]
struct CtxRpcId(u64);

impl NetworkContext for Context {
    fn session_id(&self) -> Result<SessionId, NetworkError> {
        self.get::<usize>("session_id")
            .map(|sid| SessionId::new(*sid))
            .ok_or_else(|| ErrorKind::NoSessionId.into())
    }

    #[must_use]
    fn set_session_id(&mut self, sid: SessionId) -> Self {
        self.with_value::<usize>("session_id", sid.value())
    }

    fn remote_peer_id(&self) -> Result<PeerId, NetworkError> {
        self.get::<PeerId>("remote_peer_id")
            .map(ToOwned::to_owned)
            .ok_or_else(|| ErrorKind::NoRemotePeerId.into())
    }

    #[must_use]
    fn set_remote_peer_id(&mut self, pid: PeerId) -> Self {
        self.with_value::<PeerId>("remote_peer_id", pid)
    }

    fn remote_connected_addr(&self) -> Option<ConnectedAddr> {
        self.get::<ConnectedAddr>("remote_connected_addr")
            .map(ToOwned::to_owned)
    }

    #[must_use]
    fn set_remote_connected_addr(&mut self, addr: ConnectedAddr) -> Self {
        self.with_value::<ConnectedAddr>("remote_connected_addr", addr)
    }

    fn rpc_id(&self) -> Result<u64, NetworkError> {
        self.get::<CtxRpcId>("rpc_id")
            .map(|ctx_rid| ctx_rid.0)
            .ok_or_else(|| ErrorKind::NoRpcId.into())
    }

    #[must_use]
    fn set_rpc_id(&mut self, rid: u64) -> Self {
        self.with_value::<CtxRpcId>("rpc_id", CtxRpcId(rid))
    }

    fn url(&self) -> Result<&str, NetworkError> {
        self.get::<String>("url")
            .map(String::as_str)
            .ok_or_else(|| {
                NetworkError::UnexpectedError(Box::<dyn std::error::Error + Send + Sync>::from(
                    "not found",
                ))
            })
    }

    #[must_use]
    fn set_url(&mut self, url: String) -> Self {
        self.with_value::<String>("url", url)
    }
}
