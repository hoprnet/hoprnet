use std::collections::HashMap;
use std::ffi::OsString;
use std::str::FromStr;

use clap::builder::{PossibleValuesParser, ValueParser};
use clap::{ArgAction, Args, Command, FromArgMatches as _};
use core_strategy::Strategy;
use core_transport::config::HostConfig;
use hex;
use hopr_lib::ProtocolConfig;
use serde::{Deserialize, Serialize};
use strum::VariantNames;

#[cfg(not(feature = "wasm"))]
use utils_validation::network::native::is_dns_address;
#[cfg(feature = "wasm")]
use {
    utils_validation::network::wasm::is_dns_address,
    wasm_bindgen::JsError
};

pub const DEFAULT_API_HOST: &str = "localhost";
pub const DEFAULT_API_PORT: u16 = 3001;

pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 9091;

pub const DEFAULT_HEALTH_CHECK_HOST: &str = "localhost";
pub const DEFAULT_HEALTH_CHECK_PORT: u16 = 8080;

pub const MINIMAL_API_TOKEN_LENGTH: usize = 8;

fn parse_host(s: &str) -> Result<HostConfig, String> {
    let host = s.split_once(':').map_or(s, |(h, _)| h);
    if !(validator::validate_ip_v4(host) || is_dns_address(host)) {
        return Err(
            "Given string {s} is not a valid host, Example: {DEFAULT_HOST}:{DEFAULT_PORT}".into()
        );
    }

    HostConfig::from_str(s)
}

/// Parse a hex string private key to a boxed u8 slice
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn parse_private_key(s: &str) -> Result<Box<[u8]>, String> {
    if crate::config::validate_private_key(s).is_ok() {
        let mut decoded = [0u8; 64];

        let priv_key = match s.strip_prefix("0x") {
            Some(priv_without_prefix) => priv_without_prefix,
            None => s,
        };

        // no errors because filtered by regex
        hex::decode_to_slice(priv_key, &mut decoded).unwrap();

        Ok(Box::new(decoded))
    } else {
        Err(
            "Given string is not a private key. A private key must contain 128 hex chars.".into()
        )
    }
}

fn parse_api_token(mut s: &str) -> Result<String, String> {
    if s.len() < MINIMAL_API_TOKEN_LENGTH {
        return Err(format!(
            "Length of API token is too short, minimally required {MINIMAL_API_TOKEN_LENGTH} but given {}", s.len()
        ));
    }

    match (s.starts_with('\''), s.ends_with('\'')) {
        (true, true) => {
            s = s.strip_prefix('\'').unwrap();
            s = s.strip_suffix('\'').unwrap();

            Ok(s.into())
        }
        (true, false) => Err("Found leading quote but no trailing quote".into()),
        (false, true) => Err("Found trailing quote but no leading quote".into()),
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
    /// Network the node will operate in
    #[arg(
        long,
        env = "HOPRD_NETWORK",
        help = "ID of the network the node will attempt to connect to",
        required = false,
        value_parser = PossibleValuesParser::new(ProtocolConfig::default().supported_networks().iter().map(|e| e.id.clone()))
    )]
    pub network: Option<String>,

    // Identitiy details
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
        action = ArgAction::SetTrue,
        default_value_t = hopr_lib::config::Chain::default().announce
    )]
    pub announce: bool,

    #[arg(
        long,
        env = "HOPRD_API",
        help = format!("Expose the API on {}:{}", DEFAULT_API_HOST, DEFAULT_API_PORT),
        action = ArgAction::SetTrue,
        default_value_t = crate::config::Api::default().enable
    )]
    pub api: bool,

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
        hide = true,
        default_value_t = crate::config::Api::default().is_auth_disabled()
    )]
    pub disable_api_authentication: bool,

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
        action = ArgAction::SetTrue,
        default_value_t = crate::config::HealthCheck::default().enable
    )]
    pub health_check: bool,

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
        value_parser = PossibleValuesParser::new(Strategy::VARIANTS)
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
        long = "disableTicketAutoRedeem",
        env = "HOPRD_DISABLE_AUTO_REDEEEM_TICKETS",
        help = "Disables automatic redeeming of winning tickets.",
        action = ArgAction::SetFalse,
        default_value_t = false
    )]
    pub auto_redeem_tickets: bool,

    #[arg(
        long = "disableUnrealizedBalanceCheck",
        env = "HOPRD_DISABLE_UNREALIZED_BALANCE_CHECK",
        help = "Disables checking of unrealized balance before validating unacknowledged tickets.",
        action = ArgAction::SetFalse,
        default_value_t = hopr_lib::config::Chain::default().check_unrealized_balance
    )]
    pub check_unrealized_balance: bool,

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
        default_value_t = hopr_lib::config::Db::default().initialize
    )]
    pub init: bool,

    #[arg(
        long = "forceInit",
        help = "initialize a database, even if it already exists",
        action = ArgAction::SetTrue,
        env = "HOPRD_FORCE_INIT",
        default_value_t = hopr_lib::config::Db::default().force_initialize
    )]
    pub force_init: bool,

    #[arg(
        long = "privateKey",
        hide = true,
        help = "A private key to be used for the node",
        env = "HOPRD_PRIVATE_KEY",
        value_name = "PRIVATE_KEY"
    )]
    pub private_key: Option<String>,

    #[arg(
        long = "inbox-capacity",
        value_parser = clap::value_parser ! (u32).range(1..),
        value_name = "INBOX_CAPACITY",
        help = "Set maximum capacity of the HOPRd inbox",
        env = "HOPRD_INBOX_CAPACITY"
    )]
    pub inbox_capacity: Option<u32>,

    #[arg(
        long = "testAnnounceLocalAddresses",
        env = "HOPRD_TEST_ANNOUNCE_LOCAL_ADDRESSES",
        help = "For testing local testnets. Announce local addresses",
        action = ArgAction::SetTrue,
        default_value_t = hopr_lib::config::TransportConfig::default().announce_local_addresses
    )]
    pub test_announce_local_addresses: bool,

    #[arg(
        long = "testPreferLocalAddresses",
        env = "HOPRD_TEST_PREFER_LOCAL_ADDRESSES",
        action = ArgAction::SetTrue,
        help = "For testing local testnets. Prefer local peers to remote",
        hide = true,
        default_value_t = hopr_lib::config::TransportConfig::default().prefer_local_addresses
    )]
    pub test_prefer_local_addresses: bool,

    #[arg(
        long = "testUseWeakCrypto",
        env = "HOPRD_TEST_USE_WEAK_CRYPTO",
        action = ArgAction::SetTrue,
        help = "weaker crypto for faster node startup",
        hide = true,
        default_value_t = crate::config::Testing::default().use_weak_crypto
    )]
    pub test_use_weak_crypto: bool,

    #[arg(
        long = "heartbeatInterval",
        help = "Interval in milliseconds in which the availability of other nodes get measured",
        value_name = "MILLISECONDS",
        value_parser = clap::value_parser ! (u64),
        env = "HOPRD_HEARTBEAT_INTERVAL",
    )]
    pub heartbeat_interval: Option<u64>,

    #[arg(
        long = "heartbeatThreshold",
        help = "Timeframe in milliseconds after which a heartbeat to another peer is performed, if it hasn't been seen since",
        value_name = "MILLISECONDS",
        value_parser = clap::value_parser ! (u64),
        env = "HOPRD_HEARTBEAT_THRESHOLD",
    )]
    pub heartbeat_threshold: Option<u64>,

    #[arg(
        long = "heartbeatVariance",
        help = "Upper bound for variance applied to heartbeat interval in milliseconds",
        value_name = "MILLISECONDS",
        value_parser = clap::value_parser ! (u64),
        env = "HOPRD_HEARTBEAT_VARIANCE"
    )]
    pub heartbeat_variance: Option<u64>,

    #[arg(
        long = "networkQualityThreshold",
        help = "Minimum quality of a peer connection to be considered usable",
        value_name = "THRESHOLD",
        value_parser = clap::value_parser ! (f64),
        env = "HOPRD_NETWORK_QUALITY_THRESHOLD"
    )]
    pub network_quality_threshold: Option<f64>,

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
        long = "safeTransactionServiceProvider",
        value_name = "HOPRD_SAFE_TX_SERVICE_PROVIDER",
        help = "Base URL for safe transaction service",
        env = "HOPRD_SAFE_TRANSACTION_SERVICE_PROVIDER"
    )]
    pub safe_transaction_service_provider: Option<String>,

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
        help = "Address of the node mangement module",
        env = "HOPRD_MODULE_ADDRESS"
    )]
    pub module_address: Option<String>,
}

#[cfg(feature = "wasm")]
impl CliArgs {
    /// Creates a new instance using custom cli_args and custom network variables
    fn new_from(cli_args: Vec<&str>, env_vars: HashMap<OsString, OsString>) -> Result<Self, wasm_bindgen::JsError> {
        let mut cmd = Command::new("hoprd")
            .about("HOPRd")
            .bin_name("index.cjs")
            .after_help("All CLI options can be configured through environment variables as well. CLI parameters have precedence over environment variables.")
            .version(hopr_lib::constants::APP_VERSION_COERCED);

        cmd = Self::augment_args(cmd);

        cmd.update_env_from(env_vars);

        let derived_matches = cmd.try_get_matches_from(cli_args).map_err(JsError::from)?;

        Self::from_arg_matches(&derived_matches)
            .map_err(wasm_bindgen::JsError::from)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_private_key() {
        let parsed =
            super::parse_private_key("56b29cefcdf576eea306ba2fd5f32e651c09e0abbc018c47bdc6ef44f6b7506f1050f95137770478f50b456267f761f1b8b341a13da68bc32e5c96984fcd52ae").unwrap();

        let priv_key: Vec<u8> = vec![
            86, 178, 156, 239, 205, 245, 118, 238, 163, 6, 186, 47, 213, 243, 46, 101, 28, 9, 224, 171, 188, 1, 140,
            71, 189, 198, 239, 68, 246, 183, 80, 111, 16, 80, 249, 81, 55, 119, 4, 120, 245, 11, 69, 98, 103, 247, 97,
            241, 184, 179, 65, 161, 61, 166, 139, 195, 46, 92, 150, 152, 79, 205, 82, 174,
        ];

        assert_eq!(parsed, priv_key.into())
    }

    #[test]
    fn parse_private_key_with_prefix() {
        let parsed_with_prefix =
            super::parse_private_key("0x56b29cefcdf576eea306ba2fd5f32e651c09e0abbc018c47bdc6ef44f6b7506f1050f95137770478f50b456267f761f1b8b341a13da68bc32e5c96984fcd52ae").unwrap();

        let priv_key: Vec<u8> = vec![
            86, 178, 156, 239, 205, 245, 118, 238, 163, 6, 186, 47, 213, 243, 46, 101, 28, 9, 224, 171, 188, 1, 140,
            71, 189, 198, 239, 68, 246, 183, 80, 111, 16, 80, 249, 81, 55, 119, 4, 120, 245, 11, 69, 98, 103, 247, 97,
            241, 184, 179, 65, 161, 61, 166, 139, 195, 46, 92, 150, 152, 79, 205, 82, 174,
        ];

        assert_eq!(parsed_with_prefix, priv_key.into())
    }

    #[test]
    fn parse_too_short_private_key() {
        let parsed =
            super::parse_private_key("56b29cefcdf576eea306ba2fd5f32e651c09e0abbc018c47bdc6ef44f6b7506f1050f95137770478f50b456267f761f1b8b341a13da68bc32e5c96984fcd52").unwrap_err();

        assert_eq!(
            parsed,
            "Given string is not a private key. A private key must contain 128 hex chars."
        )
    }

    #[test]
    fn parse_too_long_private_key() {
        let parsed =
            super::parse_private_key("0x56b29cefcdf576eea306ba2fd5f32e651c09e0abbc018c47bdc6ef44f6b7506f1050f95137770478f50b456267f761f1b8b341a13da68bc32e5c96984fcd52aeae").unwrap_err();

        assert_eq!(
            parsed,
            "Given string is not a private key. A private key must contain 128 hex chars."
        )
    }

    #[test]
    fn parse_non_hex_values() {
        let parsed = super::parse_private_key("really not a private key").unwrap_err();

        assert_eq!(
            parsed,
            "Given string is not a private key. A private key must contain 128 hex chars."
        )
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::JsString;
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::str::FromStr;
    use utils_misc::convert_from_jstrvec;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen]
    pub fn parse_cli_arguments(cli_args: Vec<JsString>, envs: &JsValue) -> Result<JsValue, JsError> {
        convert_from_jstrvec!(cli_args, cli_str_args);

        // wasm_bindgen receives Strings but to
        // comply with Rust standard, turn them into OsStrings
        let string_envs = serde_wasm_bindgen::from_value::<HashMap<String, String>>(envs.into(),)
            .map_err(JsError::from)?;

        let mut env_map: HashMap<OsString, OsString> = HashMap::new();
        for (ref k, ref v) in string_envs {
            let key = OsString::from_str(k)
                .map_err(|e| JsError::new(format!("Could not convert key {k} to OsString: {e}").as_str()))?;
            let value = OsString::from_str(v)
                .map_err(|e| JsError::new(format!("Could not convert value {v} to OsString: {e}").as_str()))?;

            env_map.insert(key, value);
        }

        let args = super::CliArgs::new_from(cli_str_args, env_map,)?;

        serde_wasm_bindgen::to_value(&args).map_err(JsError::from)
    }
}
