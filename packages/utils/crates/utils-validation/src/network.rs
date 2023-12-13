#[cfg(feature = "wasm")]
pub mod wasm {
    use proc_macro_regex::regex;

    regex!(is_dns_address_regex "^(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\\.)*[a-z0-9][a-z0-9-]{0,61}[a-z0-9]$");

    pub fn is_dns_address(s: &str) -> bool {
        is_dns_address_regex(s)
    }
}

#[cfg(not(feature = "wasm"))]
pub mod native {
    use std::net::ToSocketAddrs;

    pub fn is_dns_address(s: &str) -> bool {
        let host = format!("{}:80", s);
        host.to_socket_addrs().map_or(false, |i| i.into_iter().next().is_some())
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "wasm")]
    #[test]
    fn test_verify_valid_dns_addresses() {
        assert!(super::wasm::is_dns_address_regex("localhost"));
        assert!(super::wasm::is_dns_address_regex("hoprnet.org"));
        assert!(super::wasm::is_dns_address_regex("hub.hoprnet.org"));
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_verify_invalid_dns_addresses() {
        assert!(!super::wasm::is_dns_address_regex(".org"));
        assert!(!super::wasm::is_dns_address_regex("-hoprnet-.org"));
    }
}
