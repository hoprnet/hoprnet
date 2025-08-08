use std::{
    fmt::{Display, Formatter},
    net::ToSocketAddrs,
    num::ParseIntError,
    str::FromStr,
    time::Duration,
};

use hopr_transport_identity::Multiaddr;
pub use hopr_transport_network::config::NetworkConfig;
pub use hopr_transport_probe::config::ProbeConfig;
pub use hopr_transport_protocol::config::ProtocolConfig;
use hopr_transport_session::{MIN_BALANCER_SAMPLING_INTERVAL, MIN_SURB_BUFFER_DURATION};
use proc_macro_regex::regex;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use validator::{Validate, ValidationError};

use crate::errors::HoprTransportError;

pub struct HoprTransportConfig {
    pub transport: TransportConfig,
    pub network: NetworkConfig,
    pub protocol: ProtocolConfig,
    pub probe: ProbeConfig,
    pub session: SessionGlobalConfig,
}

regex!(is_dns_address_regex "^(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\\.)*[a-z0-9][a-z0-9-]{0,61}[a-z0-9]$");

/// Check whether the string looks like a valid domain.
#[inline]
pub fn looks_like_domain(s: &str) -> bool {
    is_dns_address_regex(s)
}

/// Check whether the string is an actual reachable domain.
pub fn is_reachable_domain(host: &str) -> bool {
    host.to_socket_addrs().is_ok_and(|i| i.into_iter().next().is_some())
}

/// Enumeration of possible host types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HostType {
    /// IPv4 based host
    IPv4(String),
    /// DNS based host
    Domain(String),
}

impl Default for HostType {
    fn default() -> Self {
        HostType::IPv4("127.0.0.1".to_owned())
    }
}

/// Configuration of the listening host.
///
/// This is used for the P2P and REST API listeners.
///
/// Intentionally has no default because it depends on the use case.
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HostConfig {
    /// Host on which to listen
    #[serde(default)] // must be defaulted to be mergeable from CLI args
    pub address: HostType,
    /// Listening TCP or UDP port (mandatory).
    #[validate(range(min = 1u16))]
    #[serde(default)] // must be defaulted to be mergeable from CLI args
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

        if validator::ValidateIp::validate_ipv4(&ip_or_dns) {
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

impl Display for HostConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}:{}", self.address, self.port)
    }
}

fn default_multiaddr_transport(port: u16) -> String {
    cfg_if::cfg_if! {
        if #[cfg(feature = "transport-quic")] {
            // In case we run on a Dappnode-like device, presumably behind NAT, we fall back to TCP
            // to circumvent issues with QUIC in such environments. To make this work reliably,
            // we would need proper NAT traversal support.
            let on_dappnode = std::env::var("DAPPNODE")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false);

            // Using HOPRD_NAT a user can overwrite the default behaviour even on a Dappnode-like device
            let uses_nat = std::env::var("HOPRD_NAT")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(on_dappnode);

            if uses_nat {
                format!("tcp/{port}")
            } else {
                format!("udp/{port}/quic-v1")
            }
        } else {
            format!("tcp/{port}")
        }
    }
}

impl TryFrom<&HostConfig> for Multiaddr {
    type Error = HoprTransportError;

    fn try_from(value: &HostConfig) -> Result<Self, Self::Error> {
        match &value.address {
            HostType::IPv4(ip) => Multiaddr::from_str(
                format!("/ip4/{}/{}", ip.as_str(), default_multiaddr_transport(value.port)).as_str(),
            )
            .map_err(|e| HoprTransportError::Api(e.to_string())),
            HostType::Domain(domain) => Multiaddr::from_str(
                format!("/dns4/{}/{}", domain.as_str(), default_multiaddr_transport(value.port)).as_str(),
            )
            .map_err(|e| HoprTransportError::Api(e.to_string())),
        }
    }
}

fn validate_ipv4_address(s: &str) -> Result<(), ValidationError> {
    if validator::ValidateIp::validate_ipv4(&s) {
        let ipv4 = std::net::Ipv4Addr::from_str(s)
            .map_err(|_| ValidationError::new("Failed to deserialize the string into an ipv4 address"))?;

        if ipv4.is_private() || ipv4.is_multicast() || ipv4.is_unspecified() {
            return Err(ValidationError::new(
                "IPv4 cannot be private, multicast or unspecified (0.0.0.0)",
            ))?;
        }
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

/// Validates the HostConfig to be used as an external host
pub fn validate_external_host(host: &HostConfig) -> Result<(), ValidationError> {
    match &host.address {
        HostType::IPv4(ip4) => validate_ipv4_address(ip4),
        HostType::Domain(domain) => validate_dns_address(domain),
    }
}

/// Configuration of the physical transport mechanism.
#[derive(Debug, Default, Serialize, Deserialize, Validate, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TransportConfig {
    /// When true, assume that the node is running in an isolated network and does
    /// not need any connection to nodes outside the subnet
    #[serde(default)]
    pub announce_local_addresses: bool,
    /// When true, assume a testnet with multiple nodes running on the same machine
    /// or in the same private IPv4 network
    #[serde(default)]
    pub prefer_local_addresses: bool,
}

const DEFAULT_SESSION_MAX_SESSIONS: u32 = 2048;

const DEFAULT_SESSION_IDLE_TIMEOUT: Duration = Duration::from_secs(180);

const SESSION_IDLE_MIN_TIMEOUT: Duration = Duration::from_secs(60);

const DEFAULT_SESSION_ESTABLISH_RETRY_DELAY: Duration = Duration::from_secs(2);

const DEFAULT_SESSION_ESTABLISH_MAX_RETRIES: u32 = 3;

const DEFAULT_SESSION_BALANCER_SAMPLING: Duration = Duration::from_millis(500);

const DEFAULT_SESSION_BALANCER_BUFFER_DURATION: Duration = Duration::from_millis(5000);

fn default_session_max_sessions() -> u32 {
    DEFAULT_SESSION_MAX_SESSIONS
}

fn default_session_balancer_buffer_duration() -> std::time::Duration {
    DEFAULT_SESSION_BALANCER_BUFFER_DURATION
}

fn default_session_establish_max_retries() -> u32 {
    DEFAULT_SESSION_ESTABLISH_MAX_RETRIES
}

fn default_session_idle_timeout() -> std::time::Duration {
    DEFAULT_SESSION_IDLE_TIMEOUT
}

fn default_session_establish_retry_delay() -> std::time::Duration {
    DEFAULT_SESSION_ESTABLISH_RETRY_DELAY
}

fn default_session_balancer_sampling() -> std::time::Duration {
    DEFAULT_SESSION_BALANCER_SAMPLING
}

fn validate_session_idle_timeout(value: &std::time::Duration) -> Result<(), ValidationError> {
    if SESSION_IDLE_MIN_TIMEOUT <= *value {
        Ok(())
    } else {
        Err(ValidationError::new("session idle timeout is too low"))
    }
}

fn validate_balancer_sampling(value: &std::time::Duration) -> Result<(), ValidationError> {
    if MIN_BALANCER_SAMPLING_INTERVAL <= *value {
        Ok(())
    } else {
        Err(ValidationError::new("balancer sampling interval is too low"))
    }
}

fn validate_balancer_buffer_duration(value: &std::time::Duration) -> Result<(), ValidationError> {
    if MIN_SURB_BUFFER_DURATION <= *value {
        Ok(())
    } else {
        Err(ValidationError::new("minmum SURB buffer duration is too low"))
    }
}

/// Global configuration of Sessions and the Session manager.
#[serde_as]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Validate, smart_default::SmartDefault)]
#[serde(deny_unknown_fields)]
pub struct SessionGlobalConfig {
    /// Maximum time before an idle Session is closed.
    ///
    /// Defaults to 3 minutes.
    #[validate(custom(function = "validate_session_idle_timeout"))]
    #[default(DEFAULT_SESSION_IDLE_TIMEOUT)]
    #[serde(default = "default_session_idle_timeout")]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub idle_timeout: std::time::Duration,

    /// The maximum number of outgoing or incoming Sessions that
    /// are allowed by the Session manager.
    ///
    /// Minimum is 1, the maximum is given by the Session tag range.
    /// Default is 2048.
    #[default(DEFAULT_SESSION_MAX_SESSIONS)]
    #[serde(default = "default_session_max_sessions")]
    #[validate(range(min = 1))]
    pub maximum_sessions: u32,

    /// Maximum retries to attempt to establish the Session
    /// Set 0 for no retries.
    ///
    /// Defaults to 3, maximum is 20.
    #[validate(range(min = 0, max = 20))]
    #[default(DEFAULT_SESSION_ESTABLISH_MAX_RETRIES)]
    #[serde(default = "default_session_establish_max_retries")]
    pub establish_max_retries: u32,

    /// Delay between Session establishment retries.
    ///
    /// Default is 2 seconds.
    #[default(DEFAULT_SESSION_ESTABLISH_RETRY_DELAY)]
    #[serde(default = "default_session_establish_retry_delay")]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub establish_retry_timeout: std::time::Duration,

    /// Sampling interval for SURB balancer in milliseconds.
    ///
    /// Default is 500 milliseconds.
    #[validate(custom(function = "validate_balancer_sampling"))]
    #[default(DEFAULT_SESSION_BALANCER_SAMPLING)]
    #[serde(default = "default_session_balancer_sampling")]
    #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
    pub balancer_sampling_interval: std::time::Duration,

    /// Minimum runway of received SURBs in seconds.
    ///
    /// This applies to incoming Sessions on Exit nodes only and is the main indicator of how
    /// the egress traffic will be shaped, unless the `NoRateControl` Session
    /// capability is specified during initiation.
    ///
    /// Default is 5 seconds, minimum is 1 second.
    #[validate(custom(function = "validate_balancer_buffer_duration"))]
    #[default(DEFAULT_SESSION_BALANCER_BUFFER_DURATION)]
    #[serde(default = "default_session_balancer_buffer_duration")]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub balancer_minimum_surb_buffer_duration: std::time::Duration,
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

    #[test]
    fn test_multiaddress_on_dappnode_default() {
        temp_env::with_var("DAPPNODE", Some("true"), || {
            assert_eq!(default_multiaddr_transport(1234), "tcp/1234");
        });
    }

    #[cfg(feature = "transport-quic")]
    #[test]
    fn test_multiaddress_on_non_dappnode_default() {
        temp_env::with_vars([("DAPPNODE", Some("false")), ("HOPRD_NAT", Some("false"))], || {
            assert_eq!(default_multiaddr_transport(1234), "udp/1234/quic-v1");
        });
    }

    #[cfg(not(feature = "transport-quic"))]
    #[test]
    fn test_multiaddress_on_non_dappnode_default() {
        assert_eq!(default_multiaddr_transport(1234), "tcp/1234");
    }

    #[test]
    fn test_multiaddress_on_non_dappnode_uses_nat() {
        temp_env::with_var("HOPRD_NAT", Some("true"), || {
            assert_eq!(default_multiaddr_transport(1234), "tcp/1234");
        });
    }

    #[cfg(feature = "transport-quic")]
    #[test]
    fn test_multiaddress_on_non_dappnode_not_uses_nat() {
        temp_env::with_var("HOPRD_NAT", Some("false"), || {
            assert_eq!(default_multiaddr_transport(1234), "udp/1234/quic-v1");
        });
    }

    #[cfg(not(feature = "transport-quic"))]
    #[test]
    fn test_multiaddress_on_non_dappnode_not_uses_nat() {
        temp_env::with_var("HOPRD_NAT", Some("false"), || {
            assert_eq!(default_multiaddr_transport(1234), "tcp/1234");
        });
    }

    #[cfg(feature = "transport-quic")]
    #[test]
    fn test_multiaddress_on_dappnode_not_uses_nat() {
        temp_env::with_vars([("DAPPNODE", Some("true")), ("HOPRD_NAT", Some("false"))], || {
            assert_eq!(default_multiaddr_transport(1234), "udp/1234/quic-v1");
        });
    }

    #[cfg(not(feature = "transport-quic"))]
    #[test]
    fn test_multiaddress_on_dappnode_not_uses_nat() {
        temp_env::with_vars([("DAPPNODE", Some("true")), ("HOPRD_NAT", Some("false"))], || {
            assert_eq!(default_multiaddr_transport(1234), "tcp/1234");
        });
    }
}
