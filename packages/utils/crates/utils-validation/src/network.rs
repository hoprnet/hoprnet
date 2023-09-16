use proc_macro_regex::regex;

regex!(is_dns_address_regex "^(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\\.)*[a-z0-9][a-z0-9-]{0,61}[a-z0-9]$");

#[cfg(feature = "wasm")]
pub mod wasm {
    pub fn is_dns_address(s: &str) -> bool {
        crate::network::is_dns_address_regex(s)
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
    use super::*;

    #[test]
    fn test_verify_valid_dns_addresses() {
        assert!(is_dns_address_regex("localhost"));
        assert!(is_dns_address_regex("hoprnet.org"));
        assert!(is_dns_address_regex("hub.hoprnet.org"));
    }

    #[test]
    fn test_verify_invalid_dns_addresses() {
        assert!(!is_dns_address_regex(".org"));
        assert!(!is_dns_address_regex("-hoprnet-.org"));
    }
}
