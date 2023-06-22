#![allow(clippy::uninlined_format_args, dead_code)]

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
pub use self::traits::NetworkContext;
pub use tentacle::{
    multiaddr,
    secio::{error::SecioError, KeyProvider, PeerId, SecioKeyPair},
};

use crate::error::NetworkError;
use protocol::types::Bytes;
use tentacle::secio::PublicKey;

pub trait PeerIdExt {
    fn from_pubkey_bytes<'a, B: AsRef<[u8]> + 'a>(bytes: B) -> Result<PeerId, NetworkError> {
        Ok(PeerId::from_public_key(&PublicKey::from_raw_key(
            bytes.as_ref().to_vec(),
        )))
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

/// Observe listen port occupancy
pub async fn observe_listen_port_occupancy(
    _addrs: &[multiaddr::MultiAddr],
) -> Result<(), NetworkError> {
    #[cfg(target_os = "linux")]
    {
        use std::net::{SocketAddr, TcpListener};
        use tentacle::utils::{dns::DnsResolver, multiaddr_to_socketaddr};

        for raw_addr in _addrs {
            let ip_addr: Option<SocketAddr> = match DnsResolver::new(raw_addr.clone()) {
                Some(dns) => dns.await.ok().as_ref().and_then(multiaddr_to_socketaddr),
                None => multiaddr_to_socketaddr(raw_addr),
            };

            if let Some(addr) = ip_addr {
                if let Err(e) = TcpListener::bind(addr) {
                    log::error!(
                        "addr {} can't use on your machines by error: {}, please check",
                        raw_addr,
                        e
                    );
                    return Err(NetworkError::IoError(e));
                }
            }
        }
    }

    Ok(())
}
