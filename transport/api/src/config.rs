use std::{
    fmt::{Display, Formatter},
    net::ToSocketAddrs,
    num::ParseIntError,
    str::FromStr,
    time::Duration,
};

pub use hopr_protocol_hopr::{HoprCodecConfig, HoprTicketProcessorConfig, SurbStoreConfig};
use hopr_transport_identity::Multiaddr;
pub use hopr_transport_probe::config::ProbeConfig;
use hopr_transport_protocol::PacketPipelineConfig;
use hopr_transport_session::{MIN_BALANCER_SAMPLING_INTERVAL, MIN_SURB_BUFFER_DURATION};
use proc_macro_regex::regex;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::errors::HoprTransportError;

/// Complete configuration of the HOPR protocol stack.
#[derive(Debug, smart_default::SmartDefault, Validate, Clone, Copy, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct HoprProtocolConfig {
    /// Libp2p-related transport configuration
    #[validate(nested)]
    pub transport: TransportConfig,
    /// HOPR packet pipeline configuration
    #[validate(nested)]
    pub packet: HoprPacketPipelineConfig,
    /// Probing protocol configuration
    #[validate(nested)]
    pub probe: ProbeConfig,
    /// Session protocol global configuration
    #[validate(nested)]
    pub session: SessionGlobalConfig,
}

/// Configuration of the HOPR packet pipeline.
#[derive(Clone, Copy, Debug, Default, PartialEq, Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct HoprPacketPipelineConfig {
    /// HOPR packet codec configuration
    #[validate(nested)]
    pub codec: HoprCodecConfig,
    /// HOPR ticket processing configuration
    #[validate(nested)]
    pub ticket_processing: HoprTicketProcessorConfig,
    /// Single Use Reply Block (SURB) handling configuration
    #[validate(nested)]
    pub surb_store: SurbStoreConfig,
    /// Additional configuration affecting the acknowledgement processing
    #[validate(nested)]
    pub pipeline: PacketPipelineConfig,
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
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HostType {
    /// IPv4 based host
    IPv4(String),
    /// DNS based host
    Domain(String),
}

impl validator::Validate for HostType {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match &self {
            HostType::IPv4(ip4) => validate_ipv4_address(ip4).map_err(|e| {
                let mut errs = ValidationErrors::new();
                errs.add("ipv4", e);
                errs
            }),
            HostType::Domain(domain) => validate_dns_address(domain).map_err(|e| {
                let mut errs = ValidationErrors::new();
                errs.add("domain", e);
                errs
            }),
        }
    }
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
#[derive(Debug, Validate, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct HostConfig {
    /// Host on which to listen
    #[cfg_attr(feature = "serde", serde(default))]
    pub address: HostType,
    /// Listening TCP or UDP port (mandatory).
    #[validate(range(min = 1u16))]
    #[cfg_attr(feature = "serde", serde(default))]
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
        if #[cfg(all(feature = "p2p-announce-quic", feature = "p2p-transport-quic"))] {
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

/// Configuration of the physical transport mechanism.
#[derive(Debug, Default, Validate, Clone, Copy, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct TransportConfig {
    /// When true, assume that the node is running in an isolated network and does
    /// not need any connection to nodes outside the subnet
    #[cfg_attr(feature = "serde", serde(default))]
    pub announce_local_addresses: bool,
    /// When true, assume a testnet with multiple nodes running on the same machine
    /// or in the same private IPv4 network
    #[cfg_attr(feature = "serde", serde(default))]
    pub prefer_local_addresses: bool,
}

const DEFAULT_SESSION_MAX_SESSIONS: u32 = 2048;

const DEFAULT_SESSION_IDLE_TIMEOUT: Duration = Duration::from_mins(3);

const SESSION_IDLE_MIN_TIMEOUT: Duration = Duration::from_secs(2);

const DEFAULT_SESSION_ESTABLISH_RETRY_DELAY: Duration = Duration::from_secs(2);

const DEFAULT_SESSION_ESTABLISH_MAX_RETRIES: u32 = 3;

const DEFAULT_SESSION_BALANCER_SAMPLING: Duration = Duration::from_millis(500);

const DEFAULT_SESSION_BALANCER_BUFFER_DURATION: Duration = Duration::from_secs(5);

fn default_session_max_sessions() -> u32 {
    DEFAULT_SESSION_MAX_SESSIONS
}

fn default_session_balancer_buffer_duration() -> Duration {
    DEFAULT_SESSION_BALANCER_BUFFER_DURATION
}

fn default_session_establish_max_retries() -> u32 {
    DEFAULT_SESSION_ESTABLISH_MAX_RETRIES
}

fn default_session_idle_timeout() -> Duration {
    DEFAULT_SESSION_IDLE_TIMEOUT
}

fn default_session_establish_retry_delay() -> Duration {
    DEFAULT_SESSION_ESTABLISH_RETRY_DELAY
}

fn default_session_balancer_sampling() -> Duration {
    DEFAULT_SESSION_BALANCER_SAMPLING
}

fn validate_session_idle_timeout(value: &Duration) -> Result<(), ValidationError> {
    if SESSION_IDLE_MIN_TIMEOUT <= *value {
        Ok(())
    } else {
        Err(ValidationError::new("session idle timeout is too low"))
    }
}

fn validate_balancer_sampling(value: &Duration) -> Result<(), ValidationError> {
    if MIN_BALANCER_SAMPLING_INTERVAL <= *value {
        Ok(())
    } else {
        Err(ValidationError::new("balancer sampling interval is too low"))
    }
}

fn validate_balancer_buffer_duration(value: &Duration) -> Result<(), ValidationError> {
    if MIN_SURB_BUFFER_DURATION <= *value {
        Ok(())
    } else {
        Err(ValidationError::new("minmum SURB buffer duration is too low"))
    }
}

/// Global configuration of Sessions and the Session manager.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Validate, smart_default::SmartDefault)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct SessionGlobalConfig {
    /// Maximum time before an idle Session is closed.
    ///
    /// Defaults to 3 minutes.
    #[validate(custom(function = "validate_session_idle_timeout"))]
    #[default(default_session_idle_timeout())]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_session_idle_timeout", with = "humantime_serde")
    )]
    pub idle_timeout: Duration,

    /// The maximum number of outgoing or incoming Sessions that
    /// are allowed by the Session manager.
    ///
    /// Minimum is 1, the maximum is given by the Session tag range.
    /// Default is 2048.
    #[default(default_session_max_sessions())]
    #[cfg_attr(feature = "serde", serde(default = "default_session_max_sessions"))]
    #[validate(range(min = 1))]
    pub maximum_sessions: u32,

    /// Maximum retries to attempt to establish the Session
    /// Set 0 for no retries.
    ///
    /// Defaults to 3, maximum is 20.
    #[validate(range(min = 0, max = 20))]
    #[default(default_session_establish_max_retries())]
    #[cfg_attr(feature = "serde", serde(default = "default_session_establish_max_retries"))]
    pub establish_max_retries: u32,

    /// Delay between Session establishment retries.
    ///
    /// Default is 2 seconds.
    #[default(default_session_establish_retry_delay())]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_session_establish_retry_delay", with = "humantime_serde")
    )]
    pub establish_retry_timeout: Duration,

    /// Sampling interval for SURB balancer in milliseconds.
    ///
    /// Default is 500 milliseconds.
    #[validate(custom(function = "validate_balancer_sampling"))]
    #[default(default_session_balancer_sampling())]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_session_balancer_sampling", with = "humantime_serde")
    )]
    pub balancer_sampling_interval: Duration,

    /// Minimum runway of received SURBs in seconds.
    ///
    /// This applies to incoming Sessions on Exit nodes only and is the main indicator of how
    /// the egress traffic will be shaped, unless the `NoRateControl` Session
    /// capability is specified during initiation.
    ///
    /// Default is 5 seconds, minimum is 1 second.
    #[validate(custom(function = "validate_balancer_buffer_duration"))]
    #[default(default_session_balancer_buffer_duration())]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_session_balancer_buffer_duration", with = "humantime_serde")
    )]
    pub balancer_minimum_surb_buffer_duration: Duration,
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

    #[cfg(feature = "p2p-announce-quic")]
    #[test]
    fn test_multiaddress_on_non_dappnode_default() {
        temp_env::with_vars([("DAPPNODE", Some("false")), ("HOPRD_NAT", Some("false"))], || {
            assert_eq!(default_multiaddr_transport(1234), "udp/1234/quic-v1");
        });
    }

    #[cfg(not(feature = "p2p-announce-quic"))]
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

    #[cfg(feature = "p2p-announce-quic")]
    #[test]
    fn test_multiaddress_on_non_dappnode_not_uses_nat() {
        temp_env::with_var("HOPRD_NAT", Some("false"), || {
            assert_eq!(default_multiaddr_transport(1234), "udp/1234/quic-v1");
        });
    }

    #[cfg(not(feature = "p2p-announce-quic"))]
    #[test]
    fn test_multiaddress_on_non_dappnode_not_uses_nat() {
        temp_env::with_var("HOPRD_NAT", Some("false"), || {
            assert_eq!(default_multiaddr_transport(1234), "tcp/1234");
        });
    }

    #[cfg(feature = "p2p-announce-quic")]
    #[test]
    fn test_multiaddress_on_dappnode_not_uses_nat() {
        temp_env::with_vars([("DAPPNODE", Some("true")), ("HOPRD_NAT", Some("false"))], || {
            assert_eq!(default_multiaddr_transport(1234), "udp/1234/quic-v1");
        });
    }

    #[cfg(not(feature = "p2p-announce-quic"))]
    #[test]
    fn test_multiaddress_on_dappnode_not_uses_nat() {
        temp_env::with_vars([("DAPPNODE", Some("true")), ("HOPRD_NAT", Some("false"))], || {
            assert_eq!(default_multiaddr_transport(1234), "tcp/1234");
        });
    }
}
