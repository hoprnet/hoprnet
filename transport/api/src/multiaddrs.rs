use multiaddr::Multiaddr;

/// Remove the `p2p/<PeerId>` component from a multiaddress
pub fn decapsulate_p2p_protocol(ma: &Multiaddr) -> Multiaddr {
    Multiaddr::from_iter(ma.iter().filter(|v| !matches!(v, multiaddr::Protocol::P2p(_))))
}

/// Check whether the first multiaddress protocol component is a `dns*` component
pub(crate) fn is_dns(ma: &Multiaddr) -> bool {
    ma.iter()
        .next()
        .map(|proto| matches!(proto, multiaddr::Protocol::Dns(_)))
        .unwrap_or(false)
}

/// Check whether the first multiaddress protocol component represents a private address
pub(crate) fn is_private(ma: &Multiaddr) -> bool {
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
pub(crate) fn is_supported(ma: &Multiaddr) -> bool {
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

    use super::*;

    #[test]
    fn test_private_multiaddresses_are_shown_as_private() {
        assert!(!is_private(
            &Multiaddr::from_str("/ip4/33.42.112.22/udp/9090/quic").unwrap()
        ));

        assert!(is_private(
            &Multiaddr::from_str("/ip4/192.168.1.23/udp/9090/quic").unwrap()
        ));
    }

    #[test]
    fn test_domain_dns4_multiaddresses_should_be_supported() {
        assert!(is_supported(&Multiaddr::from_str("/dns4/localhost/tcp/5543").unwrap()));
    }
}
