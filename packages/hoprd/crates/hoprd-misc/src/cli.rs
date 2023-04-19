use std::collections::HashMap;
use std::ffi::OsString;

use clap::builder::{PossibleValuesParser, ValueParser};
use clap::{Arg, ArgAction, ArgMatches, Args, Command, FromArgMatches as _};
use core_misc::environment::{Environment, FromJsonFile, PackageJsonFile, ProtocolConfig};
use core_strategy::{
    generic::ChannelStrategy, passive::PassiveStrategy, promiscuous::PromiscuousStrategy, random::RandomStrategy,
};
use hex;
use proc_macro_regex::regex;
use real_base::real;
use serde::{Deserialize, Serialize};
use serde_json;
use utils_misc::ok_or_str;

pub const DEFAULT_API_HOST: &str = "localhost";
pub const DEFAULT_API_PORT: u16 = 3001;

pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 9091;

pub const DEFAULT_HEALTH_CHECK_HOST: &str = "localhost";
pub const DEFAULT_HEALTH_CHECK_PORT: u16 = 8080;

pub const MINIMAL_API_TOKEN_LENGTH: usize = 8;

regex!(is_ipv4_host "^[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}[:]{1}[0-9]{1,6}$");

fn parse_host(s: &str) -> Result<crate::config::Host, String> {
    if !is_ipv4_host(s) {
        return Err(format!(
            "Given string {} is not a valid host, Example: {}:{}",
            s,
            DEFAULT_HOST.to_string(),
            DEFAULT_PORT.to_string()
        ));
    }

    crate::config::Host::from_ipv4_host_string(s)
}

/// Parse a hex string private key to a boxed u8 slice
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn parse_private_key(s: &str) -> Result<Box<[u8]>, String> {
    if crate::config::validate_private_key(s).is_ok() {
        let mut decoded = [0u8; 32];

        let priv_key = match s.strip_prefix("0x") {
            Some(priv_without_prefix) => priv_without_prefix,
            None => s,
        };

        // no errors because filtered by regex
        hex::decode_to_slice(priv_key, &mut decoded).unwrap();

        Ok(Box::new(decoded))
    } else {
        Err(format!(
            "Given string is not a private key. A private key must contain 64 hex chars."
        ))
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

/// Takes all CLI arguments whose structure is known at compile-time.
/// Arguments whose structure, e.g. their default values depend on
/// file contents need be specified using `clap`s builder API
#[derive(Serialize, Deserialize, Args, Clone)]
#[command(about = "HOPRd")]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct CliArgs {
    /// Environment
    // Filled by Builder API at runtime
    #[arg(skip)]
    pub environment: String,

    // Filled by Builder API at runtime
    #[arg(skip)]
    pub identity: String,

    // Filled by Builder API at runtime
    #[arg(skip)]
    pub data: String,

    #[arg(
        long,
        env = "HOPRD_HOST",
        help = "Host to listen on for P2P connections",
        value_parser = ValueParser::new(parse_host),
    )]
    pub host: Option<crate::config::Host>,

    #[arg(
        long,
        env = "HOPRD_ANNOUNCE",
        help = "Run as a Public Relay Node (PRN)",
        action = ArgAction::SetTrue
    )]
    pub announce: Option<bool>,

    #[arg(
        long,
        env = "HOPRD_API",
        help = "Enable the internal API",
        action = ArgAction::SetTrue,
    )]
    pub api: Option<bool>,

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
        long = "disableApiAuthentication",
        help = "Completely disables the token authentication for the API, overrides any apiToken if set",
        action = ArgAction::SetTrue,
        env = "HOPRD_DISABLE_API_AUTHENTICATION",
        hide = true
    )]
    pub disable_api_authentication: Option<bool>,

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
        long = "healthCheck",
        env = "HOPRD_HEALTH_CHECK",
        help = "Run a health check end point",
        action = ArgAction::SetTrue
    )]
    pub health_check: Option<bool>,

    #[arg(
        long = "healthCheckHost",
        value_name = "HOST",
        help = "Updates the host for the healthcheck server",
        env = "HOPRD_HEALTH_CHECK_HOST"
    )]
    pub health_check_host: Option<String>,

    #[arg(
        long = "healthCheckPort",
        value_name = "PORT",
        value_parser = clap::value_parser ! (u16),
        help = "Updates the port for the healthcheck server",
        env = "HOPRD_HEALTH_CHECK_PORT"
    )]
    pub health_check_port: Option<u16>,

    #[arg(
        long,
        env = "HOPRD_PASSWORD",
        help = "A password to encrypt your keys",
        value_name = "PASSWORD"
    )]
    pub password: Option<String>,

    #[arg(
        long = "defaultStrategy",
        help = "Default channel strategy to use after node starts up",
        env = "HOPRD_DEFAULT_STRATEGY",
        value_name = "DEFAULT_STRATEGY",
        value_parser = PossibleValuesParser::new([PromiscuousStrategy::NAME, PassiveStrategy::NAME, RandomStrategy::NAME])
    )]
    pub default_strategy: Option<String>,

    #[arg(
        long = "maxAutoChannels",
        help = "Maximum number of channel a strategy can open. If not specified, square root of number of available peers is used.",
        env = "HOPRD_MAX_AUTO_CHANNELS",
        value_name = "MAX_AUTO_CHANNELS",
        value_parser = clap::value_parser ! (u32)
    )]
    pub max_auto_channels: Option<u32>, // Make this a string if we want to supply functions instead in the future.

    #[arg(
        long = "autoRedeemTickets",
        env = "HOPRD_AUTO_REDEEEM_TICKETS",
        help = "If enabled automatically redeems winning tickets.",
        action = ArgAction::SetTrue
    )]
    pub auto_redeem_tickets: Option<bool>,

    #[arg(
        long = "checkUnrealizedBalance",
        env = "HOPRD_CHECK_UNREALIZED_BALANCE",
        help = "Determines if unrealized balance shall be checked first before validating unacknowledged tickets.",
        action = ArgAction::SetTrue
    )]
    pub check_unrealized_balance: Option<bool>,

    #[arg(
        long,
        help = "A custom RPC provider to be used for the node to connect to blockchain",
        env = "HOPRD_PROVIDER",
        value_name = "PROVIDER"
    )]
    pub provider: Option<String>,

    #[arg(
        long = "dryRun",
        help = "List all the options used to run the HOPR node, but quit instead of starting",
        env = "HOPRD_DRY_RUN",
        default_value_t = false,
        action = ArgAction::SetTrue
    )]
    pub dry_run: bool,

    #[arg(
        long,
        help = "initialize a database if it doesn't already exist",
        action = ArgAction::SetTrue,
        env = "HOPRD_INIT",
        action = ArgAction::SetTrue
    )]
    pub init: Option<bool>,

    #[arg(
        long = "forceInit",
        help = "initialize a database, even if it already exists",
        action = ArgAction::SetTrue,
        env = "HOPRD_FORCE_INIT",
        action = ArgAction::SetTrue
    )]
    pub force_init: Option<bool>,

    #[arg(
        long = "privateKey",
        hide = true,
        help = "A private key to be used for the node",
        env = "HOPRD_PRIVATE_KEY",
        value_name = "PRIVATE_KEY"
    )]
    pub private_key: Option<String>,

    #[arg(
        long = "allowLocalNodeConnections",
        env = "HOPRD_ALLOW_LOCAL_NODE_CONNECTIONS",
        action = ArgAction::SetTrue,
        help = "Allow connections to other nodes running on localhost",
    )]
    pub allow_local_node_connections: Option<bool>,

    #[arg(
        long = "allowPrivateNodeConnections",
        env = "HOPRD_ALLOW_PRIVATE_NODE_CONNECTIONS",
        action = ArgAction::SetTrue,
        help = "Allow connections to other nodes running on private addresses",
    )]
    pub allow_private_node_connections: Option<bool>,

    #[arg(
        long = "maxParallelConnections",
        value_parser = clap::value_parser ! (u32).range(1..),
        value_name = "CONNECTIONS",
        help = "Set maximum parallel connections",
        env = "HOPRD_MAX_PARALLEL_CONNECTIONS"
    )]
    pub max_parallel_connections: Option<u32>,

    #[arg(
        long = "testAnnounceLocalAddresses",
        env = "HOPRD_TEST_ANNOUNCE_LOCAL_ADDRESSES",
        help = "For testing local testnets. Announce local addresses",
        action = ArgAction::SetTrue,
    )]
    pub test_announce_local_addresses: Option<bool>,

    #[arg(
        long = "testPreferLocalAddresses",
        env = "HOPRD_TEST_PREFER_LOCAL_ADDRESSES",
        action = ArgAction::SetTrue,
        help = "For testing local testnets. Prefer local peers to remote",
        hide = true
    )]
    pub test_prefer_local_addresses: Option<bool>,

    #[arg(
        long = "testUseWeakCrypto",
        env = "HOPRD_TEST_USE_WEAK_CRYPTO",
        action = ArgAction::SetTrue,
        help = "weaker crypto for faster node startup",
        hide = true,
    )]
    pub test_use_weak_crypto: Option<bool>,

    #[arg(
        long = "testNoDirectConnections",
        help = "NAT traversal testing: prevent nodes from establishing direct TCP connections",
        env = "HOPRD_TEST_NO_DIRECT_CONNECTIONS",
        action = ArgAction::SetTrue,
        hide = true
    )]
    pub test_no_direct_connections: Option<bool>,

    #[arg(
        long = "testNoWebRTCUpgrade",
        help = "NAT traversal testing: prevent nodes from establishing direct TCP connections",
        env = "HOPRD_TEST_NO_WEBRTC_UPGRADE",
        action = ArgAction::SetTrue,
        hide = true
    )]
    pub test_no_webrtc_upgrade: Option<bool>,

    #[arg(
        long = "testLocalModeStun",
        help = "Transport testing: use full-featured STUN with local addresses",
        env = "HOPRD_TEST_LOCAL_MODE_STUN",
        action = ArgAction::SetTrue,
        hide = true
    )]
    pub test_local_mode_stun: Option<bool>,

    #[arg(
        long = "heartbeatInterval",
        help = "Interval in milliseconds in which the availability of other nodes get measured",
        value_name = "MILLISECONDS",
        value_parser = clap::value_parser ! (u32),
        env = "HOPRD_HEARTBEAT_INTERVAL",
    )]
    pub heartbeat_interval: Option<u32>,

    #[arg(
        long = "heartbeatThreshold",
        help = "Timeframe in milliseconds after which a heartbeat to another peer is performed, if it hasn't been seen since",
        value_name = "MILLISECONDS",
        value_parser = clap::value_parser ! (u32),
        env = "HOPRD_HEARTBEAT_THRESHOLD",
    )]
    pub heartbeat_threshold: Option<u32>,

    #[arg(
        long = "heartbeatVariance",
        help = "Upper bound for variance applied to heartbeat interval in milliseconds",
        value_name = "MILLISECONDS",
        value_parser = clap::value_parser ! (u32),
        env = "HOPRD_HEARTBEAT_VARIANCE"
    )]
    pub heartbeat_variance: Option<u32>,

    #[arg(
        long = "onChainConfirmations",
        help = "Number of confirmations required for on-chain transactions",
        value_name = "CONFIRMATIONS",
        value_parser = clap::value_parser ! (u32),
        env = "HOPRD_ON_CHAIN_CONFIRMATIONS",
    )]
    pub on_chain_confirmations: Option<u32>,

    #[arg(
        long = "networkQualityThreshold",
        help = "Minimum quality of a peer connection to be considered usable",
        value_name = "THRESHOLD",
        value_parser = clap::value_parser ! (f32),
        env = "HOPRD_NETWORK_QUALITY_THRESHOLD"
    )]
    pub network_quality_threshold: Option<f32>,

    #[arg(
        long = "configurationFilePath",
        required = false,
        help = "Path to a file containing the entire HOPRd configuration",
        value_name = "CONFIG_FILE_PATH",
        value_parser = clap::value_parser ! (String),
        env = "HOPRD_CONFIGURATION_FILE_PATH"
    )]
    pub configuration_file_path: Option<String>,
}

impl CliArgs {
    /// Add values of those CLI arguments whose structure is known at runtime
    fn augment_runtime_args(&mut self, m: &ArgMatches) {
        self.environment = m.get_one::<String>("environment").unwrap().to_owned();
        self.data = m.get_one::<String>("data").unwrap().to_owned();
        self.identity = m.get_one::<String>("identity").unwrap().to_owned();
    }

    /// Creates a new instance using custom cli_args and custom environment variables
    fn new_from(
        cli_args: Vec<&str>,
        env_vars: HashMap<OsString, OsString>,
        mono_repo_path: &str,
        home_path: &str,
    ) -> Result<Self, String> {
        let envs: Vec<Environment> =
            ProtocolConfig::from_json_file(mono_repo_path).and_then(|c| c.supported_environments(mono_repo_path))?;

        let version = PackageJsonFile::from_json_file(mono_repo_path).and_then(|p| p.coerced_version())?;

        let maybe_default_environment = get_default_environment(mono_repo_path);

        let mut env_arg = Arg::new("environment")
            .long("environment")
            .required(true)
            .env("HOPRD_ENVIRONMENT")
            .value_name("ENVIRONMENT")
            .help("Environment id which the node shall run on")
            .value_parser(PossibleValuesParser::new(envs.iter().map(|e| e.id.to_owned())));

        if let Some(default_environment) = &maybe_default_environment {
            // Add default value if we got one
            env_arg = env_arg.default_value(default_environment);
        }

        let mut cmd = Command::new("hoprd")
            .about("HOPRd")
            .bin_name("index.cjs")
            .after_help("All CLI options can be configured through environment variables as well. CLI parameters have precedence over environment variables.")
            .version(&version)
            .arg(env_arg)
            .arg(Arg::new("identity")
                .long("identity")
                .help("The path to the identity file")
                .env("HOPRD_IDENTITY")
                .default_value(format!("{}/.hopr-identity", home_path)))
            .arg(Arg::new("data")
                .long("data")
                .help("manually specify the data directory to use")
                .env("HOPRD_DATA")
                .default_value(get_data_path(mono_repo_path, maybe_default_environment)));

        // Add compile args to runtime-time args
        cmd = Self::augment_args(cmd);

        cmd.update_env_from(env_vars);

        let derived_matches = cmd.try_get_matches_from(cli_args).map_err(|e| e.to_string())?;

        let mut args = ok_or_str!(Self::from_arg_matches(&derived_matches))?;

        args.augment_runtime_args(&derived_matches);

        Ok(args)
    }
}

#[derive(Deserialize)]
struct DefaultEnvironmentFile {
    id: String,
}

impl FromJsonFile for DefaultEnvironmentFile {
    fn from_json_file(mono_repo_path: &str) -> Result<Self, String> {
        let default_environment_json_path: String = format!("{}/default-environment.json", mono_repo_path);
        let data = ok_or_str!(real::read_file(default_environment_json_path.as_str()))?;

        ok_or_str!(serde_json::from_slice::<DefaultEnvironmentFile>(&data))
    }
}

/// Checks for `default_environment.json` file and, if present, returns their environment id
fn get_default_environment(mono_repo_path: &str) -> Option<String> {
    match DefaultEnvironmentFile::from_json_file(mono_repo_path) {
        Ok(json) => Some(json.id),
        Err(_) => None,
    }
}

/// Gets the default path where the database is stored at
fn get_data_path(mono_repo_path: &str, maybe_default_environment: Option<String>) -> String {
    match maybe_default_environment {
        Some(default_environment) => format!("{}/packages/hoprd/hoprd-db/{}", mono_repo_path, default_environment),
        None => format!("{}/packages/hoprd/hoprd-db", mono_repo_path),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_private_key() {
        let parsed =
            super::parse_private_key("cd09f9293ffdd69be978032c533b6bcd02dfd5d937c987bedec3e28de07e0317").unwrap();

        let priv_key: Vec<u8> = vec![
            205, 9, 249, 41, 63, 253, 214, 155, 233, 120, 3, 44, 83, 59, 107, 205, 2, 223, 213, 217, 55, 201, 135, 190,
            222, 195, 226, 141, 224, 126, 3, 23,
        ];

        assert_eq!(parsed, priv_key.into())
    }

    #[test]
    fn parse_private_key_with_prefix() {
        let parsed_with_prefix =
            super::parse_private_key("cd09f9293ffdd69be978032c533b6bcd02dfd5d937c987bedec3e28de07e0317").unwrap();

        let priv_key: Vec<u8> = vec![
            205, 9, 249, 41, 63, 253, 214, 155, 233, 120, 3, 44, 83, 59, 107, 205, 2, 223, 213, 217, 55, 201, 135, 190,
            222, 195, 226, 141, 224, 126, 3, 23,
        ];

        assert_eq!(parsed_with_prefix, priv_key.into())
    }

    #[test]
    fn parse_too_short_private_key() {
        let parsed =
            super::parse_private_key("cd09f9293ffdd69be978032c533b6bcd02dfd5d937c987bedec3e28de07e031").unwrap_err();

        assert_eq!(
            parsed,
            "Given string is not a private key. A private key must contain 64 hex chars."
        )
    }

    #[test]
    fn parse_too_long_private_key() {
        let parsed =
            super::parse_private_key("cd09f9293ffdd69be978032c533b6bcd02dfd5d937c987bedec3e28de07e03177").unwrap_err();

        assert_eq!(
            parsed,
            "Given string is not a private key. A private key must contain 64 hex chars."
        )
    }

    #[test]
    fn parse_non_hex_values() {
        let parsed = super::parse_private_key("really not a private key").unwrap_err();

        assert_eq!(
            parsed,
            "Given string is not a private key. A private key must contain 64 hex chars."
        )
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::JsString;
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::str::FromStr;
    use utils_misc::{clean_mono_repo_path, convert_from_jstrvec, ok_or_jserr};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen]
    pub fn parse_cli_arguments(
        cli_args: Vec<JsString>,
        envs: &JsValue,
        mono_repo_path: &str,
        home_path: &str,
    ) -> Result<JsValue, JsValue> {
        convert_from_jstrvec!(cli_args, cli_str_args);
        clean_mono_repo_path!(mono_repo_path, cleaned_mono_repo_path);

        // wasm_bindgen receives Strings but to
        // comply with Rust standard, turn them into OsStrings
        let string_envs = ok_or_jserr!(serde_wasm_bindgen::from_value::<HashMap<String, String>>(envs.into(),))?;

        let mut env_map: HashMap<OsString, OsString> = HashMap::new();
        for (ref k, ref v) in string_envs {
            let key = OsString::from_str(k)
                .or_else(|e| Err(format!("Could not convert key {} to OsString: {}", k, e.to_string())))?;
            let value = OsString::from_str(v)
                .or_else(|e| Err(format!("Could not convert value {} to OsString: {}", v, e.to_string())))?;

            env_map.insert(key, value);
        }

        let args = ok_or_jserr!(super::CliArgs::new_from(
            cli_str_args,
            env_map,
            cleaned_mono_repo_path,
            home_path
        ))?;

        ok_or_jserr!(serde_wasm_bindgen::to_value(&args))
    }
}
