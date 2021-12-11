#![allow(dead_code)]

mod common;
mod compress;
mod config;
mod endpoint;
mod error;
mod message;
mod outbound;
mod peer_manager;
mod protocols;
mod reactor;
mod rpc;
mod service;
mod traits;

pub use self::config::NetworkConfig;
pub use self::service::{NetworkService, NetworkServiceHandle};
pub use tentacle::secio::PeerId;

use crate::error::NetworkError;
use protocol::types::Bytes;
use tentacle::secio::PublicKey;

pub trait PeerIdExt {
    fn from_pubkey_bytes<'a, B: AsRef<[u8]> + 'a>(bytes: B) -> Result<PeerId, NetworkError> {
        let pubkey = PublicKey::secp256k1_raw_key(bytes.as_ref())
            .map_err(|_| NetworkError::InvalidPublicKey)?;

        Ok(PeerId::from_public_key(&pubkey))
    }

    fn from_bytes<'a, B: AsRef<[u8]> + 'a>(bytes: B) -> Result<PeerId, NetworkError> {
        PeerId::from_bytes(bytes.as_ref().to_vec()).map_err(|_| NetworkError::InvalidPeerId)
    }

    fn to_string(&self) -> String;

    fn into_bytes_ext(self) -> Bytes;

    fn from_str_ext<'a, S: AsRef<str> + 'a>(s: S) -> Result<PeerId, NetworkError> {
        s.as_ref().parse().map_err(|_| NetworkError::InvalidPeerId)
    }
}

impl PeerIdExt for PeerId {
    fn into_bytes_ext(self) -> Bytes {
        Bytes::from(self.into_bytes())
    }

    fn to_string(&self) -> String {
        self.to_base58()
    }
}
