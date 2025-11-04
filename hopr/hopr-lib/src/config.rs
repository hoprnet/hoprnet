use hopr_primitive_types::prelude::*;
pub use hopr_strategy::StrategyConfig;
use hopr_transport::config::SessionGlobalConfig;
pub use hopr_transport::config::{
    HostConfig, HostType, NetworkConfig, ProbeConfig, ProtocolConfig, TransportConfig, validate_external_host,
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use validator::{Validate, ValidationError};

pub const DEFAULT_SAFE_TRANSACTION_SERVICE_PROVIDER: &str = "https://safe-transaction.prod.hoprtech.net/";
pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 9091;

#[inline]
fn just_true() -> bool {
    true
}

#[inline]
fn default_invalid_address() -> Address {
    Address::default()
}

#[inline]
fn default_safe_transaction_service_provider() -> String {
    DEFAULT_SAFE_TRANSACTION_SERVICE_PROVIDER.to_owned()
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct SafeModule {
    #[validate(url)]
    #[serde(default = "default_safe_transaction_service_provider")]
    #[default(default_safe_transaction_service_provider())]
    pub safe_transaction_service_provider: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "default_invalid_address")]
    #[default(default_invalid_address())]
    pub safe_address: Address,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "default_invalid_address")]
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

#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Db {
    /// Path to the directory containing the database
    #[serde(default)]
    pub data: String,
    #[serde(default = "just_true")]
    #[default = true]
    pub initialize: bool,
    #[serde(default)]
    pub force_initialize: bool,
}

#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Serialize, Deserialize, Validate)]
pub struct HoprLibConfig {
    /// Configuration related to host specifics
    #[validate(nested)]
    #[validate(custom(function = "validate_external_host"))]
    #[serde(default = "default_host")]
    #[default(default_host())]
    pub host: HostConfig,
    /// Determines whether the node should be advertised publicly on-chain.
    #[serde(default)]
    pub publish: bool,
    /// Configuration of the underlying database engine
    #[validate(nested)]
    #[serde(default)]
    pub db: Db,
    /// Configuration of underlying node behavior in the form strategies
    ///
    /// Strategies represent automatically executable behavior performed by
    /// the node given pre-configured triggers.
    #[validate(nested)]
    #[serde(default = "hopr_strategy::hopr_default_strategies")]
    #[default(hopr_strategy::hopr_default_strategies())]
    pub strategy: StrategyConfig,
    /// Configuration of the protocol heartbeat mechanism
    #[validate(nested)]
    #[serde(default)]
    pub probe: ProbeConfig,
    /// Configuration of network properties
    #[validate(nested)]
    #[serde(default)]
    pub network_options: NetworkConfig,
    /// Configuration specific to transport mechanics
    #[validate(nested)]
    #[serde(default)]
    pub transport: TransportConfig,
    /// Configuration specific to protocol execution on the p2p layer
    #[validate(nested)]
    #[serde(default)]
    pub protocol: ProtocolConfig,
    /// Configuration specific to Session management.
    #[validate(nested)]
    #[serde(default)]
    pub session: SessionGlobalConfig,
    /// Configuration of the `Safe` mechanism
    #[validate(nested)]
    #[serde(default)]
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
