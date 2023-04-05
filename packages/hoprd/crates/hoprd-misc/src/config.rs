use proc_macro_regex::regex;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use core_ethereum_misc::constants::DEFAULT_CONFIRMATIONS;
use core_misc::constants::{
    DEFAULT_HEARTBEAT_INTERVAL, DEFAULT_HEARTBEAT_INTERVAL_VARIANCE, DEFAULT_HEARTBEAT_THRESHOLD,
    DEFAULT_MAX_PARALLEL_CONNECTIONS, DEFAULT_MAX_PARALLEL_CONNECTION_PUBLIC_RELAY, DEFAULT_NETWORK_QUALITY_THRESHOLD,
};

use utils_proc_macros::wasm_bindgen_if;

pub const DEFAULT_API_HOST: &str = "localhost";
pub const DEFAULT_API_PORT: u16 = 3001;

pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 9091;

pub const DEFAULT_HEALTH_CHECK_HOST: &str = "localhost";
pub const DEFAULT_HEALTH_CHECK_PORT: u16 = 8080;

pub const MINIMAL_API_TOKEN_LENGTH: usize = 8;

fn validate_ipv4_address(s: &str) -> Result<(), ValidationError> {
    if validator::validate_ip(s) {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid IPv4 address provided"))
    }
}

fn validate_api_auth(token: &Auth) -> Result<(), ValidationError> {
    match &token {
        Auth::None => Ok(()),
        Auth::Token(token) => {
            if token.len() >= MINIMAL_API_TOKEN_LENGTH {
                // TODO: add more token limitations? alhpanumeric?
                Ok(())
            } else {
                Err(ValidationError::new("The validation token is too short"))
            }
        }
    }
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Default, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Host {
    #[validate(custom = "validate_ipv4_address")]
    pub ip: String,
    #[validate(range(min = 1u16))]
    pub port: u16,
}

impl Host {
    pub fn from_ipv4_host_string(s: &str) -> Result<Self, String> {
        let (ip, str_port) = match s.split_once(":") {
            None => return Err(format!("Invalid host")),
            Some(split) => split,
        };

        let port = u16::from_str_radix(str_port, 10).map_err(|e| e.to_string())?;

        Ok(Self {
            ip: ip.to_owned(),
            port,
        })
    }
}

impl ToString for Host {
    fn to_string(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Auth {
    None,
    Token(String),
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Validate, Serialize, Deserialize, Clone, PartialEq)]
pub struct Api {
    pub enabled: bool,
    /// Auth enum holding the API auth configuration
    ///
    /// The auth enum cannot be made public due to incompatibility with the wasm_bindgen.
    #[validate(custom = "validate_api_auth")]
    auth: Auth,
    #[validate]
    pub host: Host,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Api {
    pub fn is_auth_disabled(&self) -> bool {
        self.auth == Auth::None
    }

    pub fn auth_token(&self) -> Option<String> {
        match &self.auth {
            Auth::None => None,
            Auth::Token(token) => Some(token.clone()),
        }
    }
}

impl Default for Api {
    fn default() -> Self {
        Self {
            enabled: false,
            auth: Auth::Token("".to_owned()),
            host: Host {
                ip: DEFAULT_API_HOST.to_string(),
                port: DEFAULT_API_PORT,
            },
        }
    }
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct HealthCheck {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self {
            enabled: false,
            host: DEFAULT_HEALTH_CHECK_HOST.to_string(),
            port: DEFAULT_HEALTH_CHECK_PORT,
        }
    }
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Heartbeat {
    pub interval: u32,
    pub threshold: u32,
    pub variance: u32,
}

impl Default for Heartbeat {
    fn default() -> Self {
        Self {
            interval: DEFAULT_HEARTBEAT_INTERVAL,
            threshold: DEFAULT_HEARTBEAT_THRESHOLD,
            variance: DEFAULT_HEARTBEAT_INTERVAL_VARIANCE,
        }
    }
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Network {
    pub announce: bool,
    pub allow_local_node_connections: bool,
    pub allow_private_node_connections: bool,
    pub max_parallel_connections: u32,
    pub network_quality_threshold: f32,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            announce: false,
            allow_local_node_connections: false,
            allow_private_node_connections: false,
            max_parallel_connections: DEFAULT_MAX_PARALLEL_CONNECTIONS,
            network_quality_threshold: DEFAULT_NETWORK_QUALITY_THRESHOLD,
        }
    }
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Chain {
    pub provider: Option<String>,
    pub check_unrealized_balance: bool,
    pub on_chain_confirmations: u32,
}

impl Default for Chain {
    fn default() -> Self {
        Self {
            provider: None,
            check_unrealized_balance: true,
            on_chain_confirmations: DEFAULT_CONFIRMATIONS,
        }
    }
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Strategy {
    // TODO: implement checks
    pub name: Option<String>,
    pub max_auto_channels: Option<u32>,
    pub auto_redeem_tickets: bool,
}

impl Default for Strategy {
    fn default() -> Self {
        Self {
            name: Some("passive".to_owned()),
            max_auto_channels: None,
            auto_redeem_tickets: true,
        }
    }
}

fn validate_file_path(s: &str) -> Result<(), ValidationError> {
    if std::path::Path::new(s).is_file() {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid file path specified"))
    }
}

fn validate_password(s: &str) -> Result<(), ValidationError> {
    if !s.is_empty() {
        Ok(())
    } else {
        Err(ValidationError::new("No password could be found"))
    }
}

regex!(is_private_key "^[a-fA-F0-9]{64}$");
regex!(is_prefixed_private_key "^0x[a-fA-F0-9]{64}$");

pub(crate) fn validate_private_key(s: &str) -> Result<(), ValidationError> {
    if is_private_key(s) || is_prefixed_private_key(s) {
        Ok(())
    } else {
        Err(ValidationError::new("No valid private key could be found"))
    }
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Default, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Identity {
    #[validate(custom = "validate_file_path")] // TODO: works in wasm?
    pub file: String,
    #[validate(custom = "validate_password")]
    pub password: String,
    #[validate(custom = "validate_private_key")]
    pub private_key: String,
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let obfuscated: String = "<REDACTED>".into();

        f.debug_struct("Identity")
            .field("file", &self.file)
            .field("password", &obfuscated)
            .field("private_key", &obfuscated)
            .finish()
    }
}

fn validate_directory_path(s: &str) -> Result<(), ValidationError> {
    if std::path::Path::new(s).is_dir() {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid directory path specified"))
    }
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Default, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Db {
    /// Path to the directory containing the database
    #[validate(custom = "validate_directory_path")] // TODO: works in wasm?
    pub data: String,
    pub init: bool,
    pub force_init: bool,
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Default, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Testing {
    pub announce_local_addresses: bool,
    pub prefer_local_addresses: bool,
    pub use_weak_crypto: bool,
    pub no_direct_connections: bool,
    pub no_webrtc_upgrade: bool,
    pub local_mode_stun: bool,
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct HoprdConfig {
    #[validate]
    pub host: Host,
    #[validate]
    pub identity: Identity,
    #[validate]
    pub db: Db,
    #[validate]
    pub api: Api,
    #[validate]
    pub strategy: Strategy,
    #[validate]
    pub heartbeat: Heartbeat,
    #[validate]
    pub network: Network,
    #[validate]
    pub healthcheck: HealthCheck,
    pub environment: String,
    #[validate]
    pub chain: Chain,

    #[validate]
    pub test: Testing,
}

impl Default for HoprdConfig {
    fn default() -> Self {
        Self {
            host: Host {
                ip: DEFAULT_HOST.to_string(),
                port: DEFAULT_PORT,
            },
            identity: Identity::default(),
            db: Db::default(),
            api: Api::default(),
            strategy: Strategy::default(),
            heartbeat: Heartbeat::default(),
            network: Network::default(),
            healthcheck: HealthCheck::default(),
            environment: String::default(),
            chain: Chain::default(),
            test: Testing::default(),
        }
    }
}

#[cfg(any(not(feature = "wasm"), test))]
use real_base::file::native::read_to_string;

#[cfg(all(feature = "wasm", not(test)))]
use real_base::file::wasm::read_to_string;

impl HoprdConfig {
    pub fn from_cli_args(cli_args: crate::cli::CliArgs, skip_validation: bool) -> crate::errors::Result<HoprdConfig> {
        let mut cfg: HoprdConfig = if let Some(cfg_path) = cli_args.configuration_file_path {
            let yaml_configuration = read_to_string(cfg_path.as_str())
                .map_err(|e| crate::errors::HoprdConfigError::FileError(e.to_string()))?;
            serde_yaml::from_str(&yaml_configuration)
                .map_err(|e| crate::errors::HoprdConfigError::SerializationError(e.to_string()))?
        } else {
            HoprdConfig::default()
        };

        cfg.environment = cli_args.environment;

        // host
        if let Some(x) = cli_args.host {
            cfg.host = x
        };

        // db
        cfg.db.data = cli_args.data;
        if let Some(x) = cli_args.init {
            cfg.db.init = x
        };
        if let Some(x) = cli_args.force_init {
            cfg.db.force_init = x
        };

        // api
        if let Some(x) = cli_args.api {
            cfg.api.enabled = x
        };
        if let Some(x) = cli_args.disable_api_authentication {
            if !x {
                if &cfg.api.auth != &Auth::None {
                    cfg.api.auth = Auth::None;
                }
            }
        };
        if let Some(x) = cli_args.api_token {
            cfg.api.auth = Auth::Token(x);
        };
        if let Some(x) = cli_args.api_host {
            cfg.api.host.ip = x
        };
        if let Some(x) = cli_args.api_port {
            cfg.api.host.port = x
        };

        // heartbeat
        if let Some(x) = cli_args.heartbeat_interval {
            cfg.heartbeat.interval = x
        };
        if let Some(x) = cli_args.heartbeat_threshold {
            cfg.heartbeat.threshold = x
        };
        if let Some(x) = cli_args.heartbeat_variance {
            cfg.heartbeat.variance = x
        };

        // network
        if let Some(x) = cli_args.announce {
            cfg.network.announce = x
        };
        if let Some(x) = cli_args.allow_local_node_connections {
            cfg.network.allow_local_node_connections = x
        };
        if let Some(x) = cli_args.allow_private_node_connections {
            cfg.network.allow_private_node_connections = x
        };
        if let Some(x) = cli_args.max_parallel_connections {
            cfg.network.max_parallel_connections = x
        } else if cfg.network.announce {
            cfg.network.max_parallel_connections = DEFAULT_MAX_PARALLEL_CONNECTION_PUBLIC_RELAY
        };
        if let Some(x) = cli_args.network_quality_threshold {
            cfg.network.network_quality_threshold = x
        };

        // healthcheck
        if let Some(x) = cli_args.health_check {
            cfg.healthcheck.enabled = x
        };
        if let Some(x) = cli_args.health_check_host {
            cfg.healthcheck.host = x
        };
        if let Some(x) = cli_args.health_check_port {
            cfg.healthcheck.port = x
        };

        // identity
        cfg.identity.file = cli_args.identity;
        if let Some(x) = cli_args.password {
            cfg.identity.password = x
        };
        if let Some(x) = cli_args.private_key {
            cfg.identity.private_key = x
        };

        // strategy
        if let Some(x) = cli_args.default_strategy {
            cfg.strategy.name = Some(x)
        };
        if let Some(x) = cli_args.max_auto_channels {
            cfg.strategy.max_auto_channels = Some(x)
        };
        if let Some(x) = cli_args.auto_redeem_tickets {
            cfg.strategy.auto_redeem_tickets = x
        };

        // chain
        if let Some(x) = cli_args.provider {
            cfg.chain.provider = Some(x)
        };
        if let Some(x) = cli_args.check_unrealized_balance {
            cfg.chain.check_unrealized_balance = x
        };
        if let Some(x) = cli_args.on_chain_confirmations {
            cfg.chain.on_chain_confirmations = x
        };

        // test
        if let Some(x) = cli_args.test_announce_local_addresses {
            cfg.test.announce_local_addresses = x
        };
        if let Some(x) = cli_args.test_prefer_local_addresses {
            cfg.test.prefer_local_addresses = x
        };
        if let Some(x) = cli_args.test_local_mode_stun {
            cfg.test.local_mode_stun = x
        };
        if let Some(x) = cli_args.test_no_webrtc_upgrade {
            cfg.test.no_webrtc_upgrade = x
        };
        if let Some(x) = cli_args.test_no_direct_connections {
            cfg.test.no_direct_connections = x
        };
        if let Some(x) = cli_args.test_use_weak_crypto {
            cfg.test.use_weak_crypto = x
        };

        if skip_validation {
            Ok(cfg)
        } else {
            match cfg.validate() {
                Ok(_) => Ok(cfg),
                Err(e) => Err(crate::errors::HoprdConfigError::ValidationError(e.to_string())),
            }
        }
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::config::HoprdConfig;
    use utils_misc::ok_or_jserr;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen]
    impl HoprdConfig {
        #[wasm_bindgen]
        pub fn as_string(&self) -> String {
            let output = serde_json::to_string(&self);
            match output {
                Ok(o) => o,
                Err(e) => e.to_string(),
            }
        }
    }

    #[wasm_bindgen]
    pub fn fetch_configuration(cli_args: crate::cli::CliArgs) -> Result<JsValue, JsValue> {
        let cfg = HoprdConfig::from_cli_args(cli_args, false);
        ok_or_jserr!(serde_wasm_bindgen::to_value(&cfg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Args, Command, FromArgMatches};
    use std::io::{Read, Write};
    use tempfile::NamedTempFile;

    pub fn example_cfg() -> HoprdConfig {
        HoprdConfig {
            host: Host {
                ip: "127.0.0.1".to_owned(),
                port: 47462,
            },
            identity: Identity {
                file: "identity".to_string(),
                password: "".to_owned(),
                private_key: "".to_owned(),
            },
            strategy: Strategy {
                name: None,
                max_auto_channels: None,
                auto_redeem_tickets: true,
            },
            db: Db {
                data: "/tmp/db".to_owned(),
                init: false,
                force_init: false,
            },
            api: Api {
                enabled: false,
                auth: Auth::None,
                host: Host {
                    ip: "127.0.0.1".to_string(),
                    port: 1233,
                },
            },
            heartbeat: Heartbeat {
                interval: 0,
                threshold: 0,
                variance: 0,
            },
            network: Network {
                announce: false,
                allow_local_node_connections: false,
                allow_private_node_connections: false,
                max_parallel_connections: 0,
                network_quality_threshold: 0.0,
            },
            healthcheck: HealthCheck {
                enabled: false,
                host: "127.0.0.1".to_string(),
                port: 0,
            },
            environment: "testing".to_string(),
            chain: Chain {
                provider: None,
                check_unrealized_balance: true,
                on_chain_confirmations: 0,
            },
            test: Testing {
                announce_local_addresses: false,
                prefer_local_addresses: false,
                use_weak_crypto: false,
                no_direct_connections: false,
                no_webrtc_upgrade: false,
                local_mode_stun: false,
            },
        }
    }

    const DEFAULT_YAML: &'static str = r#"host:
  ip: 127.0.0.1
  port: 47462
identity:
  file: identity
  password: ''
  private_key: ''
db:
  data: /tmp/db
  init: false
  force_init: false
api:
  enabled: false
  auth: None
  host:
    ip: 127.0.0.1
    port: 1233
strategy:
  name: null
  max_auto_channels: null
  auto_redeem_tickets: true
heartbeat:
  interval: 0
  threshold: 0
  variance: 0
network:
  announce: false
  allow_local_node_connections: false
  allow_private_node_connections: false
  max_parallel_connections: 0
  network_quality_threshold: 0.0
healthcheck:
  enabled: false
  host: 127.0.0.1
  port: 0
environment: testing
chain:
  provider: null
  check_unrealized_balance: true
  on_chain_confirmations: 0
test:
  announce_local_addresses: false
  prefer_local_addresses: false
  use_weak_crypto: false
  no_direct_connections: false
  no_webrtc_upgrade: false
  local_mode_stun: false
"#;

    #[test]
    fn test_config_should_be_serializable_into_string() -> Result<(), Box<dyn std::error::Error>> {
        let cfg = example_cfg();

        let yaml = serde_yaml::to_string(&cfg)?;
        assert_eq!(yaml, DEFAULT_YAML);

        Ok(())
    }

    #[test]
    fn test_config_should_be_deserializable_from_a_string_in_a_file() -> Result<(), Box<dyn std::error::Error>> {
        let mut config_file = NamedTempFile::new()?;
        let mut prepared_config_file = config_file.reopen()?;

        let cfg = example_cfg();
        let yaml = serde_yaml::to_string(&cfg)?;
        config_file.write_all(yaml.as_bytes())?;

        let mut buf = String::new();
        prepared_config_file.read_to_string(&mut buf)?;
        let deserialized_cfg: HoprdConfig = serde_yaml::from_str(&buf)?;

        assert_eq!(deserialized_cfg, cfg);

        Ok(())
    }

    #[test]
    fn test_config_is_extractable_from_the_cli_arguments() -> Result<(), Box<dyn std::error::Error>> {
        let pwnd = "rpc://pawned!";

        let mut config_file = NamedTempFile::new()?;

        let mut cfg = example_cfg();
        cfg.chain.provider = Some(pwnd.to_owned());

        let yaml = serde_yaml::to_string(&cfg)?;
        config_file.write_all(yaml.as_bytes())?;
        let cfg_file_path = config_file.path().to_str().unwrap().to_string();

        let cli_args = vec!["hoprd", "--configurationFilePath", cfg_file_path.as_str()];

        let mut cmd = Command::new("hoprd").version("0.0.0");
        cmd = crate::cli::CliArgs::augment_args(cmd);
        let derived_matches = cmd.try_get_matches_from(cli_args).map_err(|e| e.to_string())?;
        let args = crate::cli::CliArgs::from_arg_matches(&derived_matches)?;

        // skipping validation
        let cfg = HoprdConfig::from_cli_args(args, true);

        assert!(cfg.is_ok());

        let cfg = cfg.unwrap();

        assert_eq!(cfg.chain.provider, Some(pwnd.to_owned()));

        Ok(())
    }
}
