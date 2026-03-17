pub use multiaddr::{Multiaddr, Protocol};

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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::with_p2p(
        "/ip4/1.2.3.4/tcp/9000/p2p/12D3KooWD3bkCaU1vQxKqFsfEUVFyGNqCbGsBGzqmrQasRL6qMpo",
        "/ip4/1.2.3.4/tcp/9000"
    )]
    #[case::without_p2p("/ip4/1.2.3.4/tcp/9000", "/ip4/1.2.3.4/tcp/9000")]
    fn strip_p2p_protocol_should_remove_p2p_when_present(
        #[case] input: &str,
        #[case] expected: &str,
    ) -> anyhow::Result<()> {
        let addr = Multiaddr::from_str(input)?;
        let stripped = strip_p2p_protocol(&addr);
        assert_eq!(stripped, Multiaddr::from_str(expected)?);
        Ok(())
    }

    #[rstest]
    #[case::dns_is_true("/dns/example.com/tcp/443", true)]
    #[case::dns4_is_false("/dns4/example.com/tcp/443", false)]
    #[case::ip4_is_false("/ip4/1.2.3.4/tcp/9000", false)]
    fn is_dns_should_identify_dns_protocols(#[case] input: &str, #[case] expected: bool) -> anyhow::Result<()> {
        let addr = Multiaddr::from_str(input)?;
        assert_eq!(is_dns(&addr), expected);
        Ok(())
    }

    #[rstest]
    #[case::private_ip4("/ip4/192.168.1.23/udp/9090/quic", true)]
    #[case::public_ip4("/ip4/33.42.112.22/udp/9090/quic", false)]
    #[case::dns_localhost("/dns/localhost/tcp/443", true)]
    #[case::dns4_external("/dns4/example.com/tcp/443", false)]
    fn is_private_should_detect_private_addresses(#[case] input: &str, #[case] expected: bool) -> anyhow::Result<()> {
        let addr = Multiaddr::from_str(input)?;
        assert_eq!(is_private(&addr), expected);
        Ok(())
    }

    #[rstest]
    #[case::ip4("/ip4/1.2.3.4/tcp/9000", true)]
    #[case::ip6("/ip6/::1/tcp/9000", true)]
    #[case::dns("/dns/example.com/tcp/443", true)]
    #[case::dns4("/dns4/localhost/tcp/5543", true)]
    #[case::dns6("/dns6/example.com/tcp/443", true)]
    #[case::p2p_unsupported("/p2p/12D3KooWD3bkCaU1vQxKqFsfEUVFyGNqCbGsBGzqmrQasRL6qMpo", false)]
    fn is_supported_should_accept_known_protocols(#[case] input: &str, #[case] expected: bool) -> anyhow::Result<()> {
        let addr = Multiaddr::from_str(input)?;
        assert_eq!(is_supported(&addr), expected);
        Ok(())
    }
}
