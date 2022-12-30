use clap::builder::PossibleValuesParser;
use clap::{Arg, ArgAction, ArgMatches, Args, Command, FromArgMatches as _};
use real_base::real;
use serde::{Deserialize, Serialize};
use serde_json;

use serde_json::{Map, Value};
use wasm_bindgen::JsValue;

use wasm_bindgen::prelude::*;

const DEFAULT_ID_PATH: &str = ".hopr-identity";

const HEARTBEAT_INTERVAL: u32 = 60000;
const HEARTBEAT_THRESHOLD: u32 = 60000;
const HEARTBEAT_INTERVAL_VARIANCE: u32 = 2000;

const CONFIRMATIONS: u32 = 8;

const NETWORK_QUALITY_THRESHOLD: f32 = 0.5;

const DEFAULT_API_HOST: &str = "localhost";
const DEFAULT_API_PORT: u16 = 3001;

const DEFAULT_HEALTH_CHECK_HOST: &str = "localhost";
const DEFAULT_HEALTH_CHECK_PORT: u16 = 8080;

#[derive(serde::Deserialize)]
pub struct ProtocolConfigFile {
    environments: Map<String, Value>,
}

#[derive(Serialize, Args)]
struct CliArgs {
    // Filled by Builder API at runtime
    #[arg(skip)]
    environment: String,

    // Filled by Builder API at runtime
    #[arg(skip)]
    identity: String,
    
    // Filled by Builder API at runtime
    #[arg(skip)]
    data: String,

    #[arg(
        long, 
        default_value_t = false, 
        env = "HOPRD_API", 
        help = format!("Expose the API on {}:{}", DEFAULT_API_HOST, DEFAULT_API_PORT), 
        action = ArgAction::SetTrue
    )]
    api: bool,

    #[arg(
        long = "apiHost",
        default_value_t = DEFAULT_API_HOST.to_string(),
        value_name = "HOST",
        help = "Set host IP to which the API server will bind",
        env = "HOPRD_API_HOST"
    )]
    api_host: String,
    #[arg(
        long = "apiPort",
        default_value_t = DEFAULT_API_PORT,
        value_parser = clap::value_parser!(u16),
        value_name = "PORT",
        help = "Set port to which the API server will bind",
        env = "HOPRD_API_HOST"
    )]
    api_port: u16,

    #[arg(
        long = "apiToken",
        help = "A REST API token and for user authentication",
        value_name = "TOKEN",
        env = "HOPRD_API_TOKEN"
    )]
    api_token: Option<String>,

    #[arg(
        long = "healthCheck",
        default_value_t = false,
        env = "HOPRD_HEALTH_CHECK",
        help = format!("Run a health check end point on {}:{}", DEFAULT_HEALTH_CHECK_HOST, DEFAULT_HEALTH_CHECK_PORT)
    )]
    health_check: bool,

    #[arg(
        long = "healthCheckHost",
        default_value_t = DEFAULT_HEALTH_CHECK_HOST.to_string(),
        value_name = "HOST",
        help = "Updates the host for the healthcheck server",
        env = "HOPRD_HEALTH_CHECK_HOST",
    )]
    health_check_host: String,

    #[arg(
        long = "healthCheckPort",
        default_value_t = DEFAULT_HEALTH_CHECK_PORT,
        value_name = "PORT",
        value_parser = clap::value_parser!(u16),
        help = "Updates the port for the healthcheck server",
        env = "HOPRD_HEALTH_CHECK_PORT"
    )]
    health_check_port: u16,

    #[arg(
        long,
        env = "HOPRD_PASSWORD",
        help = "A password to encrypt your keys"
    )]
    password: Option<String>,

    #[arg(
        long,
        help = "A custom RPC provider to be used for the node to connect to blockchain",
        env = "HOPRD_PROVIDER",
        value_name = "PROVIDER"
    )]
    provider: Option<String>,

    #[arg(
        long = "dryRun",
        help = "List all the options used to run the HOPR node, but quit instead of starting",
        env = "HOPRD_DRY_RUN",
        default_value_t = false,
        action = ArgAction:: SetTrue
    )]
    dry_run: bool,

    #[arg(
        long,
        help = "initialize a database if it doesn't already exist",
        action = ArgAction::SetTrue,
        env = "HOPRD_INIT",
        default_value_t = false
    )]
    init: bool,

    #[arg(
        long = "privateKey",
        hide = true,
        help = "A private key to be used for the node",
        env = "HOPRD_PRIVATE_KEY",
        value_name = "PRIVATE_KEY"
    )]
    private_key: Option<String>,

    #[arg(
        long = "allowLocalNodeConnections",
        env = "HOPRD_ALLOW_LOCAL_NODE_CONNECTIONS",
        action = ArgAction::SetTrue,
        help = "Allow connections to other nodes running on localhost",
        default_value_t = false
    )]
    allow_local_node_connections: bool,

    #[arg(
        long = "allowPrivateNodeConnections",
        env = "HOPRD_ALLOW_PRIVATE_NODE_CONNECTIONS",
        action = ArgAction::SetTrue,
        default_value_t = false,
        help = "Allow connections to other nodes running on private addresses",
    )]
    allow_private_node_connections: bool,

    #[arg(
        long = "testAnnounceLocalAddresses",
        env = "HOPRD_TEST_ANNOUNCE_LOCAL_ADDRESSES",
        help = "For testing local testnets. Announce local addresses",
        action = ArgAction::SetTrue,
        default_value_t = false
    )]
    test_announce_local_addresses:bool,

    #[arg(
        long = "testPreferLocalAddresses",
        env = "HOPRD_TEST_PREFER_LOCAL_ADDRESSES",
        action = ArgAction::SetTrue,
        help = "For testing local testnets. Prefer local peers to remote",
        default_value_t = true,
        hide = true
    )]
    test_prefer_local_addresses: bool,

    #[arg(
        long = "testUseWeakCrypto",
        env = "HOPRD_TEST_USE_WEAK_CRYPTO",
        action = ArgAction::SetTrue,
        help = "weaker crypto for faster node startup",
        hide = true,
        default_value_t = false
    )]
    test_use_weak_crypto: bool,

    #[arg(
        long = "testNoAuthentication",
        help = "no remote authentication for easier testing",
        action = ArgAction::SetTrue,
        env = "HOPRD_TEST_NO_AUTHENTICATION",
        default_value_t = false,
        hide = true
    )]
    test_no_authentication: bool,

    #[arg(
        long = "testNoDirectConnections",
        help = "NAT traversal testing: prevent nodes from establishing direct TCP connections",
        env = "HOPRD_TEST_NO_DIRECT_CONNECTIONS",
        default_value_t = false,
        action = ArgAction::SetTrue,
        hide = true
    )]
    test_no_direct_connections: bool,

    #[arg(
        long = "testNoWebRTCUpgrade",
        help = "NAT traversal testing: prevent nodes from establishing direct TCP connections",
        env = "HOPRD_TEST_NO_WEBRTC_UPGRADE",
        default_value_t = false,
        action = ArgAction::SetTrue,
        hide = true
    )]
    test_no_webrtc_upgrade: bool,

    #[arg(
        long = "heartbeatInterval",
        help = "Interval in milliseconds in which the availability of other nodes get measured",
        value_name = "MILLISECONDS",
        value_parser = clap::value_parser!(u32),
        default_value_t = HEARTBEAT_INTERVAL,
        env = "HOPRD_HEARTBEAT_INTERVAL",
    )]
    heartbeat_interval: u32,

    #[arg(
        long = "heartbeatThreshold",
        help = "Timeframe in milliseconds after which a heartbeat to another peer is performed, if it hasn't been seen since",
        value_name = "MILLISECONDS",
        value_parser = clap::value_parser!(u32),
        default_value_t = HEARTBEAT_THRESHOLD,
        env = "HOPRD_HEARTBEAT_THRESHOLD",
    )]
    heartbeat_threshold: u32,

    #[arg(
        long = "heartbeatVariance",
        help = "Upper bound for variance applied to heartbeat interval in milliseconds",
        value_name = "MILLISECONDS",
        value_parser = clap::value_parser!(u32),
        default_value_t = HEARTBEAT_INTERVAL_VARIANCE,
        env = "HOPRD_HEARTBEAT_VARIANCE"
    )]
    heartbeat_variance: u32,

    #[arg(
        long = "onChainConfirmations",
        help = "Number of confirmations required for on-chain transactions",
        value_name = "CONFIRMATIONS",
        value_parser = clap::value_parser!(u32),
        default_value_t = CONFIRMATIONS,
        env = "HOPRD_ON_CHAIN_CONFIRMATIONS",

    )]
    on_chain_confirmations: u32,

    #[arg(
        long = "networkQualityThreshold",
        help = "Miniumum quality of a peer connection to be considered usable",
        value_name = "THRESHOLD",
        value_parser = clap::value_parser!(f32),
        default_value_t = NETWORK_QUALITY_THRESHOLD,
        env = "HOPRD_NETWORK_QUALITY_THRESHOLD"
    )]
    network_quality_threshold: f32,
}

impl CliArgs {
    fn augment_runtime_args(&mut self, m: &ArgMatches) {
        self.environment = m.get_one::<String>("environment").unwrap().to_owned();
        self.data = m.get_one::<String>("data").unwrap().to_owned();
        self.identity = m.get_one::<String>("identity").unwrap().to_owned();
    }
}

#[derive(serde::Deserialize)]
struct PackageJsonFile {
    version: String,
}

fn get_package_version(path: String) -> Result<String, JsValue> {
    let data = real::read_file(&path)?;

    match serde_json::from_slice::<PackageJsonFile>(&data) {
        Ok(json) => Ok(json.version),
        Err(e) => Err(JsValue::from(e.to_string())),
    }
}

fn get_environments(path: String) -> Result<Vec<String>, JsValue> {
    let data = real::read_file(&path)?;

    let protocolConfig = serde_json::from_slice::<ProtocolConfigFile>(&data)
        .map_err(|e| JsValue::from(e.to_string()))?;

    Ok(protocolConfig
        .environments
        .iter()
        .map(|env| env.0.to_owned())
        .collect::<Vec<String>>())
}

#[derive(Deserialize)]
struct DefaultEnvironmentFile {
    id: String,
}

fn get_default_environment(path: String) -> Result<Option<String>, JsValue> {
    let data = match real::read_file(&path) {
        Ok(data) => data,
        // File only exists in containers,
        // so nothing to worry if file cannot be read
        Err(_) => return Ok(None)
    };

    match serde_json::from_slice::<DefaultEnvironmentFile>(&data) {
        Ok(json) => Ok(Some(json.id)),
        Err(e) => Err(JsValue::from(e.to_string())),
    }
}

pub fn parse_cli_arguments(cli_args: Vec<&str>) -> Result<JsValue, JsValue> {
    let envs: Vec<String> = get_environments(String::from("./packages/core/protocol-config.json"))?;

    let version = get_package_version(String::from("./packages/hoprd/package.json"))?;

    let maybe_default_environment = get_default_environment(String::from("../default-environment.json"))?;

    let mut default_data_path = String::from("hoprd-db");

    let mut env_arg = Arg::new("environment")
        .long("environment")
        .required(true)
        .env("HOPRD_ENVIRONMENT")
        .value_name("ENVIRONMENT")
        .help("Environment id which the node shall run on")
        .value_parser(PossibleValuesParser::new(envs));

    if let Some(default_environment) = maybe_default_environment {
        default_data_path.push_str(default_environment.as_str());

        env_arg = env_arg.default_value(default_environment);
    }

    let mut cmd = Command::new("hoprd")
        .about("HOPRd")
        .bin_name("index.cjs")
        .version(&version)
        .arg(env_arg)
        .arg(Arg::new("identity")
            .long("identity")
            .help("The path to the identity file")
            .env("HOPRD_IDENTITY")
            .default_value(DEFAULT_ID_PATH))
        .arg(Arg::new("data")
            .long("data")
            .help("manually specify the data directory to use")
            .env("HOPRD_DATA")
            .default_value(&default_data_path));

    cmd = CliArgs::augment_args(cmd);

    let derived_matches = match cmd.try_get_matches_from(cli_args) {
        Ok(matches) => matches,
        Err(e) => return Err(JsValue::from(e.to_string()))
    };

    let mut args = CliArgs::from_arg_matches(&derived_matches).unwrap();

    args.augment_runtime_args(&derived_matches);

    match serde_wasm_bindgen::to_value(&args) {
        Ok(s) => Ok(s),
        Err(e) => Err(JsValue::from(e.to_string())),
    }
}

pub mod wasm {
    use js_sys::JsString;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    /// Macro used to convert Vec<JsString> to Vec<&str>
    macro_rules! convert_from_jstrvec {
        ($v:expr,$r:ident) => {
            let _aux: Vec<String> = $v.iter().map(String::from).collect();
            let $r = _aux.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
        };
    }

    #[wasm_bindgen]
    pub fn parse_cli_arguments(
        cli_args: Vec<JsString>,
        envs: &JsValue,
    ) -> Result<JsValue, JsValue> {
        convert_from_jstrvec!(cli_args, cli);

        super::parse_cli_arguments(cli)
    }
}
