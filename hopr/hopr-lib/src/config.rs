use std::time::Duration;

use hopr_api::types::primitive::prelude::*;
pub use hopr_transport::{
    TagAllocatorConfig,
    config::{
        HoprPacketPipelineConfig, HoprProtocolConfig, HostConfig, HostType, MixerConfig, ProbeConfig,
        SessionGlobalConfig, TransitLatencyConfig, TransportConfig, looks_like_domain,
    },
};
use validator::{Validate, ValidationError};

pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 9091;

#[inline]
fn default_invalid_address() -> Address {
    Address::default()
}

#[cfg_attr(feature = "serde", cfg_eval::cfg_eval, serde_with::serde_as)]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct SafeModule {
    #[cfg_attr(
        feature = "serde",
        serde_as(as = "serde_with::DisplayFromStr"),
        serde(default = "default_invalid_address")
    )]
    #[default(default_invalid_address())]
    pub safe_address: Address,
    #[cfg_attr(
        feature = "serde",
        serde_as(as = "serde_with::DisplayFromStr"),
        serde(default = "default_invalid_address")
    )]
    #[default(default_invalid_address())]
    pub module_address: Address,
}

#[cfg(feature = "session-server")]
#[inline]
fn default_incoming_session_capacity() -> usize {
    256
}

#[allow(dead_code)]
fn validate_directory_exists(s: &str) -> Result<(), ValidationError> {
    if std::path::Path::new(s).is_dir() {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid directory path specified"))
    }
}

#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct HoprLibConfig {
    /// Configuration related to host specifics
    #[validate(nested)]
    #[default(default_host())]
    #[cfg_attr(feature = "serde", serde(default = "default_host"))]
    pub host: HostConfig,
    /// Determines whether the node should be advertised publicly on-chain.
    #[cfg_attr(feature = "serde", serde(default))]
    pub publish: bool,
    /// Configuration of the HOPR protocol.
    #[validate(nested)]
    #[cfg_attr(feature = "serde", serde(default))]
    pub protocol: HoprProtocolConfig,
    /// Configuration of the node Safe and Module.
    #[validate(nested)]
    #[cfg_attr(feature = "serde", serde(default))]
    pub safe_module: SafeModule,
    /// Defines how often the outgoing ticket indices be saved to the persistent storage.
    ///
    /// If synchronization to a persistent storage does not happen and the node restarts,
    /// the node will start from the current on-chain channel index and could as a result
    /// be creating invalid outgoing tickets.
    ///
    /// Default is 15 seconds, minimum is 1 second.
    #[default(default_out_index_sync_period())]
    #[validate(custom(function = "validate_out_index_sync_period"))]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_out_index_sync_period", with = "humantime_serde")
    )]
    pub out_index_sync_period: Duration,
    /// Capacity of the incoming session channel (number of buffered sessions).
    ///
    /// Only relevant when the `session-server` feature is enabled. Default is 256.
    #[cfg(feature = "session-server")]
    #[default(default_incoming_session_capacity())]
    #[cfg_attr(feature = "serde", serde(default = "default_incoming_session_capacity"))]
    pub incoming_session_capacity: usize,
    /// Disables win-probability and ticket-price protocol safety checks.
    ///
    /// Only available in debug builds. Never set in production.
    #[cfg(debug_assertions)]
    #[cfg_attr(feature = "serde", serde(default))]
    pub disable_protocol_checks: bool,
}

const MINIMUM_OUT_SYNC_PERIOD: Duration = Duration::from_secs(1);

fn validate_out_index_sync_period(lifetime: &Duration) -> Result<(), ValidationError> {
    if lifetime < &MINIMUM_OUT_SYNC_PERIOD {
        Err(ValidationError::new("out_index_sync_period is too low"))
    } else {
        Ok(())
    }
}

fn default_out_index_sync_period() -> Duration {
    Duration::from_secs(15)
}

// NOTE: this intentionally does not validate (0.0.0.0) to force user to specify
// their external IP.
#[inline]
fn default_host() -> HostConfig {
    HostConfig {
        address: HostType::IPv4(DEFAULT_HOST.to_owned()),
        port: DEFAULT_PORT,
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "serde")]
    #[test]
    fn test_config_should_be_serializable_using_serde() -> Result<(), Box<dyn std::error::Error>> {
        let cfg = super::HoprLibConfig::default();

        let yaml = serde_saphyr::to_string(&cfg)?;
        let cfg_after_serde: super::HoprLibConfig = serde_saphyr::from_str(&yaml)?;
        assert_eq!(cfg, cfg_after_serde);

        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test]
    fn explicit_mixer_section_round_trips() -> anyhow::Result<()> {
        use std::time::Duration;

        let mut cfg = super::HoprLibConfig::default();
        cfg.protocol.mixer = super::MixerConfig {
            min_delay: Duration::from_millis(5),
            delay_range: Duration::from_millis(50),
            capacity: 1_000,
            ..Default::default()
        };
        insta::assert_yaml_snapshot!(cfg.protocol.mixer);

        let yaml = serde_saphyr::to_string(&cfg.protocol.mixer)?;
        let parsed: super::MixerConfig = serde_saphyr::from_str(&yaml)?;
        assert_eq!(cfg.protocol.mixer, parsed);

        Ok(())
    }
}
