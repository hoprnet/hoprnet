use std::{
    fmt::{Display, Formatter},
    net::SocketAddr,
    str::FromStr,
};

use hickory_resolver::name_server::ConnectionProvider;
use hopr_crypto_packet::{HoprSurb, prelude::HoprSenderId};
use hopr_crypto_random::Randomizable;
use hopr_internal_types::prelude::HoprPseudonym;
pub use hopr_path::ValidatedPath;
use hopr_primitive_types::{
    bounded::{BoundedSize, BoundedVec},
    prelude::Address,
};
use libp2p_identity::PeerId;

use crate::errors::NetworkTypeError;

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
    /// Uses `tokio` resolver.
    #[cfg(feature = "runtime-tokio")]
    pub async fn resolve_tokio(self) -> std::io::Result<Vec<SocketAddr>> {
        cfg_if::cfg_if! {
            if #[cfg(test)] {
                // This resolver setup is used in the tests to be executed in a sandbox environment
                // which prevents IO access to system-level files.
                let config = hickory_resolver::config::ResolverConfig::new();
                let options = hickory_resolver::config::ResolverOpts::default();
                let resolver = hickory_resolver::Resolver::builder_with_config(config, hickory_resolver::name_server::TokioConnectionProvider::default()).with_options(options).build();
            } else {
                let resolver = hickory_resolver::Resolver::builder_tokio()?.build();
            }
        };

        self.resolve(resolver).await
    }

    /// Tries to resolve the DNS name and returns all IP addresses found.
    /// If this enum is already an IP address and port, it will simply return it.
    pub async fn resolve<P: ConnectionProvider>(
        self,
        resolver: hickory_resolver::Resolver<P>,
    ) -> std::io::Result<Vec<SocketAddr>> {
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

/// Contains optionally encrypted [`IpOrHost`].
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
/// The length of the encrypted host is also obscured by the use of random padding before
/// encryption.
///
/// ### Example
/// ```rust
/// use hopr_crypto_types::prelude::{Keypair, OffchainKeypair};
/// use hopr_network_types::prelude::{IpOrHost, SealedHost};
/// use libp2p_identity::PeerId;
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
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::EnumTryAs)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SealedHost {
    /// Plain (not sealed) [`IpOrHost`]
    Plain(IpOrHost),
    /// Encrypted [`IpOrHost`]
    Sealed(Box<[u8]>),
}

impl SealedHost {
    const MAX_LEN_WITH_PADDING: usize = 50;
    /// Character that can be appended to the host to obscure its length.
    ///
    /// User can add as many of this character to the host, and it will be removed
    /// during unsealing.
    pub const PADDING_CHAR: char = '@';

    /// Seals the given [`IpOrHost`] using the Exit node's peer ID.
    pub fn seal(host: IpOrHost, peer_id: PeerId) -> crate::errors::Result<Self> {
        let mut host_str = host.to_string();

        // Add randomly long padding, so the length of the short hosts is obscured
        if host_str.len() < Self::MAX_LEN_WITH_PADDING {
            let pad_len = hopr_crypto_random::random_integer(0, (Self::MAX_LEN_WITH_PADDING as u64).into());
            for _ in 0..pad_len {
                host_str.push(Self::PADDING_CHAR);
            }
        }

        hopr_crypto_types::seal::seal_data(host_str.as_bytes(), peer_id)
            .map(Self::Sealed)
            .map_err(|e| NetworkTypeError::Other(e.to_string()))
    }

    /// Tries to unseal the sealed [`IpOrHost`] using the private key as Exit node.
    /// No-op, if the data is already unsealed.
    pub fn unseal(self, key: &hopr_crypto_types::keypairs::OffchainKeypair) -> crate::errors::Result<IpOrHost> {
        match self {
            SealedHost::Plain(host) => Ok(host),
            SealedHost::Sealed(enc) => hopr_crypto_types::seal::unseal_data(&enc, key)
                .map_err(|e| NetworkTypeError::Other(e.to_string()))
                .and_then(|data| {
                    String::from_utf8(data.into_vec())
                        .map_err(|e| NetworkTypeError::Other(e.to_string()))
                        .and_then(|s| IpOrHost::from_str(s.trim_end_matches(Self::PADDING_CHAR)))
                }),
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
            SealedHost::Sealed(_) => Err(NetworkTypeError::SealedTarget),
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
    IntermediatePath(BoundedVec<Address, { RoutingOptions::MAX_INTERMEDIATE_HOPS }>),
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

/// Allows finding saved SURBs based on different criteria.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SurbMatcher {
    /// Try to find a SURB that has the exact given [`HoprSenderId`].
    Exact(HoprSenderId),
    /// Find any SURB with the given pseudonym.
    Pseudonym(HoprPseudonym),
}

impl SurbMatcher {
    /// Get the pseudonym part of the match.
    pub fn pseudonym(&self) -> HoprPseudonym {
        match self {
            SurbMatcher::Exact(id) => id.pseudonym(),
            SurbMatcher::Pseudonym(p) => *p,
        }
    }
}

impl From<HoprPseudonym> for SurbMatcher {
    fn from(value: HoprPseudonym) -> Self {
        Self::Pseudonym(value)
    }
}

impl From<&HoprPseudonym> for SurbMatcher {
    fn from(pseudonym: &HoprPseudonym) -> Self {
        (*pseudonym).into()
    }
}

/// Routing information containing forward or return routing options.
///
/// Information in this object represents the minimum required basis
/// to generate forward paths and return paths.
///
/// See also [`RoutingOptions`].
#[derive(Debug, Clone, PartialEq, Eq, strum::EnumIs)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DestinationRouting {
    /// Forward routing using the destination address and path,
    /// with a possible return path.
    Forward {
        /// The destination address.
        destination: Address,
        /// Our pseudonym shown to the destination.
        ///
        /// If not given, it will be resolved as random.
        pseudonym: Option<HoprPseudonym>,
        /// The path to the destination.
        forward_options: RoutingOptions,
        /// Optional return path.
        return_options: Option<RoutingOptions>,
    },
    /// Return routing using a SURB with the given pseudonym.
    ///
    /// Will fail if no SURB for this pseudonym is found.
    Return(SurbMatcher),
}

impl DestinationRouting {
    /// Shortcut for routing that does not create any SURBs for a return path.
    pub fn forward_only(destination: Address, forward_options: RoutingOptions) -> Self {
        Self::Forward {
            destination,
            pseudonym: None,
            forward_options,
            return_options: None,
        }
    }
}

/// Contains the resolved routing information for the packet.
///
/// Instance of this object is typically constructed via some resolution of a
/// [`DestinationRouting`] instance.
///
/// It contains the actual forward and return paths for forward packets,
/// or an actual SURB for return (reply) packets.
#[derive(Clone, Debug, strum::EnumIs)]
pub enum ResolvedTransportRouting {
    /// Concrete routing information for a forward packet.
    Forward {
        /// Pseudonym of the sender.
        pseudonym: HoprPseudonym,
        /// Forward path.
        forward_path: ValidatedPath,
        /// Optional list of return paths.
        return_paths: Vec<ValidatedPath>,
    },
    /// Sender ID and the corresponding SURB.
    Return(HoprSenderId, HoprSurb),
}

impl ResolvedTransportRouting {
    /// Shortcut for routing that does not create any SURBs for a return path.
    pub fn forward_only(forward_path: ValidatedPath) -> Self {
        Self::Forward {
            pseudonym: HoprPseudonym::random(),
            forward_path,
            return_paths: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use anyhow::anyhow;
    use hopr_crypto_types::prelude::{Keypair, OffchainKeypair};

    use super::*;

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

    #[test]
    fn sealing_adds_padding_to_hide_length() -> anyhow::Result<()> {
        let peer_id: PeerId = OffchainKeypair::random().public().into();
        let last_len = SealedHost::seal("127.0.0.1:1234".parse()?, peer_id)?
            .try_as_sealed()
            .unwrap()
            .len();
        for _ in 0..20 {
            let current_len = SealedHost::seal("127.0.0.1:1234".parse()?, peer_id)?
                .try_as_sealed()
                .unwrap()
                .len();
            if current_len != last_len {
                return Ok(());
            }
        }

        anyhow::bail!("sealed length not randomized");
    }
}
