use clap::builder::{
    IntoResettable, PossibleValue, PossibleValuesParser, Resettable, Str, StringValueParser,
    TypedValueParser,
};
use clap::{Arg, ArgAction, ArgMatches, Args, Command, FromArgMatches as _};
use real_base::real;
use serde::{Deserialize, Serialize};
use serde_json;
impl From<ArgMatches> for CliArgs {
    fn from(m: ArgMatches) -> Self {
        CliArgs {
            enviromment: m.get_one::<String>("name").cloned().unwrap(),
            api_port: m.get_one::<u16>("apiPort").cloned().unwrap(),
            api_host: m.get_one("apiHost").cloned().unwrap(),
        }
    }
}
use serde_json::{Map, Value};
use wasm_bindgen::JsValue;

const DEFAULT_ID_PATH: &str = ".hopr-identity";

const HEARTBEAT_INTERVAL: u32 = 60000;
const HEARTBEAT_THRESHOLD: u32 = 60000;
const HEARTBEAT_INTERVAL_VARIANCE: u32 = 2000;

const CONFIRMATIONS: u32 = 8;

const NETWORK_QUALITY_THRESHOLD: f32 = 0.5;

#[derive(serde::Deserialize)]
pub struct ProtocolConfigFile {
    environments: Map<String, Value>,
}

#[derive(Serialize, Args)]
struct CliArgs {
    enviromment: String,
    api_port: u16,
    api_host: String,
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

fn get_default_environment(path: String) -> Result<String, JsValue> {
    let data = real::read_file(&path)?;

    match serde_json::from_slice::<DefaultEnvironmentFile>(&data) {
        Ok(json) => Ok(json.id),
        Err(e) => Err(JsValue::from(e.to_string())),
    }
}

pub fn parse_cli_arguments(cli_args: Vec<&str>) -> Result<JsValue, JsValue> {
    let envs: Vec<String> = get_environments(String::from("./packages/core/protocol-config.json"))?;

    let version = get_package_version(String::from("./package.json"))?;

    let default_environment = get_default_environment(String::from("../default-environment.json"))?;

    let mut default_data_path = String::from("hoprd-db");
    default_data_path.push_str(default_environment.as_str());

    let cmd = Command::new("hoprd")
    .after_help("All CLI options can be configured through environment variables as well. CLI parameters have precedence over environment variables.")
        .version(&version)
        .arg(
            Arg::new("apiHost")
                .long("apiHost")
                .default_value("localhost")
                .env("HOPRD_API_HOST")
                .value_name("HOST")
                .help("Set host IP to which the API server will bind"))
        .arg(
            Arg::new("apiPort")
                .long("apiPort")
                .default_value("3001")
                .value_parser(clap::value_parser!(u16))
                .env("HOPRD_API_PORT")
                .help("Set host port to which the API server will bind."))
        .arg(
            Arg::new("environment")
                .long("environment")
                .required(true)
                .env("HOPRD_ENVIRONMENT")
                .value_name("ENVIRONMENT")
                .help("Environment id which the node shall run on")
                .value_parser(PossibleValuesParser::new(envs)))
        .arg(
            Arg::new("api")
                .long("api")
                .env("HOPRD_API")
                .action(ArgAction::SetTrue)
                .help("Expose the API on localhost:3001")
                .default_value("false"))
        .arg(Arg::new("apiToken")
                .long("apiToken")
                .env("HOPRD_API_TOKEN")
                .help("A REST API token and for user authentication"))
        .arg(Arg::new("healthCheck")
                .long("healthCheck")
                .env("HOPRD_HEALTH_CHECK")
                .help("Run a health check end point on localhost:8080")
                .action(ArgAction::SetTrue)
            .default_value("false"))
        .arg(Arg::new("healthCheckHost")
                .long("healthCheckHost")
                .env("HOPRD_HEALTH_CHECK_HOST")
                .help("Updates the host for the healthcheck server")
                .default_value("localhost"))
        .arg(Arg::new("healthCheckPort")
                .long("healthCheckPort")
                .env("HOPRD_HEALTH_CHECK_PORT")
                .help("Updates the port for the healthcheck server")
                .value_parser(clap::value_parser!(u16))
                .default_value("8080"))
        .arg(Arg::new("password")
                .long("password")
                .help("A password to encrypt your keys")
                .env("HOPRD_PASSWORD")
                .default_value(""))
        .arg(Arg::new("provider")
                .long("provider")
                .help("A custom RPC provider to be used for the node to connect to blockchain")
                .env("HOPRD_PROVIDER"))
        .arg(Arg::new("identity")
                .long("identity")
                .help("The path to the identity file")
                .env("HOPRD_IDENTITY")
                .default_value(DEFAULT_ID_PATH))
        .arg(Arg::new("dryRun")
                .long("dryRun")
                .help("List all the options used to run the HOPR node, but quit instead of starting")
                .env("HOPRD_DRY_RUN")
                .default_value("false")
                .action(ArgAction::SetTrue))
        .arg(Arg::new("data")
                .long("data")
                .help("manually specify the data directory to use")
                .env("HOPRD_DATA")
                .default_value(&default_data_path))
        .arg(Arg::new("init")
                .long("init")
                .help("initialize a database if it doesn't already exist")
                .action(ArgAction::SetTrue)
                .env("HOPRD_INIT")
                .default_value("false"))
        .arg(Arg::new("privateKey")
                .long("privateKey")
                .hide(true)
                .help("A private key to be used for the node")
                .env("HOPRD_PRIVATE_KEY"))
        .arg(Arg::new("allowLocalNodeConnections")
                .long("allowLocalNodeConnections")
                .env("HOPRD_ALLOW_LOCAL_NODE_CONNECTIONS")
                .action(ArgAction::SetTrue)
                .help("Allow connections to other nodes running on localhost")
                .default_value("false"))
        .arg(Arg::new("allowPrivateNodeConnections")
                .long("allowPrivateNodeConnections")
                .env("HOPRD_ALLOW_PRIVATE_NODE_CONNECTIONS")
                .action(ArgAction::SetTrue)
                .help("Allow connections to other nodes running on private addresses")
                .default_value("false"))
        .arg(Arg::new("testAnnounceLocalAddresses")
                .long("testAnnounceLocalAddresses")
                .env("HOPRD_TEST_ANNOUNCE_LOCAL_ADDRESSES")
                .help("For testing local testnets. Announce local addresses")
                .action(ArgAction::SetTrue)
                .default_value("false"))
                .hide(true)
        .arg(Arg::new("testPreferLocalAddresses")
                .long("testPreferLocalAddresses")
                .env("HOPRD_TEST_PREFER_LOCAL_ADDRESSES")
                .action(ArgAction::SetTrue)
                .help("For testing local testnets. Prefer local peers to remote")
                .default_value("false")
                .hide(true))
        .arg(Arg::new("testUseWeakCrypto")
                .long("testUseWeakCrypto")
                .env("HOPRD_TEST_USE_WEAK_CRYPTO")
                .action(ArgAction::SetTrue)
                .help("weaker crypto for faster node startup")
                .default_value("false")
                .hide(true))
        .arg(Arg::new("testNoAuthentication")
                .long("testNoAuthentication")
                .help("no remote authentication for easier testing")
                .action(ArgAction::SetTrue)
                .env("HOPRD_TEST_NO_AUTHENTICATION")
                .default_value("false")
                .hide(true))
        .arg(Arg::new("testNoDirectConnections")
                .long("testNoDirectConnections")
                .help("NAT traversal testing: prevent nodes from establishing direct TCP connections")
                .env("HOPRD_TEST_NO_DIRECT_CONNECTIONS")
                .default_value("false")
                .action(ArgAction::SetTrue)
                .hide(true))
        .arg(Arg::new("testNoWebRTCUpgrade")
                .long("testNoWebRTCUpgrade")
                .help("NAT traversal testing: prevent nodes from establishing direct TCP connections")
                .env("HOPRD_TEST_NO_WEB_RTC_UPGRADE")
                .default_value("false")
                .hide(true))
        .arg(Arg::new("heartbeatInterval")
                .long("heartbeatInterval")
                .help("Interval in milliseconds in which the availability of other nodes get measured")
                .env("HOPRD_HEARTBEAT_INTERVAL")
                .default_value(HEARTBEAT_INTERVAL.to_string()))
        .arg(Arg::new("heartbeatThreshold")
                .long("heartbeatThreshold")
                .help("Timeframe in milliseconds after which a heartbeat to another peer is performed, if it hasn't been seen since")
                .env("HOPRD_HEARTBEAT_THRESHOLD")
                .default_value(HEARTBEAT_THRESHOLD.to_string()))
        .arg(Arg::new("heartbeatVariance")
                .long("heartbeatVariance")
                .help("Upper bound for variance applied to heartbeat interval in milliseconds")
                .env("HOPRD_HEARTBEAT_VARIANCE")
                .default_value(HEARTBEAT_INTERVAL_VARIANCE.to_string()))
        .arg(Arg::new("networkQualityThreshold")
                .long("networkQualityThreshold")
                .help("Miniumum quality of a peer connection to be considered usable")
                .env("HOPRD_NETWORK_QUALITY_THRESHOLD")
                .default_value(NETWORK_QUALITY_THRESHOLD.to_string()))
        .arg(Arg::new("onChainConfirmations")
                .long("onChainConfirmations")
                .help("Number of confirmations required for on-chain transactions")
                .env("HOPRD_ON_CHAIN_CONFIRMATIONS")
                .default_value(CONFIRMATIONS.to_string()));

    // CliArgs::from_arg_matches(&cmd.try_get_matches_from(cli_args).unwrap());

    let args = match cmd.try_get_matches_from(cli_args) {
        Ok(matches) => CliArgs::from(matches),
        Err(e) => return Err(JsValue::from(e.to_string())),
    };

    match serde_wasm_bindgen::to_value(&args) {
        Ok(s) => Ok(s),
        Err(e) => Err(JsValue::from(e.to_string())),
    }
    // Args::try_update_from(
    //     ,
    //     cli_args,
    // );

    // real::read_file("../package.json")
    // .map(|data| {
    //     serde_json::from_slice::<PackageJsonFile>(data)
    //         .map(|json| json.version)
    //         .map_err(|e| JsValue::from(e))
    // })
    // .map_err(|e| JsValue::from(e))

    // serde_json::from_slice(&data)
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
