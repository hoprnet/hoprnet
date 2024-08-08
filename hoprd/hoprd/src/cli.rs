use clap::builder::{PossibleValuesParser, ValueParser};
use clap::{ArgAction, Parser};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::VariantNames;

use hopr_lib::{looks_like_domain, HostConfig, Strategy};

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
#[derive(Serialize, Deserialize, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Network the node will operate in
    #[arg(
        long,
        env = "HOPRD_NETWORK",
        help = "ID of the network the node will attempt to connect to",
        required = false
    )]
    pub network: Option<String>,

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
        long = "disableUnrealizedBalanceCheck",
        env = "HOPRD_DISABLE_UNREALIZED_BALANCE_CHECK",
        help = "Disables checking of unrealized balance before validating unacknowledged tickets.",
        action = ArgAction::Count
    )]
    pub check_unrealized_balance: u8,

    #[arg(
        long = "maxBlockRange",
        help = "Maximum number of blocks that can be fetched in a batch request from the RPC provider.",
        env = "HOPRD_MAX_BLOCK_RANGE",
        value_name = "MAX_BLOCK_RANGE",
        value_parser = clap::value_parser ! (u64)
    )]
    pub max_block_range: Option<u64>,

    #[arg(
        long = "maxRequestsPerSec",
        help = "Maximum number of RPC requestes that can be performed per second.",
        env = "HOPRD_MAX_RPC_REQUESTS_PER_SEC",
        value_name = "MAX_RPC_REQUESTS_PER_SEC",
        value_parser = clap::value_parser ! (u32)
    )]
    pub max_rpc_requests_per_sec: Option<u32>,

    #[arg(
        long,
        help = "A custom RPC provider to be used for the node to connect to blockchain",
        env = "HOPRD_PROVIDER",
        value_name = "PROVIDER"
    )]
    pub provider: Option<String>,

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

    #[arg(
        long = "protocolConfig",
        value_name = "HOPRD_PROTOCOL_CONFIG_PATH",
        help = "Path to the protocol-config.json file",
        env = "HOPRD_PROTOCOL_CONFIG_PATH"
    )]
    pub protocol_config_path: Option<String>,

    // ==================================
    /// deprecated
    #[deprecated]
    #[arg(
        long = "testUseWeakCrypto",
        env = "HOPRD_TEST_USE_WEAK_CRYPTO",
        action = ArgAction::SetTrue,
        help = "DEPRECATED",
        hide = true,
        default_value_t = false
    )]
    pub test_use_weak_crypto: bool,

    /// deprecated
    #[deprecated]
    #[arg(
        long = "dryRun",
        help = "DEPRECATED",
        env = "HOPRD_DRY_RUN",
        default_value_t = false,
        action = ArgAction::SetTrue
    )]
    pub dry_run: bool,

    /// deprecated
    #[deprecated]
    #[arg(
        long = "healthCheck",
        help = "DEPRECATED",
        action = ArgAction::SetTrue,
        default_value_t = false
    )]
    pub health_check: bool,

    /// deprecated
    #[deprecated]
    #[arg(long = "healthCheckHost", help = "DEPRECATED")]
    pub health_check_host: Option<String>,

    /// deprecated
    #[deprecated]
    #[arg(
        long = "healthCheckPort",
        value_parser = clap::value_parser ! (u16),
        help = "DEPRECATED",
    )]
    pub health_check_port: Option<u16>,

    //#[deprecated] not marked as deprecated to allow showing a warning
    #[arg(
    long = "defaultStrategy",
    help = "DEPRECATED",
    env = "HOPRD_DEFAULT_STRATEGY",
    value_name = "DEFAULT_STRATEGY",
    value_parser = PossibleValuesParser::new(Strategy::VARIANTS)
    )]
    pub default_strategy: Option<String>,

    //#[deprecated] not marked as deprecated to allow showing a warning
    #[arg(
    long = "maxAutoChannels",
    help = "DEPRECATED",
    env = "HOPRD_MAX_AUTO_CHANNELS",
    value_name = "MAX_AUTO_CHANNELS",
    value_parser = clap::value_parser ! (u32)
    )]
    pub max_auto_channels: Option<u32>, // Make this a string if we want to supply functions instead in the future.

    //#[deprecated] not marked as deprecated to allow showing a warning
    #[arg(
    long = "disableTicketAutoRedeem",
    env = "HOPRD_DISABLE_AUTO_REDEEEM_TICKETS",
    help = "DEPRECATED",
    action = ArgAction::Count
    )]
    pub auto_redeem_tickets: u8,
}
