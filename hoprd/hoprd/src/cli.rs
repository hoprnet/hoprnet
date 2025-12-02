use std::{net::IpAddr, str::FromStr};

use clap::{ArgAction, Parser, builder::ValueParser};
use hopr_chain_connector::Address;
use hopr_lib::config::{HostConfig, HostType, looks_like_domain};
use hoprd_api::config::Auth;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{config::HoprdConfig, errors::HoprdError};

pub const DEFAULT_API_HOST: &str = "localhost";
pub const DEFAULT_API_PORT: u16 = 3001;

pub const MINIMAL_API_TOKEN_LENGTH: usize = 8;

fn parse_host(s: &str) -> Result<HostConfig, String> {
    let host = s.split_once(':').map_or(s, |(h, _)| h);
    if !(validator::ValidateIp::validate_ipv4(&host) || looks_like_domain(host)) {
        return Err(format!(
            "Given string {s} is not a valid host, should have a format: <ip>:<port> or <domain>(:<port>)"
        ));
    }

    HostConfig::from_str(s)
}

fn parse_api_token(mut s: &str) -> Result<String, String> {
    if s.len() < MINIMAL_API_TOKEN_LENGTH {
        return Err(format!(
            "Length of API token is too short, minimally required {MINIMAL_API_TOKEN_LENGTH} but given {}",
            s.len()
        ));
    }

    match (s.starts_with('\''), s.ends_with('\'')) {
        (true, true) => {
            s = s.strip_prefix('\'').ok_or("failed to parse strip prefix part")?;
            s = s.strip_suffix('\'').ok_or("failed to parse strip suffix part")?;

            Ok(s.into())
        }
        (true, false) => Err("Found leading quote but no trailing quote".into()),
        (false, true) => Err("Found trailing quote but no leading quote".into()),
        (false, false) => Ok(s.into()),
    }
}

/// Takes all CLI arguments whose structure is known at compile-time.
/// Arguments whose structure, e.g., their default values depend on
/// file contents, need to be specified using `clap`'s builder API
#[derive(Serialize, Deserialize, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    // Identity details
    #[arg(
        long,
        env = "HOPRD_IDENTITY",
        help = "The path to the identity file",
        required = false
    )]
    pub identity: Option<String>,

    // Identity details
    #[arg(
        long,
        env = "HOPRD_DATA",
        help = "Specifies the directory to hold all the data",
        required = false
    )]
    pub data: Option<String>,

    #[arg(
        long,
        env = "HOPRD_HOST",
        help = "Host to listen on for P2P connections",
        value_parser = ValueParser::new(parse_host),
    )]
    pub host: Option<HostConfig>,

    #[arg(
        long,
        env = "HOPRD_ANNOUNCE",
        help = "Announce the node on chain with a public address",
        action = ArgAction::Count
    )]
    pub announce: u8,

    #[arg(
        long,
        env = "HOPRD_API",
        help = format!("Expose the API on {}:{}", DEFAULT_API_HOST, DEFAULT_API_PORT),
        action = ArgAction::Count
    )]
    pub api: u8,

    #[arg(
        long = "apiHost",
        value_name = "HOST",
        help = "Set host IP to which the API server will bind",
        env = "HOPRD_API_HOST"
    )]
    pub api_host: Option<String>,

    #[arg(
        long = "apiPort",
        value_parser = clap::value_parser ! (u16),
        value_name = "PORT",
        help = "Set port to which the API server will bind",
        env = "HOPRD_API_PORT"
    )]
    pub api_port: Option<u16>,

    #[arg(
        long = "defaultSessionListenHost",
        env = "HOPRD_DEFAULT_SESSION_LISTEN_HOST",
        help = "Default Session listening host for Session IP forwarding",
        value_parser = ValueParser::new(parse_host),
    )]
    pub default_session_listen_host: Option<HostConfig>,

    #[arg(
        long = "disableApiAuthentication",
        help = "Completely disables the token authentication for the API, overrides any apiToken if set",
        env = "HOPRD_DISABLE_API_AUTHENTICATION",
        hide = true,
        action = ArgAction::Count
    )]
    pub disable_api_authentication: u8,

    #[arg(
        long = "apiToken",
        alias = "api-token",
        help = "A REST API token and for user authentication",
        value_name = "TOKEN",
        value_parser = ValueParser::new(parse_api_token),
        env = "HOPRD_API_TOKEN"
    )]
    pub api_token: Option<String>,

    #[arg(
        long,
        env = "HOPRD_PASSWORD",
        help = "A password to encrypt your keys",
        value_name = "PASSWORD"
    )]
    pub password: Option<String>,

    #[arg(
        long,
        help = "URL for Blokli provider to be used for the node to connect to blockchain",
        env = "HOPRD_BLOKLI_URL",
        value_name = "BLOKLI_URL"
    )]
    pub blokli_url: Option<String>,

    #[arg(
        long,
        help = "initialize a database if it doesn't already exist",
        env = "HOPRD_INIT",
        action = ArgAction::Count
    )]
    pub init: u8,

    #[arg(
        long = "forceInit",
        help = "initialize a database, even if it already exists",
        env = "HOPRD_FORCE_INIT",
        action = ArgAction::Count
    )]
    pub force_init: u8,

    #[arg(
        long = "privateKey",
        hide = true,
        help = "A private key to be used for the node",
        env = "HOPRD_PRIVATE_KEY",
        value_name = "PRIVATE_KEY"
    )]
    pub private_key: Option<String>,

    #[arg(
        long = "testAnnounceLocalAddresses",
        env = "HOPRD_TEST_ANNOUNCE_LOCAL_ADDRESSES",
        help = "For testing local testnets. Announce local addresses",
        hide = true,
        action = ArgAction::Count
    )]
    pub test_announce_local_addresses: u8,

    #[arg(
        long = "testPreferLocalAddresses",
        env = "HOPRD_TEST_PREFER_LOCAL_ADDRESSES",
        help = "For testing local testnets. Prefer local peers to remote",
        hide = true,
        action = ArgAction::Count
    )]
    pub test_prefer_local_addresses: u8,

    #[arg(
        long = "probeRecheckThreshold",
        help = "Timeframe in seconds after which it is reasonable to recheck the nearest neighbor",
        value_name = "SECONDS",
        value_parser = clap::value_parser ! (u64),
        env = "HOPRD_PROBE_RECHECK_THRESHOLD",
    )]
    pub probe_recheck_threshold: Option<u64>,

    #[arg(
        long = "configurationFilePath",
        required = false,
        help = "Path to a file containing the entire HOPRd configuration",
        value_name = "CONFIG_FILE_PATH",
        value_parser = clap::value_parser ! (String),
        env = "HOPRD_CONFIGURATION_FILE_PATH"
    )]
    pub configuration_file_path: Option<String>,

    #[arg(
        long = "safeAddress",
        value_name = "HOPRD_SAFE_ADDR",
        help = "Address of Safe that safeguards tokens",
        env = "HOPRD_SAFE_ADDRESS"
    )]
    pub safe_address: Option<String>,

    #[arg(
        long = "moduleAddress",
        value_name = "HOPRD_MODULE_ADDR",
        help = "Address of the node management module",
        env = "HOPRD_MODULE_ADDRESS"
    )]
    pub module_address: Option<String>,
}

impl TryFrom<CliArgs> for HoprdConfig {
    type Error = HoprdError;

    fn try_from(value: CliArgs) -> Result<Self, Self::Error> {
        let mut cfg: HoprdConfig = if let Some(cfg_path) = value.configuration_file_path {
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
        if let Some(x) = value.host {
            cfg.hopr.host = x
        };

        // hopr.transport
        if value.test_announce_local_addresses > 0 {
            cfg.hopr.network.announce_local_addresses = true;
        }
        if value.test_prefer_local_addresses > 0 {
            cfg.hopr.network.prefer_local_addresses = true;
        }

        if let Some(host) = value.default_session_listen_host {
            cfg.session_ip_forwarding.default_entry_listen_host = match host.address {
                HostType::IPv4(addr) => IpAddr::from_str(&addr)
                    .map(|ip| std::net::SocketAddr::new(ip, host.port))
                    .map_err(|_| HoprdError::ConfigError("invalid default session listen IP address".into())),
                HostType::Domain(_) => Err(HoprdError::ConfigError("default session listen must be an IP".into())),
            }?;
        }

        // db
        if let Some(data) = value.data {
            cfg.db.data = data
        }
        if value.init > 0 {
            cfg.db.initialize = true;
        }
        if value.force_init > 0 {
            cfg.db.force_initialize = true;
        }

        // api
        if value.api > 0 {
            cfg.api.enable = true;
        }
        if value.disable_api_authentication > 0 && cfg.api.auth != Auth::None {
            cfg.api.auth = Auth::None;
        };
        if let Some(x) = value.api_token {
            cfg.api.auth = Auth::Token(x);
        };
        if let Some(x) = value.api_host {
            cfg.api.host =
                HostConfig::from_str(format!("{}:{}", x.as_str(), hoprd_api::config::DEFAULT_API_PORT).as_str())
                    .map_err(crate::errors::HoprdError::ValidationError)?;
        }
        if let Some(x) = value.api_port {
            cfg.api.host.port = x
        };

        // probe
        if let Some(x) = value.probe_recheck_threshold {
            cfg.hopr.network.probe_recheck_threshold = std::time::Duration::from_secs(x)
        };

        // identity
        if let Some(identity) = value.identity {
            cfg.identity.file = identity;
        }
        if let Some(x) = value.password {
            cfg.identity.password = x
        };
        if let Some(x) = value.private_key {
            cfg.identity.private_key = Some(x)
        };

        // chain
        if value.announce > 0 {
            cfg.hopr.announce = true;
        }

        if let Some(x) = value.blokli_url {
            cfg.blokli_url = Some(x);
        }

        if let Some(x) = value.safe_address {
            cfg.hopr.safe_module.safe_address =
                Address::from_str(&x).map_err(|e| HoprdError::ValidationError(e.to_string()))?
        };
        if let Some(x) = value.module_address {
            cfg.hopr.safe_module.module_address =
                Address::from_str(&x).map_err(|e| HoprdError::ValidationError(e.to_string()))?
        };

        // additional updates
        let home_symbol = '~';
        if cfg.db.data.starts_with(home_symbol) {
            cfg.db.data = home::home_dir()
                .map(|h| h.as_path().display().to_string())
                .expect("home dir for a user must be specified")
                + &cfg.db.data[1..];
        }
        if cfg.identity.file.starts_with(home_symbol) {
            cfg.identity.file = home::home_dir()
                .map(|h| h.as_path().display().to_string())
                .expect("home dir for a user must be specified")
                + &cfg.identity.file[1..];
        }

        Ok(cfg)
    }
}
