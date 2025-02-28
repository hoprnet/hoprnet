use std::net::{Ipv4Addr, ToSocketAddrs};

pub use multiaddr::{Multiaddr, Protocol};

use crate::errors::{Result, TransportIdentityError};

/// Remove the `p2p/<PeerId>` component from a multiaddress
pub fn strip_p2p_protocol(ma: &Multiaddr) -> Multiaddr {
    Multiaddr::from_iter(ma.iter().filter(|v| !matches!(v, multiaddr::Protocol::P2p(_))))
}

/// Check whether the first multiaddress protocol component is a `dns*` component
pub fn is_dns(ma: &Multiaddr) -> bool {
    ma.iter()
        .next()
        .map(|proto| matches!(proto, multiaddr::Protocol::Dns(_)))
        .unwrap_or(false)
}

/// Check whether the first multiaddress protocol component represents a private address
pub fn is_private(ma: &Multiaddr) -> bool {
    ma.iter()
        .next()
        .map(|proto| match proto {
            multiaddr::Protocol::Ip4(ip) => ip.is_private(),
            multiaddr::Protocol::Dns(domain) => domain.to_ascii_lowercase() == "localhost",
            _ => false,
        })
        .unwrap_or(false)
}

/// Check whether the multiaddr protocol component is supported by this library
pub fn is_supported(ma: &Multiaddr) -> bool {
    ma.iter()
        .next()
        .map(|proto| {
            matches!(
                proto,
                multiaddr::Protocol::Ip4(_)
                    | multiaddr::Protocol::Ip6(_)
                    | multiaddr::Protocol::Dns(_)
                    | multiaddr::Protocol::Dns4(_)
                    | multiaddr::Protocol::Dns6(_)
            )
        })
        .unwrap_or(false)
}

/// Replaces the IPv4 and IPv6 from the network layer with a unspecified interface in any multiaddress.
pub fn replace_transport_with_unspecified(ma: &Multiaddr) -> Result<Multiaddr> {
    let mut out = Multiaddr::empty();

    for proto in ma.iter() {
        match proto {
            Protocol::Ip4(_) => out.push(std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED).into()),
            Protocol::Ip6(_) => out.push(std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED).into()),
            _ => out.push(proto),
        }
    }

    Ok(out)
}

/// Resolves the DNS parts of a multiaddress and replaces it with the resolved IP address.
pub fn resolve_dns_if_any(ma: &Multiaddr) -> Result<Multiaddr> {
    let mut out = Multiaddr::empty();

    for proto in ma.iter() {
        match proto {
            Protocol::Dns4(domain) => {
                let ip = format!("{domain}:443") // dummy port, irrevelant at this point
                    .to_socket_addrs()
                    .map_err(|e| TransportIdentityError::Multiaddress(e.to_string()))?
                    .filter(|sa| sa.is_ipv4())
                    .collect::<Vec<_>>()
                    .first()
                    .ok_or(TransportIdentityError::Multiaddress(format!(
                        "Failed to resolve {domain} to an IPv4 address. Does the DNS entry has an A record?"
                    )))?
                    .ip();

                out.push(ip.into())
            }
            Protocol::Dns6(domain) => {
                let ip = format!("{domain}:443") // dummy port, irrevelant at this point
                    .to_socket_addrs()
                    .map_err(|e| TransportIdentityError::Multiaddress(e.to_string()))?
                    .filter(|sa| sa.is_ipv6())
                    .collect::<Vec<_>>()
                    .first()
                    .ok_or(TransportIdentityError::Multiaddress(format!(
                        "Failed to resolve {domain} to an IPv6 address. Does the DNS entry has an AAAA record?"
                    )))?
                    .ip();

                out.push(ip.into())
            }
            _ => out.push(proto),
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_private_multiaddresses_are_shown_as_private() -> anyhow::Result<()> {
        assert!(!is_private(&Multiaddr::from_str("/ip4/33.42.112.22/udp/9090/quic")?));

        assert!(is_private(&Multiaddr::from_str("/ip4/192.168.1.23/udp/9090/quic")?));

        Ok(())
    }

    #[test]
    fn test_domain_dns4_multiaddresses_should_be_supported() -> anyhow::Result<()> {
        assert!(is_supported(&Multiaddr::from_str("/dns4/localhost/tcp/5543")?));

        Ok(())
    }

    #[test]
    fn multiaddrs_modification_specific_ipv4_transport_should_be_replacable_with_unspecified() -> anyhow::Result<()> {
        assert_eq!(
            replace_transport_with_unspecified(&Multiaddr::from_str("/ip4/33.42.112.22/udp/9090/quic")?),
            Ok(Multiaddr::from_str("/ip4/0.0.0.0/udp/9090/quic")?)
        );

        Ok(())
    }

    #[test]
    fn multiaddrs_modification_specific_ipv6_transport_should_be_replacable_with_unspecified() -> anyhow::Result<()> {
        assert_eq!(
            replace_transport_with_unspecified(&Multiaddr::from_str(
                "/ip6/82b0:a523:d8c0:1cba:365f:85f6:af3b:e369/udp/9090/quic"
            )?),
            Ok(Multiaddr::from_str("/ip6/0:0:0:0:0:0:0:0/udp/9090/quic")?)
        );

        Ok(())
    }
}
