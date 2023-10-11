use std::str::FromStr;

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[cfg(not(feature = "wasm"))]
use utils_validation::network::native::is_dns_address;
#[cfg(feature = "wasm")]
use utils_validation::network::wasm::is_dns_address;

pub use core_network::{heartbeat::HeartbeatConfig, network::NetworkConfig};
pub use core_protocol::config::ProtocolConfig;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum HostType {
    IPv4(String),
    Domain(String),
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct HostConfig {
    #[validate(custom = "validate_host_address")]
    address: HostType,
    #[validate(range(min = 1u16))]
    pub port: u16,
}

// necessary only while the object must be wasm-compatible
impl HostConfig {
    pub fn address(&self) -> &HostType {
        &self.address
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
impl HostConfig {
    #[wasm_bindgen::prelude::wasm_bindgen]
    pub fn is_ipv4(&self) -> bool {
        match &self.address {
            HostType::IPv4(_) => true,
            _ => false,
        }
    }

    #[wasm_bindgen::prelude::wasm_bindgen]
    pub fn is_domain(&self) -> bool {
        match &self.address {
            HostType::Domain(_) => true,
            _ => false,
        }
    }

    #[wasm_bindgen::prelude::wasm_bindgen(js_name=address)]
    pub fn _address(&self) -> String {
        match &self.address {
            HostType::IPv4(s) => s.clone(),
            HostType::Domain(s) => s.clone(),
        }
    }
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            address: HostType::IPv4("127.0.0.1".to_owned()),
            port: 0,
        }
    }
}

impl FromStr for HostConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ip_or_dns, str_port) = match s.split_once(":") {
            None => return Err(format!("Invalid host, is not in the '<host>:<port>' format")),
            Some(split) => split,
        };

        let port = u16::from_str_radix(str_port, 10).map_err(|e| e.to_string())?;

        if validator::validate_ip_v4(ip_or_dns) {
            Ok(Self {
                address: HostType::IPv4(ip_or_dns.to_owned()),
                port,
            })
        } else if is_dns_address(ip_or_dns) {
            Ok(Self {
                address: HostType::Domain(ip_or_dns.to_owned()),
                port,
            })
        } else {
            Err(format!("Not a valid IPv4 or domain host"))
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
    if is_dns_address(s) {
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

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[derive(Debug, Default, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct TransportConfig {
    /// When true, assume that the node is running in an isolated network and does
    /// not need any connection to nodes outside of the subnet
    pub announce_local_addresses: bool,
    /// When true, assume a testnet with multiple nodes running on the same machine
    /// or in the same private IPv4 network
    pub prefer_local_addresses: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(validate_dns_address("hoprnet.org").is_ok());
        assert!(validate_dns_address("hub.hoprnet.org").is_ok());
    }

    #[test]
    fn test_verify_invalid_dns_addresses() {
        assert!(validate_dns_address("org").is_err());
        assert!(validate_dns_address(".org").is_err());
        assert!(validate_dns_address("-hoprnet-.org").is_err());
        assert!(validate_dns_address("unknown.sub.sub.hoprnet.org").is_err());
    }
}
