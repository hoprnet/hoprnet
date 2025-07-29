use std::str::FromStr;

use clap::{ArgAction, Parser, builder::ValueParser};
use hopr_lib::{HostConfig, looks_like_domain};
use serde::{Deserialize, Serialize};

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
        long = "noKeepLogs",
        env = "HOPRD_INDEXER_DISABLE_KEEP_LOGS",
        help = "Disables keeping RPC logs in the logs database after they were processed.",
        action = ArgAction::Count
    )]
    pub no_keep_logs: u8,

    #[arg(
        long = "noFastSync",
        env = "HOPRD_INDEXER_DISABLE_FAST_SYNC",
        help = "Disables using fast sync at node start.",
        action = ArgAction::Count
    )]
    pub no_fast_sync: u8,

    #[arg(
        long = "enableLogsSnapshot",
        env = "HOPRD_ENABLE_LOGS_SNAPSHOT",
        help = "Enables downloading logs snapshot at node start. If this is set to true, the node will attempt to \
                download logs snapshot from the configured `logsSnapshotUrl`.",
        value_name = "ENABLE_LOGS_SNAPSHOT",
        action = ArgAction::Count
    )]
    pub enable_logs_snapshot: u8,

    #[arg(
        long = "logsSnapshotUrl",
        env = "HOPRD_LOGS_SNAPSHOT_URL",
        help = "URL to download logs snapshot from. If none is provided or configured in the configuration file, the \
                node will not attempt to download any logs snapshot.",
        value_name = "LOGS_SNAPSHOT_URL"
    )]
    pub logs_snapshot_url: Option<String>,

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
        help = "Maximum number of RPC requests that can be performed per second.",
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
        help = "Address of the node management module",
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
}
