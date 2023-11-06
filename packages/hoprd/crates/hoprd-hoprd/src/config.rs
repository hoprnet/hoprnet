use std::str::FromStr;

use hoprd_inbox::config::MessageInboxConfiguration;
use proc_macro_regex::regex;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use core_strategy::{Strategy, Strategy::AutoRedeeming};
use core_transport::config::HostConfig;
use utils_types::primitives::Address;

use hopr_lib::config::HoprLibConfig;

pub const DEFAULT_API_HOST: &str = "127.0.0.1";
pub const DEFAULT_API_PORT: u16 = 3001;

pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 9091;

pub const DEFAULT_HEALTH_CHECK_HOST: &str = "127.0.0.1";
pub const DEFAULT_HEALTH_CHECK_PORT: u16 = 8080;

pub const DEFAULT_SAFE_TRANSACTION_SERVICE_PROVIDER: &str = "https://safe-transaction.stage.hoprtech.net/";

pub const MINIMAL_API_TOKEN_LENGTH: usize = 8;

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Auth {
    None,
    Token(String),
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[derive(Debug, Validate, Serialize, Deserialize, Clone, PartialEq)]
pub struct Api {
    pub enable: bool,
    /// Auth enum holding the API auth configuration
    ///
    /// The auth enum cannot be made public due to incompatibility with the wasm_bindgen.
    #[validate(custom = "validate_api_auth")]
    auth: Auth,
    #[validate]
    pub host: HostConfig,
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
            enable: false,
            auth: Auth::Token("".to_owned()),
            host: HostConfig::from_str(format!("{DEFAULT_API_HOST}:{DEFAULT_API_PORT}").as_str()).unwrap(),
        }
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct HealthCheck {
    pub enable: bool,
    pub host: String,
    pub port: u16,
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self {
            enable: false,
            host: DEFAULT_HEALTH_CHECK_HOST.to_string(),
            port: DEFAULT_HEALTH_CHECK_PORT,
        }
    }
}

/// Does not work in the WASM environment
#[allow(dead_code)]
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

regex!(is_private_key "^(0[xX])?[a-fA-F0-9]{128}$");

pub(crate) fn validate_private_key(s: &str) -> Result<(), ValidationError> {
    if is_private_key(s) {
        Ok(())
    } else {
        Err(ValidationError::new("No valid private key could be found"))
    }
}

fn validate_optional_private_key(s: &String) -> Result<(), ValidationError> {
    validate_private_key(s.as_str())
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[derive(Default, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Identity {
    pub file: String,
    #[validate(custom = "validate_password")]
    pub password: String,
    #[validate(custom = "validate_optional_private_key")]
    pub private_key: Option<String>,
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

/// Does not work in the WASM environment
#[allow(dead_code)]
fn validate_directory_path(s: &str) -> Result<(), ValidationError> {
    if std::path::Path::new(s).is_dir() {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid directory path specified"))
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[derive(Debug, Default, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Testing {
    pub use_weak_crypto: bool,
}

/// The main configuration object of the entire node.
///
/// The configuration is composed of individual configuration of corresponding
/// component configuration objects.
///
/// An always up-to-date config YAML example can be found in [`EXAMPLE_YAML`].
///
/// The default configuration as it would appear from the configuration YAML file.
/// ```yaml
/// ---
///
/// host:
///   address: !IPv4 127.0.0.1
///   port: 47462
/// identity:
///   file: identity
///   password: ''
///   private_key: ''
/// db:
///   data: /tmp/db
///   initialize: false
///   force_initialize: false
/// inbox:
///   capacity: 512
///   max_age: 900
///   excluded_tags:
///   - 0
/// api:
///   enable: true
///   auth: !Token sdjkghsfg
/// host:
///   address: !IPv4 127.0.0.1
///   port: 1233
/// strategy:
///   on_fail_continue: true
///   allow_recursive: true
///   strategies: []
/// heartbeat:
///   variance: 0
///   interval: 0
///   threshold: 0
/// network_options:
///   min_delay: 1
///   max_delay: 300
///   quality_bad_threshold: 0.2
///   quality_offline_threshold: 0.0
///   quality_step: 0.1
///   ignore_timeframe: 600
///   backoff_exponent: 1.5
///   backoff_min: 2.0
///   backoff_max: 300.0
/// healthcheck:
///   enable: false
///   host: 127.0.0.1
///   port: 0
/// protocol:
///   ack:
///     timeout: 15
///   heartbeat:
///     timeout: 15
///   msg:
///     timeout: 15
///   ticket_aggregation:
///     timeout: 15
/// network: anvil-localhost
/// chain:
///   announce: false
///   provider: null
///   check_unrealized_balance: true
/// safe_module:
///   safe_transaction_service_provider: null
///   safe_address: null
///   module_address: null
/// test:
///   announce_local_addresses: false
///   prefer_local_addresses: false
///   use_weak_crypto: false
/// ```
///
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct HoprdConfig {
    /// Configuration related to hopr functionality
    #[validate]
    pub hopr: HoprLibConfig,
    /// Configuration regarding the identity of the node
    #[validate]
    pub identity: Identity,
    /// Configuration of the underlying database engine
    #[validate]
    pub inbox: MessageInboxConfiguration,
    /// Configuration relevant for the API of the node
    #[validate]
    pub api: Api,
    /// Testing configurations
    #[validate]
    pub test: Testing,
}

impl Default for HoprdConfig {
    fn default() -> Self {
        Self {
            hopr: HoprLibConfig::default(),
            identity: Identity::default(),
            inbox: MessageInboxConfiguration::default(),
            api: Api::default(),
            test: Testing::default(),
        }
    }
}

impl Into<HoprLibConfig> for HoprdConfig {
    fn into(self) -> HoprLibConfig {
        self.hopr
    }
}

#[cfg(any(not(feature = "wasm"), test))]
use real_base::file::native::read_to_string;

#[cfg(all(feature = "wasm", not(test)))]
use real_base::file::wasm::read_to_string;

use utils_log::debug;

use crate::errors::HoprdConfigError;

impl HoprdConfig {
    pub fn from_cli_args(cli_args: crate::cli::CliArgs, skip_validation: bool) -> crate::errors::Result<HoprdConfig> {
        let mut cfg: HoprdConfig = if let Some(cfg_path) = cli_args.configuration_file_path {
            debug!("fetching configuration from file {cfg_path}");
            let yaml_configuration = read_to_string(cfg_path.as_str())
                .map_err(|e| crate::errors::HoprdConfigError::FileError(e.to_string()))?;
            serde_yaml::from_str(&yaml_configuration)
                .map_err(|e| crate::errors::HoprdConfigError::SerializationError(e.to_string()))?
        } else {
            debug!("loading default configuration");
            HoprdConfig::default()
        };

        // host
        if let Some(x) = cli_args.host {
            cfg.hopr.host = x
        };

        // hopr.transport
        cfg.hopr.transport.announce_local_addresses = cli_args.test_announce_local_addresses;
        cfg.hopr.transport.prefer_local_addresses = cli_args.test_prefer_local_addresses;

        // db
        if let Some(data) = cli_args.data {
            cfg.hopr.db.data = data
        }
        cfg.hopr.db.initialize = cli_args.init;
        cfg.hopr.db.force_initialize = cli_args.force_init;

        // inbox
        if let Some(x) = cli_args.inbox_capacity {
            cfg.inbox.capacity = x;
        }

        // api
        cfg.api.enable = cli_args.api;
        if cli_args.disable_api_authentication {
            if &cfg.api.auth != &Auth::None {
                cfg.api.auth = Auth::None;
            }
        };
        if let Some(x) = cli_args.api_token {
            cfg.api.auth = Auth::Token(x);
        };
        if let Some(x) = cli_args.api_host {
            cfg.api.host = HostConfig::from_str(format!("{}:{}", x.as_str(), DEFAULT_API_PORT).as_str())
                .map_err(|e| crate::errors::HoprdConfigError::ValidationError(e))?;
        }
        if let Some(x) = cli_args.api_port {
            cfg.api.host.port = x
        };

        // heartbeat
        if let Some(x) = cli_args.heartbeat_interval {
            cfg.hopr.heartbeat.interval = x
        };
        if let Some(x) = cli_args.heartbeat_threshold {
            cfg.hopr.heartbeat.threshold = x
        };
        if let Some(x) = cli_args.heartbeat_variance {
            cfg.hopr.heartbeat.variance = x
        };

        // network options
        if let Some(x) = cli_args.network_quality_threshold {
            cfg.hopr.network_options.quality_offline_threshold = x
        };

        // identity
        if let Some(identity) = cli_args.identity {
            cfg.identity.file = identity;
        }
        if let Some(x) = cli_args.password {
            cfg.identity.password = x
        };
        if let Some(x) = cli_args.private_key {
            cfg.identity.private_key = Some(x)
        };

        // TODO: resolve CLI configuration of strategies

        // strategy
        if let Some(x) = cli_args.default_strategy.and_then(|s| Strategy::from_str(&s).ok()) {
            cfg.hopr.strategy.get_strategies().push(x);
        }

        if cli_args.auto_redeem_tickets {
            cfg.hopr
                .strategy
                .get_strategies()
                .push(AutoRedeeming(Default::default()));
        }

        // chain
        cfg.hopr.chain.announce = cli_args.announce;
        if let Some(network) = cli_args.network {
            cfg.hopr.chain.network = network;
        }

        if let Some(x) = cli_args.provider {
            cfg.hopr.chain.provider = Some(x)
        };
        cfg.hopr.chain.check_unrealized_balance = cli_args.check_unrealized_balance;

        // safe module
        if let Some(x) = cli_args.safe_transaction_service_provider {
            cfg.hopr.safe_module.safe_transaction_service_provider = x
        };
        if let Some(x) = cli_args.safe_address {
            cfg.hopr.safe_module.safe_address =
                Address::from_str(&x).map_err(|e| HoprdConfigError::ValidationError(e.to_string()))?
        };
        if let Some(x) = cli_args.module_address {
            cfg.hopr.safe_module.module_address =
                Address::from_str(&x).map_err(|e| HoprdConfigError::ValidationError(e.to_string()))?
        };

        // test
        cfg.test.use_weak_crypto = cli_args.test_use_weak_crypto;

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
    use hopr_lib::config::HoprLibConfig;
    use utils_misc::ok_or_jserr;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = os)]
        pub fn homedir() -> String;
    }

    #[wasm_bindgen]
    impl HoprdConfig {
        #[wasm_bindgen(constructor)]
        pub fn _new() -> Self {
            Self::default()
        }

        #[wasm_bindgen]
        pub fn as_redacted_string(&self) -> Result<String, JsValue> {
            let mut redacted_cfg = self.clone();

            // redacting sensitive information
            match &mut redacted_cfg.api.auth {
                crate::config::Auth::None => {}
                crate::config::Auth::Token(_) => {
                    redacted_cfg.api.auth = crate::config::Auth::Token("<REDACTED>".to_owned())
                }
            }
            if let Some(_) = redacted_cfg.identity.private_key {
                redacted_cfg.identity.private_key = Some("<REDACTED>".to_owned());
            }

            if let Some(_) = redacted_cfg.identity.private_key {
                redacted_cfg.identity.private_key = Some("<REDACTED>".to_owned());
            }
            redacted_cfg.identity.password = "<REDACTED>".to_owned();

            ok_or_jserr!(serde_json::to_string(&redacted_cfg))
        }
    }

    #[wasm_bindgen]
    pub fn fetch_configuration(cli_args: JsValue) -> Result<HoprdConfig, JsError> {
        let args: crate::cli::CliArgs =
            serde_wasm_bindgen::from_value(cli_args).map_err(|e| JsError::new(e.to_string().as_str()))?;
        let mut cfg = HoprdConfig::from_cli_args(args, false).map_err(|e| JsError::new(e.to_string().as_str()))?;

        // replace the ~ in the path for a home paths
        if cfg.hopr.db.data.starts_with("~") {
            cfg.hopr.db.data = homedir() + &cfg.hopr.db.data[1..];
        }
        if cfg.identity.file.starts_with("~") {
            cfg.identity.file = homedir() + &cfg.identity.file[1..];
        }

        Ok(cfg)
    }

    #[wasm_bindgen]
    pub fn to_hoprlib_config(cfg: &HoprdConfig) -> HoprLibConfig {
        cfg.clone().into()
    }
}

/// Used in the testing and documentation
pub const EXAMPLE_YAML: &'static str = r#"hopr:
  host:
    address: !IPv4 127.0.0.1
    port: 47462
  db:
    data: /tmp/db
    initialize: false
    force_initialize: false
  strategy:
    on_fail_continue: true
    allow_recursive: true
    finalize_channel_closure: false
    strategies: []
  heartbeat:
    variance: 0
    interval: 0
    threshold: 0
  network_options:
    min_delay: 1
    max_delay: 300
    quality_bad_threshold: 0.2
    quality_offline_threshold: 0.0
    quality_step: 0.1
    quality_avg_window_size: 25
    ignore_timeframe: 600
    backoff_exponent: 1.5
    backoff_min: 2.0
    backoff_max: 300.0
  transport:
    announce_local_addresses: false
    prefer_local_addresses: false
  protocol:
    ack:
      timeout: 15
    heartbeat:
      timeout: 15
    msg:
      timeout: 15
    ticket_aggregation:
      timeout: 15
  chain:
    announce: false
    network: testing
    provider: null
    check_unrealized_balance: true
  safe_module:
    safe_transaction_service_provider: https:://provider.com/
    safe_address: '0x0000000000000000000000000000000000000000'
    module_address: '0x0000000000000000000000000000000000000000'
identity:
  file: identity
  password: ''
  private_key: ''
inbox:
  capacity: 512
  max_age: 900
  excluded_tags:
  - 0
api:
  enable: false
  auth: None
  host:
    address: !IPv4 127.0.0.1
    port: 1233
test:
  use_weak_crypto: false
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Args, Command, FromArgMatches};
    use std::io::{Read, Write};
    use tempfile::NamedTempFile;

    pub fn example_cfg() -> HoprdConfig {
        HoprdConfig {
            hopr: HoprLibConfig {
                host: hopr_lib::config::HostConfig::from_str(format!("127.0.0.1:47462").as_str()).unwrap(),
                db: hopr_lib::config::Db {
                    data: "/tmp/db".to_owned(),
                    initialize: false,
                    force_initialize: false,
                },
                strategy: hopr_lib::config::StrategyConfig::default(),
                heartbeat: hopr_lib::config::HeartbeatConfig {
                    interval: 0,
                    threshold: 0,
                    variance: 0,
                },
                network_options: {
                    let mut c = hopr_lib::config::NetworkConfig::default();
                    c.quality_offline_threshold = 0.0;
                    c
                },
                transport: hopr_lib::config::TransportConfig::default(),
                protocol: hopr_lib::config::ProtocolConfig::default(),
                chain: hopr_lib::config::Chain {
                    announce: false,
                    network: "testing".to_string(),
                    provider: None,
                    check_unrealized_balance: true,
                },
                safe_module: hopr_lib::config::SafeModule {
                    safe_transaction_service_provider: "https:://provider.com/".to_owned(),
                    safe_address: Address::from_str("0x0000000000000000000000000000000000000000").unwrap(),
                    module_address: Address::from_str("0x0000000000000000000000000000000000000000").unwrap(),
                },
            },
            identity: Identity {
                file: "identity".to_string(),
                password: "".to_owned(),
                private_key: Some("".to_owned()),
            },
            inbox: MessageInboxConfiguration::default(),
            api: Api {
                enable: false,
                auth: Auth::None,
                host: hopr_lib::config::HostConfig::from_str(format!("127.0.0.1:1233").as_str()).unwrap(),
            },
            test: Testing { use_weak_crypto: false },
        }
    }

    #[test]
    fn test_config_should_be_serializable_into_string() -> Result<(), Box<dyn std::error::Error>> {
        let cfg = example_cfg();

        let yaml = serde_yaml::to_string(&cfg)?;
        assert_eq!(yaml, EXAMPLE_YAML);

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

    /// TODO: This test attempts to deserialize the data structure incorrectly in the native build
    /// (`confirmations`` are an extra field), as well as misses the native implementation for the
    /// version satisfies check
    #[test]
    #[ignore]
    fn test_config_is_extractable_from_the_cli_arguments() -> Result<(), Box<dyn std::error::Error>> {
        let pwnd = "rpc://pawned!";

        let mut config_file = NamedTempFile::new()?;

        let mut cfg = example_cfg();
        cfg.hopr.chain.provider = Some(pwnd.to_owned());

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

        assert_eq!(cfg.hopr.chain.provider, Some(pwnd.to_owned()));

        Ok(())
    }
}
