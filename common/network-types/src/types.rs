use hopr_crypto_types::prelude::PeerId;
use std::fmt::{Display, Formatter};
use std::net::ToSocketAddrs;
use std::str::FromStr;

use crate::errors::NetworkTypeError;
use hopr_primitive_types::bounded::{BoundedSize, BoundedVec};

#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::Display)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl std::net::ToSocketAddrs for IpOrHost {
    type Iter = std::vec::IntoIter<std::net::SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        match &self {
            IpOrHost::Dns(host, port) => Ok(hickory_resolver::Resolver::default()?
                .lookup_ip(host)?
                .into_iter()
                .map(|ip| std::net::SocketAddr::new(ip, *port))
                .collect::<Vec<_>>()
                .into_iter()),
            IpOrHost::Ip(addr) => Ok(vec![*addr].into_iter()),
        }
    }
}

impl IpOrHost {
    /// Tries to resolve the DNS name and returns the first IP address found.
    /// If this enum is already an IP address and port, it will simply return it.
    pub fn resolve_first(self) -> Option<std::net::SocketAddr> {
        self.to_socket_addrs().ok()?.next()
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
