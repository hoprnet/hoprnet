use crate::errors::NetworkTypeError;
use hopr_primitive_types::bounded::{BoundedSize, BoundedVec};
use libp2p_identity::PeerId;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::str::FromStr;

/// Lists some of the IP protocols.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, strum::Display, strum::EnumString)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum IpProtocol {
    TCP,
    UDP,
}

/// Implements a host name with port.
/// This could be either a DNS name with port
/// or an IP address with port represented by [`std::net::SocketAddr`].
///
/// This object implements [`std::net::ToSocketAddrs`] which performs automatic
/// DNS name resolution in case this is a [`IpOrHost::Dns`] instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IpOrHost {
    /// DNS name and port.
    Dns(String, u16),
    /// IP address with port.
    Ip(std::net::SocketAddr),
}

impl Display for IpOrHost {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            IpOrHost::Dns(host, port) => write!(f, "{host}:{port}"),
            IpOrHost::Ip(ip) => write!(f, "{ip}"),
        }
    }
}

impl FromStr for IpOrHost {
    type Err = NetworkTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(addr) = std::net::SocketAddr::from_str(s) {
            Ok(IpOrHost::Ip(addr))
        } else {
            s.split_once(":")
                .ok_or(NetworkTypeError::Other("missing port delimiter".into()))
                .and_then(|(host, port_str)| {
                    u16::from_str(port_str)
                        .map(|port| IpOrHost::Dns(host.to_string(), port))
                        .map_err(|_| NetworkTypeError::Other("invalid port number".into()))
                })
        }
    }
}

impl From<std::net::SocketAddr> for IpOrHost {
    fn from(value: std::net::SocketAddr) -> Self {
        IpOrHost::Ip(value)
    }
}

impl IpOrHost {
    /// Tries to resolve the DNS name and returns all IP addresses found.
    /// If this enum is already an IP address and port, it will simply return it.
    pub async fn resolve(self) -> std::io::Result<Vec<SocketAddr>> {
        match self {
            IpOrHost::Dns(name, port) => {
                #[cfg(all(feature = "runtime-tokio", not(test)))]
                let resolver = hickory_resolver::AsyncResolver::tokio_from_system_conf()?;

                #[cfg(any(all(feature = "runtime-async-std", not(feature = "runtime-tokio")), test))]
                let resolver = async_std_resolver::resolver_from_system_conf().await?;

                Ok(resolver
                    .lookup_ip(name)
                    .await?
                    .into_iter()
                    .map(|ip| SocketAddr::new(ip, port))
                    .collect())
            }
            IpOrHost::Ip(addr) => Ok(vec![addr]),
        }
    }

    /// Gets the port number.
    pub fn port(&self) -> u16 {
        match &self {
            IpOrHost::Dns(_, port) => *port,
            IpOrHost::Ip(addr) => addr.port(),
        }
    }

    /// Gets the unresolved DNS name or IP address as string.
    pub fn host(&self) -> String {
        match &self {
            IpOrHost::Dns(host, _) => host.clone(),
            IpOrHost::Ip(addr) => addr.ip().to_string(),
        }
    }

    /// Checks if this instance is a [DNS name](IpOrHost::Dns).
    pub fn is_dns(&self) -> bool {
        matches!(self, IpOrHost::Dns(..))
    }

    /// Checks if this instance is an [IP address](IpOrHost::Ip) and whether it is
    /// an IPv4 address.
    ///
    /// Always returns `false` if this instance is a [DNS name](IpOrHost::Dns),
    /// i.e.: it does not perform any DNS resolution.
    pub fn is_ipv4(&self) -> bool {
        matches!(self, IpOrHost::Ip(addr) if addr.is_ipv4())
    }

    /// Checks if this instance is an [IP address](IpOrHost::Ip) and whether it is
    /// an IPv6 address.
    ///
    /// Always returns `false` if this instance is a [DNS name](IpOrHost::Dns),
    /// i.e.: it does not perform any DNS resolution.
    pub fn is_ipv6(&self) -> bool {
        matches!(self, IpOrHost::Ip(addr) if addr.is_ipv6())
    }

    /// Checks if this instance is an [IP address](IpOrHost::Ip) and whether it is
    /// a loopback address.
    ///
    /// Always returns `false` if this instance is a [DNS name](IpOrHost::Dns),
    /// i.e.: it does not perform any DNS resolution.
    pub fn is_loopback_ip(&self) -> bool {
        matches!(self, IpOrHost::Ip(addr) if addr.ip().is_loopback())
    }
}

/// Represents routing options in a mixnet with a maximum number of hops.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RoutingOptions {
    /// A fixed intermediate path consisting of at most [`RoutingOptions::MAX_INTERMEDIATE_HOPS`] hops.
    IntermediatePath(BoundedVec<PeerId, { RoutingOptions::MAX_INTERMEDIATE_HOPS }>),
    /// Random intermediate path with at least the given number of hops,
    /// but at most [`RoutingOptions::MAX_INTERMEDIATE_HOPS`].
    Hops(BoundedSize<{ RoutingOptions::MAX_INTERMEDIATE_HOPS }>),
}

impl RoutingOptions {
    /// The maximum number of hops this instance can represent.
    pub const MAX_INTERMEDIATE_HOPS: usize = 3;

    /// Inverts the intermediate path if this is an instance of [`RoutingOptions::IntermediatePath`].
    /// Otherwise, does nothing.
    pub fn invert(self) -> RoutingOptions {
        match self {
            RoutingOptions::IntermediatePath(v) => RoutingOptions::IntermediatePath(v.into_iter().rev().collect()),
            _ => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use std::net::SocketAddr;

    #[async_std::test]
    async fn ip_or_host_must_resolve_dns_name() -> anyhow::Result<()> {
        match IpOrHost::Dns("localhost".to_string(), 1000)
            .resolve()
            .await?
            .first()
            .ok_or(anyhow!("must resolve"))?
        {
            SocketAddr::V4(addr) => assert_eq!(*addr, "127.0.0.1:1000".parse()?),
            SocketAddr::V6(addr) => assert_eq!(*addr, "::1:1000".parse()?),
        }
        Ok(())
    }

    #[async_std::test]
    async fn ip_or_host_must_resolve_ip_address() -> anyhow::Result<()> {
        assert_eq!(
            *IpOrHost::Ip("127.0.0.1:1000".parse()?)
                .resolve()
                .await?
                .first()
                .ok_or(anyhow!("must resolve"))?,
            "127.0.0.1:1000".parse()?
        );
        Ok(())
    }

    #[test]
    fn ip_or_host_should_parse_from_string() -> anyhow::Result<()> {
        assert_eq!(
            IpOrHost::Dns("some.dns.name.info".into(), 1234),
            IpOrHost::from_str("some.dns.name.info:1234")?
        );
        assert_eq!(
            IpOrHost::Ip("1.2.3.4:1234".parse()?),
            IpOrHost::from_str("1.2.3.4:1234")?
        );
        Ok(())
    }
}
