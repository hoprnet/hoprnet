use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use validator::{Validate, ValidationError};

pub use core_strategy::StrategyConfig;
pub use core_transport::config::{
    validate_external_host, HeartbeatConfig, HostConfig, NetworkConfig, ProtocolConfig, TransportConfig,
};
use core_transport::config::HostType;

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

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Chain {
    #[validate(custom = "validate_announced")]
    #[serde(default = "just_true")]
    pub announce: bool,
    #[serde(default = "default_network")]
    pub network: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub protocols: crate::chain::ProtocolsConfig,
    #[serde(default = "just_true")]
    pub check_unrealized_balance: bool,
}

impl Default for Chain {
    fn default() -> Self {
        Self {
            announce: true,
            network: default_network(),
            provider: None,
            protocols: crate::chain::ProtocolsConfig::default(),
            check_unrealized_balance: true,
        }
    }
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
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct SafeModule {
    #[validate(url)]
    #[serde(default = "default_safe_transaction_service_provider")]
    pub safe_transaction_service_provider: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "default_invalid_address")]
    pub safe_address: Address,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "default_invalid_address")]
    pub module_address: Address,
}

impl Default for SafeModule {
    fn default() -> Self {
        Self {
            safe_transaction_service_provider: default_safe_transaction_service_provider(),
            safe_address: default_invalid_address(),
            module_address: default_invalid_address(),
        }
    }
}

#[allow(dead_code)]
fn validate_directory_exists(s: &str) -> Result<(), ValidationError> {
    if std::path::Path::new(s).is_dir() {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid directory path specified"))
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Db {
    /// Path to the directory containing the database
    #[serde(default)]
    pub data: String,
    #[serde(default = "just_true")]
    pub initialize: bool,
    #[serde(default)]
    pub force_initialize: bool,
}

impl Default for Db {
    fn default() -> Self {
        Self {
            data: "".to_owned(),
            initialize: true,
            force_initialize: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct HoprLibConfig {
    /// Configuration related to host specifics
    #[validate]
    #[validate(custom = "validate_external_host")]
    #[serde(default = "default_host")]
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

#[inline]
fn default_host() -> HostConfig {
    HostConfig {
        address: HostType::IPv4(DEFAULT_HOST.to_owned()),
        port: DEFAULT_PORT,
    }
}

impl Default for HoprLibConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            db: Db::default(),
            strategy: core_strategy::hopr_default_strategies(),
            heartbeat: HeartbeatConfig::default(),
            network_options: NetworkConfig::default(),
            transport: TransportConfig::default(),
            protocol: ProtocolConfig::default(),
            chain: Chain::default(),
            safe_module: SafeModule::default(),
        }
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
