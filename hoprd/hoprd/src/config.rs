use std::str::FromStr;

use proc_macro_regex::regex;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use hopr_lib::{config::HoprLibConfig, Address, HostConfig, ProtocolsConfig, Strategy, Strategy::AutoRedeeming};
use hoprd_api::config::{Api, Auth};
use hoprd_inbox::config::MessageInboxConfiguration;

pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 9091;

pub const DEFAULT_SAFE_TRANSACTION_SERVICE_PROVIDER: &str = "https://safe-transaction.prod.hoprtech.net/";

fn validate_file_path(s: &str) -> Result<(), ValidationError> {
    if std::path::Path::new(s).is_file() {
        Ok(())
    } else {
        Err(ValidationError::new(
            "Invalid file path specified, the file does not exist or is not a file",
        ))
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

fn validate_optional_private_key(s: &str) -> Result<(), ValidationError> {
    validate_private_key(s)
}

#[derive(Default, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Identity {
    #[validate(custom = "validate_file_path")]
    #[serde(default)]
    pub file: String,
    #[validate(custom = "validate_password")]
    #[serde(default)]
    pub password: String,
    #[validate(custom = "validate_optional_private_key")]
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

/// The main configuration object of the entire node.
///
/// The configuration is composed of individual configuration of corresponding
/// component configuration objects.
///
/// An always up-to-date config YAML example can be found in [example_cfg.yaml](https://github.com/hoprnet/hoprnet/tree/master/hoprd/example_cfg.yaml)
/// which is always in the root of this crate.
///
#[derive(Debug, Default, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct HoprdConfig {
    /// Configuration related to hopr functionality
    #[validate]
    #[serde(default)]
    pub hopr: HoprLibConfig,
    /// Configuration regarding the identity of the node
    #[validate]
    #[serde(default)]
    pub identity: Identity,
    /// Configuration of the underlying database engine
    #[validate]
    #[serde(default)]
    pub inbox: MessageInboxConfiguration,
    /// Configuration relevant for the API of the node
    #[validate]
    #[serde(default)]
    pub api: Api,
}

impl From<HoprdConfig> for HoprLibConfig {
    fn from(val: HoprdConfig) -> HoprLibConfig {
        val.hopr
    }
}

use hopr_platform::file::native::read_to_string;

use log::debug;

use crate::errors::HoprdError;

impl HoprdConfig {
    pub fn from_cli_args(cli_args: crate::cli::CliArgs, skip_validation: bool) -> crate::errors::Result<HoprdConfig> {
        let mut cfg: HoprdConfig = if let Some(cfg_path) = cli_args.configuration_file_path {
            debug!("fetching configuration from file {cfg_path}");
            let yaml_configuration =
                read_to_string(cfg_path.as_str()).map_err(|e| crate::errors::HoprdError::ConfigError(e.to_string()))?;
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

        // inbox
        if let Some(x) = cli_args.inbox_capacity {
            cfg.inbox.capacity = x;
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

        // heartbeat
        if let Some(x) = cli_args.heartbeat_interval {
            cfg.hopr.heartbeat.interval = std::time::Duration::from_secs(x)
        };
        if let Some(x) = cli_args.heartbeat_threshold {
            cfg.hopr.heartbeat.threshold = std::time::Duration::from_secs(x)
        };
        if let Some(x) = cli_args.heartbeat_variance {
            cfg.hopr.heartbeat.variance = std::time::Duration::from_secs(x)
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

        // TODO: strategy configuration from the CLI should be removed in 3.0!

        // strategy
        if let Some(x) = cli_args.default_strategy.and_then(|s| Strategy::from_str(&s).ok()) {
            // Clear all the default strategies and just use the given one
            cfg.hopr.strategy.strategies = vec![x];
        }

        // Add auto-redeeming strategy if not already there
        if cli_args.auto_redeem_tickets == 0
            && !cfg
                .hopr
                .strategy
                .strategies
                .iter()
                .any(|s| matches!(s, AutoRedeeming(_)))
        {
            cfg.hopr.strategy.strategies.push(AutoRedeeming(Default::default()));
        }

        // chain
        if cli_args.announce > 0 {
            cfg.hopr.chain.announce = true;
        }
        if let Some(network) = cli_args.network {
            cfg.hopr.chain.network = network;
        }

        if let Some(protocol_config) = cli_args.protocol_config_path {
            cfg.hopr.chain.protocols = ProtocolsConfig::from_str(
                &hopr_platform::file::native::read_to_string(&protocol_config)
                    .map_err(|e| crate::errors::HoprdError::ConfigError(e.to_string()))?,
            )
            .map_err(crate::errors::HoprdError::ConfigError)?;
        }

        //   TODO: custom provider is redundant with the introduction of protocol-config.json
        if let Some(x) = cli_args.provider {
            cfg.hopr.chain.provider = Some(x)
        };
        if cli_args.check_unrealized_balance == 0 {
            cfg.hopr.chain.check_unrealized_balance = true;
        }

        // safe module
        if let Some(x) = cli_args.safe_transaction_service_provider {
            cfg.hopr.safe_module.safe_transaction_service_provider = x
        };
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
            Ok(cfg)
        } else {
            if !cfg
                .hopr
                .chain
                .protocols
                .supported_networks()
                .iter()
                .any(|network| network == &cfg.hopr.chain.network)
            {
                return Err(crate::errors::HoprdError::ValidationError(format!(
                    "The specified network '{}' is not listed as supported ({:?})",
                    cfg.hopr.chain.network,
                    cfg.hopr.chain.protocols.supported_networks()
                )));
            }

            match cfg.validate() {
                Ok(_) => Ok(cfg),
                Err(e) => Err(crate::errors::HoprdError::ValidationError(e.to_string())),
            }
        }
    }

    pub fn as_redacted_string(&self) -> crate::errors::Result<String> {
        let mut redacted_cfg = self.clone();

        // redacting sensitive information
        match &mut redacted_cfg.api.auth {
            Auth::None => {}
            Auth::Token(_) => redacted_cfg.api.auth = Auth::Token("<REDACTED>".to_owned()),
        }
        if redacted_cfg.identity.private_key.is_some() {
            redacted_cfg.identity.private_key = Some("<REDACTED>".to_owned());
        }

        if redacted_cfg.identity.private_key.is_some() {
            redacted_cfg.identity.private_key = Some("<REDACTED>".to_owned());
        }
        redacted_cfg.identity.password = "<REDACTED>".to_owned();

        serde_json::to_string(&redacted_cfg).map_err(|e| crate::errors::HoprdError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Args, Command, FromArgMatches};
    use hopr_lib::HostType;
    use std::io::{Read, Write};
    use tempfile::NamedTempFile;

    pub fn example_cfg() -> HoprdConfig {
        let chain = hopr_lib::config::Chain {
            protocols: hopr_lib::ProtocolsConfig::from_str(
                r#"
                    {
                        "networks": {
                          "anvil-localhost": {
                            "chain": "anvil",
                            "environment_type": "local",
                            "version_range": "*",
                            "indexer_start_block_number": 5,
                            "addresses": {
                              "network_registry": "0x3Aa5ebB10DC797CAC828524e59A333d0A371443c",
                              "network_registry_proxy": "0x68B1D87F95878fE05B998F19b66F4baba5De1aed",
                              "channels": "0x9A9f2CCfdE556A7E9Ff0848998Aa4a0CFD8863AE",
                              "token": "0x9A676e781A523b5d0C0e43731313A708CB607508",
                              "module_implementation": "0xA51c1fc2f0D1a1b8494Ed1FE312d7C3a78Ed91C0",
                              "node_safe_registry": "0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82",
                              "ticket_price_oracle": "0x7a2088a1bFc9d81c55368AE168C2C02570cB814F",
                              "announcements": "0x09635F643e140090A9A8Dcd712eD6285858ceBef",
                              "node_stake_v2_factory": "0xB7f8BC63BbcaD18155201308C8f3540b07f84F5e"
                            },
                            "confirmations": 2,
                            "tags": [],
                          }
                        },
                        "chains": {
                          "anvil": {
                            "description": "Local Ethereum node, akin to Ganache, Hardhat chain",
                            "chain_id": 31337,
                            "live": false,
                            "max_fee_per_gas": "1 gwei",
                            "max_priority_fee_per_gas": "0.2 gwei",
                            "default_provider": "http://127.0.0.1:8545/",
                            "native_token_name": "ETH",
                            "hopr_token_name": "wxHOPR",
                            "block_time": 5000,
                            "tx_polling_interval": 1000,
                            "max_block_range": 200,
                            "tags": [],
                            "etherscan_api_url": null
                          }
                        }
                      }
                    "#,
            )
            .expect("protocol config should be valid"),
            ..hopr_lib::config::Chain::default()
        };

        let db = hopr_lib::config::Db {
            data: "/app/db".to_owned(),
            ..hopr_lib::config::Db::default()
        };

        let safe_module = hopr_lib::config::SafeModule {
            safe_transaction_service_provider: "https:://provider.com/".to_owned(),
            safe_address: Address::from_str("0x0000000000000000000000000000000000000000").unwrap(),
            module_address: Address::from_str("0x0000000000000000000000000000000000000000").unwrap(),
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

        HoprdConfig {
            hopr: HoprLibConfig {
                host,
                db,
                chain,
                safe_module,
                ..HoprLibConfig::default()
            },
            identity,
            ..HoprdConfig::default()
        }
    }

    #[test]
    fn test_config_should_be_serializable_into_string() -> Result<(), Box<dyn std::error::Error>> {
        let cfg = example_cfg();

        let from_yaml: HoprdConfig = serde_yaml::from_str(include_str!("../example_cfg.yaml"))?;

        assert_eq!(cfg, from_yaml);

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
