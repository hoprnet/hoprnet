use serde::{Deserialize, Serialize};
use validator::Validate;

fn validate_interval_ge_timeout(config: &ProbeConfig) -> Result<(), validator::ValidationError> {
    if config.interval < config.timeout {
        let mut err = validator::ValidationError::new("interval_less_than_timeout");
        err.message = Some(
            format!(
                "probe interval ({:?}) must be >= timeout ({:?}) to prevent overlapping rounds",
                config.interval, config.timeout
            )
            .into(),
        );
        return Err(err);
    }
    Ok(())
}

/// Configuration for the probing mechanism
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_interval_ge_timeout"))]
pub struct ProbeConfig {
    /// The waiting time for a reply from the probe.
    #[default(default_max_probe_timeout())]
    #[serde(default = "default_max_probe_timeout", with = "humantime_serde")]
    pub timeout: std::time::Duration,

    /// Maximum number of parallel probes performed by the mechanism
    #[validate(range(min = 1))]
    #[default(default_max_parallel_probes())]
    #[serde(default = "default_max_parallel_probes")]
    pub max_parallel_probes: usize,

    /// The delay between individual probing rounds for neighbor discovery.
    ///
    /// Must be >= `timeout` to prevent overlapping probe rounds, which causes
    /// pseudonym reuse in the probe cache and missed pong responses.
    #[serde(default = "default_probing_interval", with = "humantime_serde")]
    #[default(default_probing_interval())]
    pub interval: std::time::Duration,

    /// The time threshold after which it is reasonable to recheck the nearest neighbor
    #[serde(default = "default_recheck_threshold", with = "humantime_serde")]
    #[default(default_recheck_threshold())]
    pub recheck_threshold: std::time::Duration,
}

/// The maximum time waiting for a reply from the probe
const DEFAULT_MAX_PROBE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

/// The maximum number of parallel probes the heartbeat performs
const DEFAULT_MAX_PARALLEL_PROBES: usize = 50;

/// Delay before repeating probing rounds, must include enough time to traverse NATs
const DEFAULT_REPEATED_PROBING_DELAY: std::time::Duration = std::time::Duration::from_secs(5);

/// Time after which the availability of a node gets rechecked
const DEFAULT_PROBE_RECHECK_THRESHOLD: std::time::Duration = std::time::Duration::from_secs(60);

#[inline]
const fn default_max_probe_timeout() -> std::time::Duration {
    DEFAULT_MAX_PROBE_TIMEOUT
}

#[inline]
const fn default_max_parallel_probes() -> usize {
    DEFAULT_MAX_PARALLEL_PROBES
}

#[inline]
const fn default_probing_interval() -> std::time::Duration {
    DEFAULT_REPEATED_PROBING_DELAY
}

#[inline]
const fn default_recheck_threshold() -> std::time::Duration {
    DEFAULT_PROBE_RECHECK_THRESHOLD
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use validator::Validate;

    use super::*;

    #[test]
    fn probe_config_default_is_valid() -> anyhow::Result<()> {
        let cfg = ProbeConfig::default();
        cfg.validate().context("default ProbeConfig should be valid")?;
        assert!(cfg.interval >= cfg.timeout);
        insta::assert_yaml_snapshot!(cfg);
        Ok(())
    }

    #[test]
    fn probe_config_zero_parallel_probes_is_rejected() -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            max_parallel_probes: 0,
            ..Default::default()
        };
        let err = cfg.validate().err().context("expected validation error")?;
        anyhow::ensure!(
            err.field_errors().contains_key("max_parallel_probes"),
            "expected max_parallel_probes field error, got: {err}"
        );
        Ok(())
    }

    #[test]
    fn probe_config_one_parallel_probe_is_valid() -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            max_parallel_probes: 1,
            ..Default::default()
        };
        cfg.validate().context("single parallel probe should be valid")?;
        Ok(())
    }

    #[test]
    fn interval_less_than_timeout_should_be_rejected() {
        let config = ProbeConfig {
            timeout: std::time::Duration::from_secs(10),
            interval: std::time::Duration::from_secs(5),
            ..Default::default()
        };
        assert!(config.validate().is_err(), "interval < timeout must be rejected");
    }

    #[test]
    fn interval_equal_to_timeout_should_be_accepted() {
        let config = ProbeConfig {
            timeout: std::time::Duration::from_secs(5),
            interval: std::time::Duration::from_secs(5),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}
