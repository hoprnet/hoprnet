//! Subnet bucket derivation from peer multiaddresses.

use std::collections::HashSet;

use hopr_api::Multiaddr;
use multiaddr::Protocol;

/// Network-locality bucket derived from a peer's advertised multiaddresses.
/// Used as the locality dimension in the anonymity bucket grid.
///
/// Precedence (highest wins, across all multiaddrs in the set):
/// - **V4**: /24 prefix. Multi-homed → lex-smallest prefix. Mixed v4+v6 → V4.
/// - **V6**: /48 prefix. Applied only when no v4 address is present.
/// - **Dns**: lowercased hostname from `Dns`, `Dns4`, or `Dns6` protocols.
/// - **Unknown**: no IP or DNS protocol found in any of the peer's multiaddresses.
///
/// `Unknown` is excluded from the k-diversity floor but appears in the entropy
/// denominator so the floor does not block opens to unobserved peers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubnetBucket {
    V4Prefix([u8; 3]),
    V6Prefix([u8; 6]),
    Dns(String),
    Unknown,
}

impl SubnetBucket {
    /// Derives the subnet bucket from a peer's multiaddress set.
    /// Returns `Unknown` for an empty set.
    pub fn from_multiaddrs(addrs: &HashSet<Multiaddr>) -> Self {
        let mut v4s: Vec<[u8; 3]> = Vec::new();
        let mut v6s: Vec<[u8; 6]> = Vec::new();
        let mut dns: Option<String> = None;

        for ma in addrs {
            for proto in ma.iter() {
                match proto {
                    Protocol::Ip4(addr) => {
                        let o = addr.octets();
                        v4s.push([o[0], o[1], o[2]]);
                    }
                    Protocol::Ip6(addr) => {
                        let o = addr.octets();
                        v6s.push([o[0], o[1], o[2], o[3], o[4], o[5]]);
                    }
                    Protocol::Dns(h) | Protocol::Dns4(h) | Protocol::Dns6(h) if dns.is_none() => {
                        dns = Some(h.to_lowercase());
                    }
                    _ => {}
                }
            }
        }

        if !v4s.is_empty() {
            v4s.sort_unstable();
            Self::V4Prefix(v4s[0])
        } else if !v6s.is_empty() {
            v6s.sort_unstable();
            Self::V6Prefix(v6s[0])
        } else if let Some(d) = dns {
            Self::Dns(d)
        } else {
            Self::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ma(s: &str) -> Multiaddr {
        s.parse().expect("valid multiaddr")
    }

    #[test]
    fn v4_single_addr() {
        let addrs = [ma("/ip4/1.2.3.4/tcp/9090")].into();
        assert_eq!(SubnetBucket::from_multiaddrs(&addrs), SubnetBucket::V4Prefix([1, 2, 3]));
    }

    #[test]
    fn v4_multi_homed_picks_lex_smallest() {
        let addrs = [ma("/ip4/192.168.1.1/tcp/9090"), ma("/ip4/10.0.0.1/tcp/9090")].into();
        assert_eq!(
            SubnetBucket::from_multiaddrs(&addrs),
            SubnetBucket::V4Prefix([10, 0, 0])
        );
    }

    #[test]
    fn v6_single_addr() {
        use std::net::Ipv6Addr;
        let addrs = [ma("/ip6/2001:db8::1/tcp/9090")].into();
        let full: Ipv6Addr = "2001:db8::1".parse().unwrap();
        let o = full.octets();
        let expected = SubnetBucket::V6Prefix([o[0], o[1], o[2], o[3], o[4], o[5]]);
        assert_eq!(SubnetBucket::from_multiaddrs(&addrs), expected);
    }

    #[test]
    fn mixed_v4_and_v6_collapses_to_v4() {
        let addrs = [ma("/ip4/192.168.1.1/tcp/9090"), ma("/ip6/2001:db8::1/tcp/9090")].into();
        assert_eq!(
            SubnetBucket::from_multiaddrs(&addrs),
            SubnetBucket::V4Prefix([192, 168, 1])
        );
    }

    #[test]
    fn dns4_lowercased() {
        let addrs = [ma("/dns4/Node.Example.COM/tcp/9090")].into();
        assert_eq!(
            SubnetBucket::from_multiaddrs(&addrs),
            SubnetBucket::Dns("node.example.com".to_string())
        );
    }

    #[test]
    fn dns_generic() {
        let addrs = [ma("/dns/some.HOST.net/tcp/9090")].into();
        assert_eq!(
            SubnetBucket::from_multiaddrs(&addrs),
            SubnetBucket::Dns("some.host.net".to_string())
        );
    }

    #[test]
    fn empty_multiaddrs_returns_unknown() {
        assert_eq!(SubnetBucket::from_multiaddrs(&HashSet::new()), SubnetBucket::Unknown);
    }
}
