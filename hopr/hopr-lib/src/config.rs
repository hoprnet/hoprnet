use hopr_primitive_types::prelude::*;
pub use hopr_transport::config::{
    HoprPacketPipelineConfig, HoprProtocolConfig, HostConfig, HostType, NetworkConfig, ProbeConfig,
    SessionGlobalConfig, TransportConfig, looks_like_domain,
};
use validator::{Validate, ValidationError};

pub const DEFAULT_SAFE_TRANSACTION_SERVICE_PROVIDER: &str = "https://safe-transaction.prod.hoprtech.net/";
pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 9091;

#[inline]
fn default_invalid_address() -> Address {
    Address::default()
}

#[inline]
fn default_safe_transaction_service_provider() -> String {
    DEFAULT_SAFE_TRANSACTION_SERVICE_PROVIDER.to_owned()
}

#[cfg_attr(feature = "serde", cfg_eval::cfg_eval, serde_with::serde_as)]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct SafeModule {
    #[validate(url)]
    #[cfg_attr(feature = "serde", serde(default = "default_safe_transaction_service_provider"))]
    #[default(default_safe_transaction_service_provider())]
    pub safe_transaction_service_provider: String,
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
    use super::*;

    #[test]
    fn test_config_should_be_serializable_using_serde() -> Result<(), Box<dyn std::error::Error>> {
        let cfg = HoprLibConfig::default();

        let yaml = serde_yaml::to_string(&cfg)?;
        let cfg_after_serde: HoprLibConfig = serde_yaml::from_str(&yaml)?;
        assert_eq!(cfg, cfg_after_serde);

        Ok(())
    }
}
