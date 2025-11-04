use std::{
    collections::HashSet,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    time::Duration,
};

use hopr_lib::{Address, HostConfig, HostType, config::HoprLibConfig};
use hoprd_api::config::{Api, Auth};
use proc_macro_regex::regex;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tracing::debug;
use validator::{Validate, ValidationError};

use crate::errors::HoprdError;

pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 9091;

pub const DEFAULT_SAFE_TRANSACTION_SERVICE_PROVIDER: &str = "https://safe-transaction.prod.hoprtech.net/";

// Validate that the path is a valid UTF-8 path.
//
// Also used to perform the identity file existence check on the
// specified path, which is now circumvented but could
// return in the future workflows of setting up a node.
fn validate_file_path(_s: &str) -> Result<(), ValidationError> {
    Ok(())

    // if std::path::Path::new(_s).is_file() {
    //     Ok(())
    // } else {
    //     Err(ValidationError::new(
    //         "Invalid file path specified, the file does not exist or is not a file",
    //     ))
    // }
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

fn validate_optional_private_key(s: &str) -> Result<(), ValidationError> {
    validate_private_key(s)
}

#[derive(Default, Serialize, Deserialize, Validate, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    #[validate(custom(function = "validate_file_path"))]
    #[serde(default)]
    pub file: String,
    #[validate(custom(function = "validate_password"))]
    #[serde(default)]
    pub password: String,
    #[validate(custom(function = "validate_optional_private_key"))]
    #[serde(default)]
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

/// The main configuration object of the entire node.
///
/// The configuration is composed of individual configuration of corresponding
/// component configuration objects.
///
/// An always up-to-date config YAML example can be found in [example_cfg.yaml](https://github.com/hoprnet/hoprnet/tree/master/hoprd/hoprd/example_cfg.yaml)
/// which is always in the root of this crate.
#[derive(Debug, Default, Serialize, Deserialize, Validate, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HoprdConfig {
    /// Configuration related to hopr functionality
    #[validate(nested)]
    #[serde(default)]
    pub hopr: HoprLibConfig,
    /// Configuration regarding the identity of the node
    #[validate(nested)]
    #[serde(default)]
    pub identity: Identity,
    /// Configuration of the underlying database engine
    #[validate(nested)]
    #[serde(default)]
    pub db: Db,
    /// Configuration relevant for the API of the node
    #[validate(nested)]
    #[serde(default)]
    pub api: Api,
    /// Configuration of the Session entry/exit node IP protocol forwarding.
    #[validate(nested)]
    #[serde(default)]
    pub session_ip_forwarding: SessionIpForwardingConfig,
    /// Blokli provider to connect to.
    #[validate(url)]
    pub provider: Option<String>,
    /// Configuration of underlying node behavior in the form strategies
    ///
    /// Strategies represent automatically executable behavior performed by
    /// the node given pre-configured triggers.
    #[validate(nested)]
    #[serde(default = "hopr_strategy::hopr_default_strategies")]
    #[default(hopr_strategy::hopr_default_strategies())]
    pub strategy: hopr_strategy::StrategyConfig,
}

impl From<HoprdConfig> for HoprLibConfig {
    fn from(val: HoprdConfig) -> HoprLibConfig {
        val.hopr
    }
}

impl HoprdConfig {
    pub fn from_cli_args(cli_args: crate::cli::CliArgs, skip_validation: bool) -> crate::errors::Result<HoprdConfig> {
        let mut cfg: HoprdConfig = if let Some(cfg_path) = cli_args.configuration_file_path {
            debug!(cfg_path, "fetching configuration from file");
            let yaml_configuration = std::fs::read_to_string(cfg_path.as_str())
                .map_err(|e| crate::errors::HoprdError::ConfigError(e.to_string()))?;
            serde_yaml::from_str(&yaml_configuration)
                .map_err(|e| crate::errors::HoprdError::SerializationError(e.to_string()))?
        } else {
            debug!("loading default configuration");
            HoprdConfig::default()
        };

        // host
        if let Some(x) = cli_args.host {
            cfg.hopr.host = x
        };

        // hopr.transport
        if cli_args.test_announce_local_addresses > 0 {
            cfg.hopr.transport.announce_local_addresses = true;
        }
        if cli_args.test_prefer_local_addresses > 0 {
            cfg.hopr.transport.prefer_local_addresses = true;
        }

        if let Some(host) = cli_args.default_session_listen_host {
            cfg.session_ip_forwarding.default_entry_listen_host = match host.address {
                HostType::IPv4(addr) => IpAddr::from_str(&addr)
                    .map(|ip| std::net::SocketAddr::new(ip, host.port))
                    .map_err(|_| HoprdError::ConfigError("invalid default session listen IP address".into())),
                HostType::Domain(_) => Err(HoprdError::ConfigError("default session listen must be an IP".into())),
            }?;
        }

        // db
        if let Some(data) = cli_args.data {
            cfg.hopr.db.data = data
        }
        if cli_args.init > 0 {
            cfg.hopr.db.initialize = true;
        }
        if cli_args.force_init > 0 {
            cfg.hopr.db.force_initialize = true;
        }

        // api
        if cli_args.api > 0 {
            cfg.api.enable = true;
        }
        if cli_args.disable_api_authentication > 0 && cfg.api.auth != Auth::None {
            cfg.api.auth = Auth::None;
        };
        if let Some(x) = cli_args.api_token {
            cfg.api.auth = Auth::Token(x);
        };
        if let Some(x) = cli_args.api_host {
            cfg.api.host =
                HostConfig::from_str(format!("{}:{}", x.as_str(), hoprd_api::config::DEFAULT_API_PORT).as_str())
                    .map_err(crate::errors::HoprdError::ValidationError)?;
        }
        if let Some(x) = cli_args.api_port {
            cfg.api.host.port = x
        };

        // probe
        if let Some(x) = cli_args.probe_recheck_threshold {
            cfg.hopr.probe.recheck_threshold = std::time::Duration::from_secs(x)
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

        // chain
        if cli_args.announce > 0 {
            cfg.hopr.publish = true;
        }

        if let Some(x) = cli_args.provider {
            cfg.provider = Some(x);
        }

        if let Some(x) = cli_args.safe_address {
            cfg.hopr.safe_module.safe_address =
                Address::from_str(&x).map_err(|e| HoprdError::ValidationError(e.to_string()))?
        };
        if let Some(x) = cli_args.module_address {
            cfg.hopr.safe_module.module_address =
                Address::from_str(&x).map_err(|e| HoprdError::ValidationError(e.to_string()))?
        };

        // additional updates
        let home_symbol = '~';
        if cfg.hopr.db.data.starts_with(home_symbol) {
            cfg.hopr.db.data = home::home_dir()
                .map(|h| h.as_path().display().to_string())
                .expect("home dir for a user must be specified")
                + &cfg.hopr.db.data[1..];
        }
        if cfg.identity.file.starts_with(home_symbol) {
            cfg.identity.file = home::home_dir()
                .map(|h| h.as_path().display().to_string())
                .expect("home dir for a user must be specified")
                + &cfg.identity.file[1..];
        }

        if skip_validation {
            return Ok(cfg)
        }

        match cfg.validate() {
            Ok(_) => Ok(cfg),
            Err(e) => Err(crate::errors::HoprdError::ValidationError(e.to_string())),
        }
    }

    pub fn as_redacted(&self) -> Self {
        let mut ret = self.clone();
        // redacting sensitive information
        match ret.api.auth {
            Auth::None => {}
            Auth::Token(_) => ret.api.auth = Auth::Token("<REDACTED>".to_owned()),
        }

        if ret.identity.private_key.is_some() {
            ret.identity.private_key = Some("<REDACTED>".to_owned());
        }

        "<REDACTED>".clone_into(&mut ret.identity.password);

        ret
    }

    pub fn as_redacted_string(&self) -> crate::errors::Result<String> {
        let redacted_cfg = self.as_redacted();
        serde_json::to_string(&redacted_cfg).map_err(|e| crate::errors::HoprdError::SerializationError(e.to_string()))
    }
}

fn default_target_retry_delay() -> Duration {
    Duration::from_secs(2)
}

fn default_entry_listen_host() -> SocketAddr {
    "127.0.0.1:0".parse().unwrap()
}

fn default_max_tcp_target_retries() -> u32 {
    10
}

fn just_true() -> bool {
    true
}

/// Configuration of the Exit node (see [`HoprServerIpForwardingReactor`](crate::exit::HoprServerIpForwardingReactor))
/// and the Entry node.
#[serde_as]
#[derive(
    Clone, Debug, Eq, PartialEq, smart_default::SmartDefault, serde::Deserialize, serde::Serialize, validator::Validate,
)]
pub struct SessionIpForwardingConfig {
    /// Controls whether allowlisting should be done via `target_allow_list`.
    /// If set to `false`, the node will act as an Exit node for any target.
    ///
    /// Defaults to `true`.
    #[serde(default = "just_true")]
    #[default(true)]
    pub use_target_allow_list: bool,

    /// Enforces only the given target addresses (after DNS resolution).
    ///
    /// This is used only if `use_target_allow_list` is set to `true`.
    /// If left empty (and `use_target_allow_list` is `true`), the node will not act as an Exit node.
    ///
    /// Defaults to empty.
    #[serde(default)]
    #[serde_as(as = "HashSet<serde_with::DisplayFromStr>")]
    pub target_allow_list: HashSet<SocketAddr>,

    /// Delay between retries in seconds to reach a TCP target.
    ///
    /// Defaults to 2 seconds.
    #[serde(default = "default_target_retry_delay")]
    #[default(default_target_retry_delay())]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub tcp_target_retry_delay: Duration,

    /// Maximum number of retries to reach a TCP target before giving up.
    ///
    /// Default is 10.
    #[serde(default = "default_max_tcp_target_retries")]
    #[default(default_max_tcp_target_retries())]
    #[validate(range(min = 1))]
    pub max_tcp_target_retries: u32,

    /// Specifies the default `listen_host` for Session listening sockets
    /// at an Entry node.
    #[serde(default = "default_entry_listen_host")]
    #[default(default_entry_listen_host())]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub default_entry_listen_host: SocketAddr,
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use anyhow::Context;
    use clap::{Args, Command, FromArgMatches};
    use hopr_lib::HostType;
    use tempfile::NamedTempFile;

    use super::*;

    pub fn example_cfg() -> anyhow::Result<HoprdConfig> {
        let db = hopr_lib::config::Db {
            data: "/app/db".to_owned(),
            ..hopr_lib::config::Db::default()
        };

        let safe_module = hopr_lib::config::SafeModule {
            safe_transaction_service_provider: "https:://provider.com/".to_owned(),
            safe_address: Address::from_str("0x0000000000000000000000000000000000000000")?,
            module_address: Address::from_str("0x0000000000000000000000000000000000000000")?,
        };

        let identity = Identity {
            file: "path/to/identity.file".to_string(),
            password: "change_me".to_owned(),
            private_key: None,
        };

        let host = HostConfig {
            address: HostType::IPv4("1.2.3.4".into()),
            port: 9091,
        };

        Ok(HoprdConfig {
            hopr: HoprLibConfig {
                host,
                db,
                safe_module,
                ..HoprLibConfig::default()
            },
            identity,
            ..HoprdConfig::default()
        })
    }

    #[test]
    fn test_config_should_be_serializable_into_string() -> Result<(), Box<dyn std::error::Error>> {
        let cfg = example_cfg()?;

        let from_yaml: HoprdConfig = serde_yaml::from_str(include_str!("../example_cfg.yaml"))?;

        assert_eq!(cfg, from_yaml);

        Ok(())
    }

    #[test]
    fn test_config_should_be_deserializable_from_a_string_in_a_file() -> Result<(), Box<dyn std::error::Error>> {
        let mut config_file = NamedTempFile::new()?;
        let mut prepared_config_file = config_file.reopen()?;

        let cfg = example_cfg()?;
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
    fn test_config_is_extractable_from_the_cli_arguments() -> anyhow::Result<()> {
        let pwnd = "rpc://pawned!";

        let mut config_file = NamedTempFile::new()?;

        let mut cfg = example_cfg()?;
        cfg.provider = Some(pwnd.to_owned());

        let yaml = serde_yaml::to_string(&cfg)?;
        config_file.write_all(yaml.as_bytes())?;
        let cfg_file_path = config_file
            .path()
            .to_str()
            .context("file path should have a string representation")?
            .to_string();

        let cli_args = vec!["hoprd", "--configurationFilePath", cfg_file_path.as_str()];

        let mut cmd = Command::new("hoprd").version("0.0.0");
        cmd = crate::cli::CliArgs::augment_args(cmd);
        let derived_matches = cmd.try_get_matches_from(cli_args)?;
        let args = crate::cli::CliArgs::from_arg_matches(&derived_matches)?;

        // skipping validation
        let cfg = HoprdConfig::from_cli_args(args, true);

        assert!(cfg.is_ok());

        let cfg = cfg?;

        assert_eq!(cfg.provider, Some(pwnd.to_owned()));

        Ok(())
    }
}
