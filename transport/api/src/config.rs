use std::num::ParseIntError;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

pub use core_network::{heartbeat::HeartbeatConfig, network::NetworkConfig};
pub use core_protocol::config::ProtocolConfig;

use std::net::ToSocketAddrs;

use proc_macro_regex::regex;

regex!(is_dns_address_regex "^(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\\.)*[a-z0-9][a-z0-9-]{0,61}[a-z0-9]$");

#[inline]
pub fn looks_like_domain(s: &str) -> bool {
    is_dns_address_regex(s)
}

pub fn is_reachable_domain(host: &str) -> bool {
    host.to_socket_addrs().map_or(false, |i| i.into_iter().next().is_some())
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum HostType {
    IPv4(String),
    Domain(String),
}

impl Default for HostType {
    fn default() -> Self {
        HostType::IPv4("127.0.0.1".to_owned())
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct HostConfig {
    #[validate(custom = "validate_host_address")]
    #[serde(default)]
    pub address: HostType,
    #[validate(range(min = 1u16))]
    #[serde(default)]
    pub port: u16,
}

impl FromStr for HostConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ip_or_dns, str_port) = match s.split_once(':') {
            None => return Err("Invalid host, is not in the '<host>:<port>' format".into()),
            Some(split) => split,
        };

        let port = str_port.parse().map_err(|e: ParseIntError| e.to_string())?;

        if validator::validate_ip_v4(ip_or_dns) {
            Ok(Self {
                address: HostType::IPv4(ip_or_dns.to_owned()),
                port,
            })
        } else if looks_like_domain(ip_or_dns) {
            Ok(Self {
                address: HostType::Domain(ip_or_dns.to_owned()),
                port,
            })
        } else {
            Err("Not a valid IPv4 or domain host".into())
        }
    }
}

impl ToString for HostConfig {
    fn to_string(&self) -> String {
        format!("{:?}:{}", self.address, self.port)
    }
}

fn validate_ipv4_address(s: &str) -> Result<(), ValidationError> {
    if validator::validate_ip(s) {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid IPv4 address provided"))
    }
}

fn validate_dns_address(s: &str) -> Result<(), ValidationError> {
    if looks_like_domain(s) || is_reachable_domain(s) {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid DNS address provided"))
    }
}

fn validate_host_address(host: &HostType) -> Result<(), ValidationError> {
    match host {
        HostType::IPv4(ip4) => validate_ipv4_address(ip4),
        HostType::Domain(domain) => validate_dns_address(domain),
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct TransportConfig {
    /// When true, assume that the node is running in an isolated network and does
    /// not need any connection to nodes outside of the subnet
    #[serde(default)]
    pub announce_local_addresses: bool,
    /// When true, assume a testnet with multiple nodes running on the same machine
    /// or in the same private IPv4 network
    #[serde(default)]
    pub prefer_local_addresses: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_domains_for_looks_like_a_domain() {
        assert!(looks_like_domain("localhost"));
        assert!(looks_like_domain("hoprnet.org"));
        assert!(looks_like_domain("hub.hoprnet.org"));
    }

    #[test]
    fn test_valid_domains_for_does_not_look_like_a_domain() {
        assert!(!looks_like_domain(".org"));
        assert!(!looks_like_domain("-hoprnet-.org"));
    }

    #[test]
    fn test_valid_domains_should_be_reachable() {
        assert!(!is_reachable_domain("google.com"));
    }

    #[test]
    fn test_verify_valid_ip4_addresses() {
        assert!(validate_ipv4_address("1.1.1.1").is_ok());
        assert!(validate_ipv4_address("1.255.1.1").is_ok());
        assert!(validate_ipv4_address("187.1.1.255").is_ok());
        assert!(validate_ipv4_address("127.0.0.1").is_ok());
        assert!(validate_ipv4_address("0.0.0.0").is_ok());
    }

    #[test]
    fn test_verify_invalid_ip4_addresses() {
        assert!(validate_ipv4_address("1.256.1.1").is_err());
        assert!(validate_ipv4_address("-1.1.1.255").is_err());
        assert!(validate_ipv4_address("127.0.0.256").is_err());
        assert!(validate_ipv4_address("1").is_err());
        assert!(validate_ipv4_address("1.1").is_err());
        assert!(validate_ipv4_address("1.1.1").is_err());
        assert!(validate_ipv4_address("1.1.1.1.1").is_err());
    }

    #[test]
    fn test_verify_valid_dns_addresses() {
        assert!(validate_dns_address("localhost").is_ok());
        assert!(validate_dns_address("google.com").is_ok());
        assert!(validate_dns_address("hub.hoprnet.org").is_ok());
    }

    #[test]
    fn test_verify_invalid_dns_addresses() {
        assert!(validate_dns_address("-hoprnet-.org").is_err());
    }
}
