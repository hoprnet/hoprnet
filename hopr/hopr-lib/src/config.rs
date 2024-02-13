use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use validator::{Validate, ValidationError};

pub use core_strategy::StrategyConfig;
pub use core_transport::config::{
    validate_external_host, HeartbeatConfig, HostConfig, HostType, NetworkConfig, ProtocolConfig, TransportConfig,
};

use hopr_primitive_types::prelude::*;

pub const DEFAULT_SAFE_TRANSACTION_SERVICE_PROVIDER: &str = "https://safe-transaction.prod.hoprtech.net/";
pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 9091;

fn validate_announced(v: &bool) -> Result<(), ValidationError> {
    if *v {
        Ok(())
    } else {
        Err(ValidationError::new(
            "Announce option should be turned ON in 2.*, only public nodes are supported",
        ))
    }
}

#[inline]
fn default_network() -> String {
    "anvil-localhost".to_owned()
}

#[inline]
fn just_true() -> bool {
    true
}

// TODO: needs refactoring to use types from crate dependencies
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Serialize, Deserialize, Validate)]
pub struct Chain {
    #[validate(custom = "validate_announced")]
    #[serde(default = "just_true")]
    #[default = true]
    pub announce: bool,
    #[serde(default = "default_network")]
    #[default(default_network())]
    pub network: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub protocols: crate::chain::ProtocolsConfig,
    #[serde(default = "just_true")]
    #[default = true]
    pub check_unrealized_balance: bool, // TODO: should be removed
    #[serde(default = "1000")]
    #[default = 1000]
    pub max_block_range: u32,
    #[serde(default = "3000")]
    #[default = 3000]
    pub tx_polling_interval: u32,
}

#[inline]
fn default_invalid_address() -> Address {
    Address::from_bytes(&[0; Address::SIZE]).unwrap()
}

#[inline]
fn default_safe_transaction_service_provider() -> String {
    DEFAULT_SAFE_TRANSACTION_SERVICE_PROVIDER.to_owned()
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Serialize, Deserialize, Validate)]
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
    #[validate]
    #[validate(custom = "validate_external_host")]
    #[serde(default = "default_host")]
    #[default(default_host())]
    pub host: HostConfig,
    /// Configuration of the underlying database engine
    #[validate]
    #[serde(default)]
    pub db: Db,
    /// Configuration of underlying node behavior in the form strategies
    ///
    /// Strategies represent automatically executable behavior performed by
    /// the node given pre-configured triggers.
    #[validate]
    #[serde(default = "core_strategy::hopr_default_strategies")]
    #[default(core_strategy::hopr_default_strategies())]
    pub strategy: StrategyConfig,
    /// Configuration of the protocol heartbeat mechanism
    #[validate]
    #[serde(default)]
    pub heartbeat: HeartbeatConfig,
    /// Configuration of network properties
    #[validate]
    #[serde(default)]
    pub network_options: NetworkConfig,
    /// Configuration specific to transport mechanics
    #[validate]
    #[serde(default)]
    pub transport: TransportConfig,
    /// Configuration specific to protocol execution on the p2p layer
    #[validate]
    #[serde(default)]
    pub protocol: ProtocolConfig,
    /// Blockchain specific configuration
    #[validate]
    #[serde(default)]
    pub chain: Chain,
    /// Configuration of the `Safe` mechanism
    #[validate]
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
