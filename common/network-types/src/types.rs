use crate::errors::NetworkTypeError;
use hickory_resolver::name_server::ConnectionProvider;
use hickory_resolver::AsyncResolver;
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
    ///
    /// Uses `async-std` resolver.
    #[cfg(all(not(test), feature = "runtime-async-std"))]
    pub async fn resolve_async_std(self) -> std::io::Result<Vec<SocketAddr>> {
        let resolver = async_std_resolver::resolver_from_system_conf().await?;
        self.resolve(resolver).await
    }

    /// Tries to resolve the DNS name and returns all IP addresses found.
    /// If this enum is already an IP address and port, it will simply return it.
    ///
    /// Uses `tokio` resolver.
    #[cfg(feature = "runtime-tokio")]
    pub async fn resolve_tokio(self) -> std::io::Result<Vec<SocketAddr>> {
        let resolver = hickory_resolver::AsyncResolver::tokio_from_system_conf()?;
        self.resolve(resolver).await
    }

    // This resolver setup is used in our tests because these are executed in a sandbox environment
    // which prevents IO access to system-level files.
    #[cfg(all(test, feature = "runtime-async-std"))]
    pub async fn resolve_async_std(self) -> std::io::Result<Vec<SocketAddr>> {
        let config = async_std_resolver::config::ResolverConfig::new();
        let opts = async_std_resolver::config::ResolverOpts::default();
        let resolver = async_std_resolver::resolver(config, opts).await;
        self.resolve(resolver).await
    }

    /// Tries to resolve the DNS name and returns all IP addresses found.
    /// If this enum is already an IP address and port, it will simply return it.
    pub async fn resolve<P: ConnectionProvider>(self, resolver: AsyncResolver<P>) -> std::io::Result<Vec<SocketAddr>> {
        match self {
            IpOrHost::Dns(name, port) => Ok(resolver
                .lookup_ip(name)
                .await?
                .into_iter()
                .map(|ip| SocketAddr::new(ip, port))
                .collect()),
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

/// Contains an optionally encrypted [`IpOrHost`].
///
/// This is useful for hiding the [`IpOrHost`] instance from the Entry node.
/// The client first encrypts the `IpOrHost` instance via [`SealedHost::seal`] using
/// the Exit node's public key.
/// Upon receiving the `SealedHost` instance by the Exit node, it can call
/// [`SealedHost::unseal`] using its private key to get the original `IpOrHost` instance.
///
/// Sealing is fully randomized and therefore does not leak information about equal `IpOrHost`
/// instances.
///
/// ### Example
/// ````rust
/// use libp2p_identity::PeerId;
/// use hopr_crypto_types::prelude::{Keypair, OffchainKeypair};
/// use hopr_network_types::prelude::{IpOrHost, SealedHost};
///
/// # fn main() -> anyhow::Result<()> {
/// let keypair = OffchainKeypair::random();
///
/// let exit_node_peer_id: PeerId = keypair.public().into();
/// let host: IpOrHost = "127.0.0.1:1000".parse()?;
///
/// // On the Client
/// let encrypted = SealedHost::seal(host.clone(), keypair.public().into())?;
///
/// // On the Exit node
/// let decrypted = encrypted.unseal(&keypair)?;
/// assert_eq!(host, decrypted);
///
/// // Plain SealedHost unseals trivially
/// let plain_sealed: SealedHost = host.clone().into();
/// assert_eq!(host, plain_sealed.try_into()?);
///
/// // The same host sealing is randomized
/// let encrypted_1 = SealedHost::seal(host.clone(), keypair.public().into())?;
/// let encrypted_2 = SealedHost::seal(host.clone(), keypair.public().into())?;
/// assert_ne!(encrypted_1, encrypted_2);
///
/// # Ok(())
/// # }
/// ````
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SealedHost {
    /// Plain (not sealed) [`IpOrHost`]
    Plain(IpOrHost),
    /// Encrypted [`IpOrHost`]
    Sealed(Box<[u8]>),
}

#[cfg(feature = "serde")] // Serde feature required so that `IpOrHost` is serializable
impl SealedHost {
    /// Seals the given [`IpOrHost`] using the Exit node's peer ID.
    pub fn seal(host: IpOrHost, peer_id: PeerId) -> crate::errors::Result<Self> {
        hopr_crypto_types::seal::seal_data(host, peer_id)
            .map(Self::Sealed)
            .map_err(|e| NetworkTypeError::Other(e.to_string()))
    }

    /// Tries to unseal the sealed [`IpOrHost`] using the private key as Exit node.
    /// No-op, if the data is already unsealed.
    pub fn unseal(self, key: &hopr_crypto_types::keypairs::OffchainKeypair) -> crate::errors::Result<IpOrHost> {
        match self {
            SealedHost::Plain(host) => Ok(host),
            SealedHost::Sealed(enc) => {
                hopr_crypto_types::seal::unseal_data(&enc, key).map_err(|e| NetworkTypeError::Other(e.to_string()))
            }
        }
    }
}

impl From<IpOrHost> for SealedHost {
    fn from(value: IpOrHost) -> Self {
        Self::Plain(value)
    }
}

impl TryFrom<SealedHost> for IpOrHost {
    type Error = NetworkTypeError;

    fn try_from(value: SealedHost) -> Result<Self, Self::Error> {
        match value {
            SealedHost::Plain(host) => Ok(host),
            SealedHost::Sealed(_) => Err(NetworkTypeError::Other("instance is sealed".into())),
        }
    }
}

impl std::fmt::Display for SealedHost {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SealedHost::Plain(h) => write!(f, "{h}"),
            SealedHost::Sealed(_) => write!(f, "<redacted host>"),
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

    /// Returns the number of hops this instance represents.
    /// This value is guaranteed to be between 0 and [`RoutingOptions::MAX_INTERMEDIATE_HOPS`].
    pub fn count_hops(&self) -> usize {
        match &self {
            RoutingOptions::IntermediatePath(v) => v.as_ref().len(),
            RoutingOptions::Hops(h) => (*h).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use std::net::SocketAddr;

    #[cfg(feature = "runtime-async-std")]
    #[async_std::test]
    async fn ip_or_host_must_resolve_dns_name() -> anyhow::Result<()> {
        match IpOrHost::Dns("localhost".to_string(), 1000)
            .resolve_async_std()
            .await?
            .first()
            .ok_or(anyhow!("must resolve"))?
        {
            SocketAddr::V4(addr) => assert_eq!(*addr, "127.0.0.1:1000".parse()?),
            SocketAddr::V6(addr) => assert_eq!(*addr, "::1:1000".parse()?),
        }
        Ok(())
    }

    #[cfg(feature = "runtime-async-std")]
    #[async_std::test]
    async fn ip_or_host_must_resolve_ip_address() -> anyhow::Result<()> {
        let actual = IpOrHost::Ip("127.0.0.1:1000".parse()?).resolve_async_std().await?;

        let actual = actual.first().ok_or(anyhow!("must resolve"))?;

        let expected: SocketAddr = "127.0.0.1:1000".parse()?;

        assert_eq!(*actual, expected);
        Ok(())
    }

    #[cfg(all(feature = "runtime-tokio", not(feature = "runtime-async-std")))]
    #[tokio::test]
    async fn ip_or_host_must_resolve_dns_name() -> anyhow::Result<()> {
        match IpOrHost::Dns("localhost".to_string(), 1000)
            .resolve_tokio()
            .await?
            .first()
            .ok_or(anyhow!("must resolve"))?
        {
            SocketAddr::V4(addr) => assert_eq!(*addr, "127.0.0.1:1000".parse()?),
            SocketAddr::V6(addr) => assert_eq!(*addr, "::1:1000".parse()?),
        }
        Ok(())
    }

    #[cfg(all(feature = "runtime-tokio", not(feature = "runtime-async-std")))]
    #[tokio::test]
    async fn ip_or_host_must_resolve_ip_address() -> anyhow::Result<()> {
        let actual = IpOrHost::Ip("127.0.0.1:1000".parse()?).resolve_tokio().await?;

        let actual = actual.first().ok_or(anyhow!("must resolve"))?;

        let expected: SocketAddr = "127.0.0.1:1000".parse()?;

        assert_eq!(*actual, expected);
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
