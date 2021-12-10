use derive_more::Display;
use serde::{Deserialize, Serialize};
use tentacle::multiaddr::Multiaddr;

#[derive(Debug, Display, PartialEq, Eq, Serialize, Deserialize, Clone, Hash)]
#[display(fmt = "{}:{}", host, port)]
pub struct ConnectedAddr {
    pub host: String,
    pub port: u16,
}

impl From<&Multiaddr> for ConnectedAddr {
    fn from(multiaddr: &Multiaddr) -> Self {
        use tentacle::multiaddr::Protocol::{Dns4, Dns6, Ip4, Ip6, Tcp, Tls};

        let mut host = None;
        let mut port = 0u16;

        for comp in multiaddr.iter() {
            match comp {
                Ip4(ip_addr) => host = Some(ip_addr.to_string()),
                Ip6(ip_addr) => host = Some(ip_addr.to_string()),
                Dns4(dns_addr) | Dns6(dns_addr) => host = Some(dns_addr.to_string()),
                Tls(tls_addr) => host = Some(tls_addr.to_string()),
                Tcp(p) => port = p,
                _ => (),
            }
        }

        let host = host.unwrap_or_else(|| multiaddr.to_string());
        ConnectedAddr { host, port }
    }
}
