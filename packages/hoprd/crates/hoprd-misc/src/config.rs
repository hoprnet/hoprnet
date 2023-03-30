use proc_macro_regex::regex;
use utils_log::error;

use serde::{Serialize, Deserialize};
use validator::{Validate, ValidationError};

use core_misc::constants::{
    DEFAULT_HEARTBEAT_INTERVAL, DEFAULT_HEARTBEAT_INTERVAL_VARIANCE, DEFAULT_HEARTBEAT_THRESHOLD,
    DEFAULT_NETWORK_QUALITY_THRESHOLD, DEFAULT_MAX_PARALLEL_CONNECTIONS, DEFAULT_MAX_PARALLEL_CONNECTION_PUBLIC_RELAY,
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
        error!("Validation failed: '{}' is not a valid IPv4", s);
        Err(ValidationError::new("Invalid IPv4 address provided"))
    }
}

fn validate_api_token(token: Option<&str>) -> Result<(), ValidationError> {
    // TODO: should be only alphanumeric?
    if token.is_some() && token.unwrap().len() < MINIMAL_API_TOKEN_LENGTH {
        Err(ValidationError::new("The validation token is too short"))
    } else {
        Ok(())
    }
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Host {
    #[validate(custom = "validate_ipv4_address")]
    pub ip: String,
    pub port: u16,
}

fn parse_host(s: &str) -> Result<Host, String> {
    if !validator::validate_ip_v4(s) {
        return Err(format!(
            "Given string {} is not a valid host, Example: {}:{}",
            s,
            DEFAULT_HOST.to_string(),
            DEFAULT_PORT.to_string()
        ));
    }

    Host::from_ipv4_host_string(s)
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


fn parse_api_token(mut s: &str) -> Result<String, String> {
    if s.len() < MINIMAL_API_TOKEN_LENGTH {
        return Err(format!(
            "Length of API token is too short, minimally required {} but given {}",
            MINIMAL_API_TOKEN_LENGTH.to_string(),
            s.len()
        ));
    }

    match (s.starts_with("'"), s.ends_with("'")) {
        (true, true) => {
            s = s.strip_prefix("'").unwrap();
            s = s.strip_suffix("'").unwrap();

            Ok(s.into())
        }
        (true, false) => Err(format!("Found leading quote but no trailing quote")),
        (false, true) => Err(format!("Found trailing quote but no leading quote")),
        (false, false) => Ok(s.into()),
    }
}


use clap::builder::{PossibleValuesParser, ValueParser};
use clap::{Arg, ArgAction, ArgMatches, Args, Command, FromArgMatches as _};

#[command(about = "HOPRd2")]
#[wasm_bindgen_if(getter_with_clone)]
#[derive(Args,Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct TestStruct {
    #[arg(
    long = "dryRun",
    help = "List all the options used to run the HOPR node, but quit instead of starting",
    env = "HOPRD_DRY_RUN",
    default_value_t = false,
    action = ArgAction::SetTrue
    )]
    pub dry_run: bool,

    #[clap(flatten)]
    pub xxx: AbsTestStruct,

    #[clap(flatten)]
    pub yyy: AbsTestStruct
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Args,Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct AbsTestStruct {
    #[arg(
    long = "xxx",
    help = "List all the options used to run the HOPR node, but quit instead of starting",
    env = "HOPRD_DRY_RUN_xxx",
    default_value_t = false,
    action = ArgAction::SetTrue
    )]
    pub xxx: bool,
}


#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Auth {
    None,
    Token,       // To change into proper type string once wasm_bindgen disappears
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Api {
    pub enabled: bool,
    pub auth: Auth,
    pub token: Option<String>,
    pub host: Host,
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct HealthCheck {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Heartbeat {
    pub interval: u32,
    pub threshold: u32,
    pub variance: u32,
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Network {
    pub announce: bool,
    pub heartbeat: Heartbeat,
    pub allow_local_node_connections: bool,
    pub allow_private_node_connections: bool,
    pub max_parallel_connections: u32,
    pub network_quality_threshold: f32,
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Chain {
    pub provider: Option<String>,
    pub check_unrealized_balance: bool,
    pub on_chain_confirmations: u32,
}


#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Strategy {
    pub name: Option<String>,
    pub max_auto_channels: Option<u32>,
    pub auto_redeem_tickets: bool,
}


#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Identity {
    pub file: String,
    // path
    pub password: Option<String>,
    pub private_key: Option<Box<[u8]>>,
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Db {
    /// Path to the directory containing the database
    pub data: String,
    pub init: bool,
    pub force_init: bool,
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Testing {
    pub announce_local_addresses: bool,
    pub prefer_local_addresses: bool,
    pub use_weak_crypto: bool,
    pub no_direct_connections: bool,
    pub no_webrtc_upgrade: bool,
    pub local_mode_stun: bool,
}


#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct HoprdConfig {
    pub host: Host,
    pub identity: Identity,
    pub db: Db,
    pub api: Api,
    pub strategy: Strategy,
    pub network: Network,
    pub healthcheck: HealthCheck,
    pub environment: String,
    pub chain: Chain,

    pub test: Testing,

    // arguments that are not options, but rather a config behavior
    pub dry_run: bool,
}

impl From<crate::cli::CliArgs> for HoprdConfig {
    fn from(cli_args: crate::cli::CliArgs) -> Self {


        Self {
            host: cli_args.host,
            identity: Identity {
                file: cli_args.identity,
                password: cli_args.password,
                private_key: cli_args.private_key,
            },
            strategy: Strategy {
                name: cli_args.default_strategy,
                max_auto_channels: cli_args.max_auto_channels,
                auto_redeem_tickets: cli_args.auto_redeem_tickets,
            },
            db: Db {
                data: cli_args.data,
                init: cli_args.init,
                force_init: cli_args.force_init,
            },
            api: Api {
                enabled: cli_args.api,
                auth: if cli_args.disable_api_authentication {Auth::None} else {Auth::Token},
                token: cli_args.api_token,
                host: Host { ip: cli_args.api_host, port: cli_args.api_port },
            },
            network: Network {
                announce: cli_args.announce,
                heartbeat: Heartbeat {
                    interval: cli_args.heartbeat_interval,
                    threshold: cli_args.heartbeat_threshold,
                    variance: cli_args.heartbeat_variance,
                },
                allow_local_node_connections: cli_args.allow_local_node_connections,
                allow_private_node_connections: cli_args.allow_private_node_connections,
                max_parallel_connections: cli_args.max_parallel_connections,
                network_quality_threshold: cli_args.network_quality_threshold,
            },
            healthcheck: HealthCheck {
                enabled: cli_args.health_check,
                host: cli_args.health_check_host,
                port: cli_args.health_check_port,
            },
            environment: cli_args.environment,
            chain: Chain {
                provider: cli_args.provider,
                check_unrealized_balance: cli_args.check_unrealized_balance,
                on_chain_confirmations: cli_args.on_chain_confirmations,
            },
            test: Testing {     // TODO: limit these to a debug build?
                announce_local_addresses: cli_args.test_announce_local_addresses,
                prefer_local_addresses: cli_args.test_prefer_local_addresses,
                use_weak_crypto: cli_args.test_use_weak_crypto,
                no_direct_connections: cli_args.test_no_direct_connections,
                no_webrtc_upgrade: cli_args.test_no_webrtc_upgrade,
                local_mode_stun: cli_args.test_local_mode_stun,
            },
            dry_run: cli_args.dry_run,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::{self, Write, Read};

    pub fn example_cfg() -> HoprdConfig {
        HoprdConfig {
            host: Host {
                ip: "127.0.0.1".to_owned(),
                port: 47462,
            },
            identity: Identity {
                file: "identity".to_string(),
                password: None,
                private_key: None,
            },
            strategy: Strategy {
                name: None,
                max_auto_channels: None,
                auto_redeem_tickets: false,
            },
            db: Db {
                data: "/tmp/db".to_owned(),
                init: false,
                force_init: false,
            },
            api: Api {
                enabled: false,
                auth: Auth::None,
                token: None,
                host: Host { ip: "127.0.0.1".to_string(), port: 1233 },
            },
            network: Network {
                announce: false,
                heartbeat: Heartbeat {
                    interval: 0,
                    threshold: 0,
                    variance: 0,
                },
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
                check_unrealized_balance: false,
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
            dry_run: false,
        }
    }

    const DEFAULT_YAML: &'static str =
        r#"host:
  ip: 127.0.0.1
  port: 47462
identity:
  file: identity
  password: null
  private_key: null
db:
  data: /tmp/db
  init: false
  force_init: false
api:
  enabled: false
  auth: None
  token: null
  host:
    ip: 127.0.0.1
    port: 1233
strategy:
  name: null
  max_auto_channels: null
  auto_redeem_tickets: false
network:
  announce: false
  heartbeat:
    interval: 0
    threshold: 0
    variance: 0
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
  check_unrealized_balance: false
  on_chain_confirmations: 0
test:
  announce_local_addresses: false
  prefer_local_addresses: false
  use_weak_crypto: false
  no_direct_connections: false
  no_webrtc_upgrade: false
  local_mode_stun: false
dry_run: false
"#;

    #[test]
    fn test_config_should_be_serializable_into_string() -> Result<(), Box<dyn std::error::Error>> {
        let cfg = example_cfg();

        let yaml = serde_yaml::to_string(&cfg)?;
        assert_eq!(yaml, DEFAULT_YAML);

        Ok(())
    }

    #[test]
    fn test_TestStruct() -> Result<(), Box<dyn std::error::Error>> {
        let cfg = TestStruct{
            dry_run: false,
            xxx: AbsTestStruct { xxx: false },
            yyy: AbsTestStruct { xxx: true },
        };

        let yaml = serde_yaml::to_string(&cfg)?;
        assert_eq!(yaml, "");

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
}
