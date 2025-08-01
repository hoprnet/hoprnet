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

fn validate_announced(v: &bool) -> Result<(), ValidationError> {
    if *v {
        Ok(())
    } else {
        Err(ValidationError::new(
            "Announce option should be turned ON in 2.*, only public nodes are supported",
        ))
    }
}

fn validate_logs_snapshot_url(url: &&String) -> Result<(), ValidationError> {
    if url.is_empty() {
        return Err(ValidationError::new("Logs snapshot URL must not be empty"));
    }

    // Basic URL validation (allow file:// for testing)
    if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("file://") {
        return Err(ValidationError::new(
            "Logs snapshot URL must be a valid HTTP, HTTPS, or file:// URL",
        ));
    }

    // Check if URL ends with .tar.xz
    if !url.ends_with(".tar.xz") {
        return Err(ValidationError::new("Logs snapshot URL must point to a .tar.xz file"));
    }

    Ok(())
}

#[inline]
fn default_network() -> String {
    "anvil-localhost".to_owned()
}

#[inline]
fn just_true() -> bool {
    true
}

#[inline]
fn just_false() -> bool {
    false
}

#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Chain {
    #[validate(custom(function = "validate_announced"))]
    #[serde(default = "just_true")]
    #[default = true]
    pub announce: bool,
    #[serde(default = "default_network")]
    #[default(default_network())]
    pub network: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub max_rpc_requests_per_sec: Option<u32>,
    #[serde(default)]
    pub protocols: hopr_chain_api::config::ProtocolsConfig,
    #[serde(default = "just_true")]
    #[default = true]
    pub keep_logs: bool,
    #[serde(default = "just_true")]
    #[default = true]
    pub fast_sync: bool,
    #[serde(default = "just_false")]
    #[default = false]
    pub enable_logs_snapshot: bool,
    #[validate(custom(function = "validate_logs_snapshot_url"))]
    #[serde(default)]
    pub logs_snapshot_url: Option<String>,
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
    /// Blockchain-specific configuration
    #[validate(nested)]
    #[serde(default)]
    pub chain: Chain,
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
