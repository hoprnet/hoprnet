use multiaddr::{Multiaddr, Protocol};

/// Remove the `p2p/<PeerId>` component from a multiaddress
pub fn strip_p2p_protocol(ma: &Multiaddr) -> Multiaddr {
    Multiaddr::from_iter(ma.iter().filter(|v| !matches!(v, Protocol::P2p(_))))
}

/// Check whether the first multiaddress protocol component is a `dns*` component
pub fn is_dns(ma: &Multiaddr) -> bool {
    ma.iter()
        .next()
        .map(|proto| matches!(proto, Protocol::Dns(_) | Protocol::Dns4(_) | Protocol::Dns6(_)))
        .unwrap_or(false)
}

/// Check whether the multiaddr protocol component is supported by this library
pub fn is_supported(ma: &Multiaddr) -> bool {
    ma.iter()
        .next()
        .map(|proto| {
            matches!(
                proto,
                Protocol::Ip4(_) | Protocol::Ip6(_) | Protocol::Dns(_) | Protocol::Dns4(_) | Protocol::Dns6(_)
            )
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use anyhow::Context;
    use rstest::rstest;

    use super::*;

    /// Check whether the first multiaddress protocol component represents a private address
    fn is_private(ma: &Multiaddr) -> bool {
        ma.iter()
            .next()
            .map(|proto| match proto {
                Protocol::Ip4(ip) => ip.is_private(),
                Protocol::Dns(domain) => domain.to_ascii_lowercase() == "localhost",
                _ => false,
            })
            .unwrap_or(false)
    }

    #[test]
    fn private_multiaddresses_are_shown_as_private() -> anyhow::Result<()> {
        assert!(!is_private(&Multiaddr::from_str("/ip4/33.42.112.22/udp/9090/quic")?));

        assert!(is_private(&Multiaddr::from_str("/ip4/192.168.1.23/udp/9090/quic")?));

        Ok(())
    }

    // -- strip_p2p_protocol ---------------------------------------------------

    #[test]
    fn strip_p2p_protocol_should_remove_trailing_p2p_component() -> anyhow::Result<()> {
        let ma =
            Multiaddr::from_str("/ip4/127.0.0.1/tcp/9091/p2p/16Uiu2HAmA5h1q1jtAPMaJch5CRnSBMjGNq22mfMDHSgMPC27cW1B")
                .context("parsing multiaddr with p2p")?;

        let stripped = strip_p2p_protocol(&ma);
        let expected = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9091").context("parsing expected")?;

        assert_eq!(stripped, expected);

        Ok(())
    }

    #[test]
    fn strip_p2p_protocol_should_be_noop_when_no_p2p_present() -> anyhow::Result<()> {
        let ma = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9091").context("parsing multiaddr")?;

        let stripped = strip_p2p_protocol(&ma);

        assert_eq!(stripped, ma);

        Ok(())
    }

    #[test]
    fn strip_p2p_protocol_should_return_empty_for_p2p_only_address() -> anyhow::Result<()> {
        let ma = Multiaddr::from_str("/p2p/16Uiu2HAmA5h1q1jtAPMaJch5CRnSBMjGNq22mfMDHSgMPC27cW1B")
            .context("parsing p2p-only multiaddr")?;

        let stripped = strip_p2p_protocol(&ma);

        assert!(stripped.is_empty());

        Ok(())
    }

    // -- is_dns ---------------------------------------------------------------

    #[rstest]
    #[case("/dns/example.com/tcp/443", true)]
    #[case("/dns4/example.com/tcp/443", true)]
    #[case("/dns6/example.com/tcp/443", true)]
    #[case("/ip4/127.0.0.1/tcp/9091", false)]
    #[case("/ip6/::1/tcp/9091", false)]
    fn is_dns_should_detect_dns_variants(#[case] addr: &str, #[case] expected: bool) -> anyhow::Result<()> {
        let ma = Multiaddr::from_str(addr).context("parsing multiaddr")?;

        assert_eq!(is_dns(&ma), expected, "is_dns mismatch for {addr}");

        Ok(())
    }

    #[test]
    fn is_dns_should_return_false_for_empty_multiaddr() {
        assert!(!is_dns(&Multiaddr::empty()));
    }

    // -- is_supported ---------------------------------------------------------

    #[rstest]
    #[case("/ip4/127.0.0.1/tcp/9091", true)]
    #[case("/ip6/::1/tcp/9091", true)]
    #[case("/dns/example.com/tcp/443", true)]
    #[case("/dns4/localhost/tcp/5543", true)]
    #[case("/dns6/localhost/tcp/5543", true)]
    #[case("/memory/1234", false)]
    fn is_supported_should_identify_supported_protocols(
        #[case] addr: &str,
        #[case] expected: bool,
    ) -> anyhow::Result<()> {
        let ma = Multiaddr::from_str(addr).context("parsing multiaddr")?;

        assert_eq!(is_supported(&ma), expected, "is_supported mismatch for {addr}");

        Ok(())
    }

    #[test]
    fn is_supported_should_return_false_for_empty_multiaddr() {
        assert!(!is_supported(&Multiaddr::empty()));
    }
}
