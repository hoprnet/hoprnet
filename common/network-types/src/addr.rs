use std::{net::IpAddr, sync::atomic::AtomicBool};

use lazy_static::lazy_static;
use multiaddr::Multiaddr;

lazy_static! {
    pub(crate) static ref ALLOW_PRIVATE_ADDRESSES: AtomicBool =
        std::env::var("HOPR_INTERNAL_TRANSPORT_ACCEPT_PRIVATE_NETWORK_IP_ADDRESSES")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or_default()
            .into();
}

/// Check if an IP address is a public/routable one.
///
/// If the `HOPR_INTERNAL_TRANSPORT_ACCEPT_PRIVATE_NETWORK_IP_ADDRESSES` env variable
/// is set to `true`, this function always returns true to allow private addresses for
/// local development and testing.
pub fn is_public(ip_addr: IpAddr) -> bool {
    match ip_addr {
        IpAddr::V4(ip) => !ip.is_unspecified() && !ip.is_private() && !ip.is_loopback() && !ip.is_link_local(),
        IpAddr::V6(ip) => {
            !ip.is_unspecified() && !ip.is_loopback() && !ip.is_unicast_link_local() && !ip.is_unique_local()
        }
    }
}

#[inline]
fn is_allowed_for_transport(ip_addr: IpAddr) -> bool {
    ALLOW_PRIVATE_ADDRESSES.load(std::sync::atomic::Ordering::SeqCst) || is_public(ip_addr)
}

/// Check if a multiaddress contains a public/routable IP address.
///
/// If the `HOPR_INTERNAL_TRANSPORT_ACCEPT_PRIVATE_NETWORK_IP_ADDRESSES` env variable
/// is set to `true`, this function always returns true to allow private addresses for
/// local development and testing.
pub fn is_public_address(addr: &Multiaddr) -> bool {
    addr.iter().all(|protocol| match protocol {
        multiaddr::Protocol::Ip4(ip) => is_allowed_for_transport(ip.into()),
        multiaddr::Protocol::Ip6(ip) => is_allowed_for_transport(ip.into()),
        _ => true,
    })
}

#[cfg(test)]
#[inline]
fn override_allow_private_addresses(value: bool) {
    ALLOW_PRIVATE_ADDRESSES.store(value, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(test)]
#[inline]
fn get_allow_private_addresses() -> bool {
    ALLOW_PRIVATE_ADDRESSES.load(std::sync::atomic::Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    #[serial_test::serial] // must be serial to avoid random race conditions on atomic bool on parallel execution
    fn test_is_public_address_ipv4() -> anyhow::Result<()> {
        override_allow_private_addresses(false);
        assert!(!get_allow_private_addresses());

        // IPv4 public addresses - should return true
        assert!(is_public_address(&Multiaddr::from_str("/ip4/8.8.8.8")?));
        assert!(is_public_address(&Multiaddr::from_str("/ip4/1.1.1.1")?));
        assert!(is_public_address(&Multiaddr::from_str("/ip4/104.16.0.0")?));

        // IPv4 private addresses - should return false
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/192.168.0.1")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/192.168.1.254")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/10.0.0.1")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/10.1.0.1")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/10.255.255.255")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/172.16.0.0")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/172.31.255.255")?));

        // IPv4 loopback - should return false
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/127.0.0.1")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/127.255.255.255")?));

        // IPv4 link-local - should return false
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/169.254.1.1")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/169.254.254.254")?));

        // IPv4 unspecified - should return false
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/0.0.0.0")?));

        Ok(())
    }

    #[test]
    #[serial_test::serial] // must be serial to avoid random race conditions on atomic bool on parallel execution
    fn test_is_public_address_ipv6() -> anyhow::Result<()> {
        override_allow_private_addresses(false);
        assert!(!get_allow_private_addresses());

        // IPv6 public addresses - should return true
        assert!(is_public_address(&Multiaddr::from_str("/ip6/2001:4860:4860::8888")?));
        assert!(is_public_address(&Multiaddr::from_str("/ip6/2606:4700:4700::1111")?));
        assert!(is_public_address(&Multiaddr::from_str(
            "/ip6/2a00:1450:4001:830::200e"
        )?));

        // IPv6 loopback - should return false
        assert!(!is_public_address(&Multiaddr::from_str("/ip6/::1")?));

        // IPv6 unique-local - should return false
        assert!(!is_public_address(&Multiaddr::from_str("/ip6/fc00::1")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip6/fd00::1")?));
        assert!(!is_public_address(&Multiaddr::from_str(
            "/ip6/fdff:ffff:ffff:ffff:ffff:ffff:ffff:ffff"
        )?));

        // IPv6 link-local - should return false
        assert!(!is_public_address(&Multiaddr::from_str("/ip6/fe80::1")?));
        assert!(!is_public_address(&Multiaddr::from_str(
            "/ip6/fe80::dead:beef:cafe:babe"
        )?));
        assert!(!is_public_address(&Multiaddr::from_str(
            "/ip6/febf:ffff:ffff:ffff:ffff:ffff:ffff:ffff"
        )?));

        // IPv6 unspecified - should return false
        assert!(!is_public_address(&Multiaddr::from_str("/ip6/::")?));

        Ok(())
    }

    #[test]
    #[serial_test::serial] // must be serial to avoid random race conditions on atomic bool on parallel execution
    fn test_is_public_address_with_protocols() -> anyhow::Result<()> {
        override_allow_private_addresses(false);
        assert!(!get_allow_private_addresses());

        // Public addresses with additional protocols - should return true
        assert!(is_public_address(&Multiaddr::from_str("/ip4/8.8.8.8/tcp/4001")?));
        assert!(is_public_address(&Multiaddr::from_str("/ip4/1.1.1.1/udp/30303")?));
        assert!(is_public_address(&Multiaddr::from_str(
            "/ip4/8.8.8.8/tcp/4001/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp"
        )?));
        assert!(is_public_address(&Multiaddr::from_str(
            "/ip6/2001:4860:4860::8888/tcp/4001"
        )?));
        assert!(is_public_address(&Multiaddr::from_str(
            "/ip6/2001:4860:4860::8888/udp/30303"
        )?));
        assert!(is_public_address(&Multiaddr::from_str(
            "/ip6/2606:4700:4700::1111/tcp/443/wss"
        )?));

        // Private/special addresses with additional protocols - should return false
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/192.168.0.1/tcp/4001")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/127.0.0.1/tcp/8080")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/10.0.0.1/tcp/8080")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/10.1.0.1/tcp/8080")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/169.254.1.1/udp/5060")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip4/0.0.0.0/tcp/3000")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip6/::1/tcp/4001")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip6/fe80::1/udp/30303")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip6/fc00::1/tcp/443")?));
        assert!(!is_public_address(&Multiaddr::from_str("/ip6/::/tcp/8080")?));

        Ok(())
    }

    #[test]
    #[serial_test::serial] // must be serial to avoid random race conditions on atomic bool on parallel execution
    fn test_is_public_address_mixed_protocols() -> anyhow::Result<()> {
        override_allow_private_addresses(false);
        assert!(!get_allow_private_addresses());

        // Test with DNS and other protocols (no IP) - should return true (non-IP protocols default to true)
        assert!(is_public_address(&Multiaddr::from_str("/dns/example.com")?));
        assert!(is_public_address(&Multiaddr::from_str("/dns4/example.com/tcp/443")?));
        assert!(is_public_address(&Multiaddr::from_str("/dns6/example.com/tcp/443")?));

        Ok(())
    }

    #[test]
    #[serial_test::serial] // must be serial to avoid random race conditions on atomic bool on parallel execution
    fn test_local_testing_allows_all_addresses() -> anyhow::Result<()> {
        override_allow_private_addresses(true);
        assert!(get_allow_private_addresses());

        // When local-testing feature is enabled, all addresses should be considered "public"
        assert!(is_public_address(&Multiaddr::from_str("/ip4/127.0.0.1")?));
        assert!(is_public_address(&Multiaddr::from_str("/ip4/192.168.1.1")?));
        assert!(is_public_address(&Multiaddr::from_str("/ip4/10.0.0.1")?));
        assert!(is_public_address(&Multiaddr::from_str("/ip4/172.16.0.1")?));
        assert!(is_public_address(&Multiaddr::from_str("/ip6/::1")?));
        assert!(is_public_address(&Multiaddr::from_str("/ip6/fe80::1")?));

        Ok(())
    }
}
