use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use validator::{Validate, ValidationError};

pub use core_strategy::StrategyConfig;
pub use core_transport::config::{HeartbeatConfig, HostConfig, NetworkConfig, ProtocolConfig, TransportConfig};
use utils_types::{primitives::Address, traits::BinarySerializable};

pub const DEFAULT_SAFE_TRANSACTION_SERVICE_PROVIDER: &str = "https://safe-transaction.stage.hoprtech.net/";
pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 9091;

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Chain {
    pub announce: bool,
    pub network: String,
    pub provider: Option<String>,
    pub protocols: crate::chain::ProtocolsConfig,
    pub check_unrealized_balance: bool,
}

impl Default for Chain {
    fn default() -> Self {
        Self {
            announce: false,
            network: "anvil-localhost".to_owned(),
            provider: None,
            protocols: crate::chain::ProtocolsConfig::default(),
            check_unrealized_balance: true,
        }
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct SafeModule {
    #[validate(url)]
    pub safe_transaction_service_provider: String,
    #[serde_as(as = "DisplayFromStr")]
    pub safe_address: Address,
    #[serde_as(as = "DisplayFromStr")]
    pub module_address: Address,
}

impl Default for SafeModule {
    fn default() -> Self {
        Self {
            safe_transaction_service_provider: DEFAULT_SAFE_TRANSACTION_SERVICE_PROVIDER.to_owned(),
            safe_address: Address::from_bytes(&[0; Address::SIZE]).unwrap(),
            module_address: Address::from_bytes(&[0; Address::SIZE]).unwrap(),
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
    pub data: String,
    pub initialize: bool,
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
    pub host: HostConfig,
    // /// Configuration regarding the identity of the node
    // #[validate]
    // pub identity: Identity,
    /// Configuration of the underlying database engine
    #[validate]
    pub db: Db,
    /// Configuration of underlying node behavior in the form strategies
    ///
    /// Strategies represent automatically executable behavior performed by
    /// the node given pre-configured triggers.
    #[validate]
    pub strategy: StrategyConfig,
    /// Configuration of the protocol heartbeat mechanism
    #[validate]
    pub heartbeat: HeartbeatConfig,
    /// Configuration of network properties
    #[validate]
    pub network_options: NetworkConfig,
    /// Configuration specific to protocol execution on the p2p layer
    #[validate]
    pub transport: TransportConfig,
    /// Configuration specific to protocol execution on the p2p layer
    #[validate]
    pub protocol: ProtocolConfig,
    /// Blockchain specific configuration
    #[validate]
    pub chain: Chain,
    /// Configuration of the `Safe` mechanism
    #[validate]
    pub safe_module: SafeModule,
}

impl Default for HoprLibConfig {
    fn default() -> Self {
        Self {
            host: HostConfig::from_str(format!("{DEFAULT_HOST}:{DEFAULT_PORT}").as_str()).unwrap(),
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
