use std::time::Duration;

use validator::Validate;

/// Configuration for the `heartbeat` protocol.
#[derive(Debug, Copy, Clone, smart_default::SmartDefault, Validate, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct HeartbeatProtocolConfig {
    /// Maximum duration before the request times out
    #[cfg_attr(feature = "serde", serde(with = "humantime_serde"))]
    #[default(Duration::from_secs(6))]
    pub timeout: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heartbeat_config_should_have_stable_defaults() {
        let cfg = HeartbeatProtocolConfig::default();
        insta::assert_debug_snapshot!(cfg);
    }

    #[test]
    fn heartbeat_config_default_should_validate() -> anyhow::Result<()> {
        use anyhow::Context;
        let cfg = HeartbeatProtocolConfig::default();
        cfg.validate()
            .context("default HeartbeatProtocolConfig should be valid")?;
        Ok(())
    }
}
