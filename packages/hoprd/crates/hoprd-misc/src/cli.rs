use clap::builder::PossibleValuesParser;
use clap::ValueHint::DirPath;
use clap::{Error, Parser};
use serde::Serialize;
#[derive(Parser, Serialize, Debug)]
#[command(bin_name = "index.cjs")]
pub struct Args {
    /// Name of the person to greet
    #[arg(
        long,
        help = "Environment id which the node shall run on [env: HOPRD_ENVIRONMENT]",
        value_name = "ENV",
        value_parser = ["foo"]
    )]
    environment: String,

    #[arg(
        long,
        help = "The network host to run the HOPR node on [env: HOPRD_HOST]",
        default_value = "0.0.0.0:9091"
    )]
    host: String,

    #[arg(long, help = "Announce public IP to the network [env: HOPRD_ANNOUNCE]")]
    announce: bool,
    // .option('announce', {
    //   boolean: true,
    //   describe: 'Announce public IP to the network [env: HOPRD_ANNOUNCE]',
    //   default: false
    // })
}

#[derive(Clone)]
pub struct EnvironmentParser {}

// impl clap::builder::TypedValueParser for EnvironmentParser {
//     type Value = String;
//     fn parse_ref(
//         &self,
//         cmd: &clap::Command,
//         arg: Option<&clap::Arg>,
//         value: &std::ffi::OsStr,
//     ) -> Result<Self::Value, clap::Error> {
//         let inner = clap::value_parser!(String);
//         let val = inner.parse_ref(cmd, arg, value)?;

//         String::from("foo")
//     }
// }

// .env('HOPRD') // enable options to be set as environment variables with the HOPRD prefix
// .epilogue(
//   'All CLI options can be configured through environment variables as well. CLI parameters have precedence over environment variables.'
// )
// .version(version)
// .option('environment', {
//   string: true,
//   describe: 'Environment id which the node shall run on (HOPRD_ENVIRONMENT)',
//   choices: supportedEnvironments().map((env) => env.id),
//   default: defaultEnvironment()
// })

// .option('api', {
//   boolean: true,
//   describe: 'Expose the API on localhost:3001. [env: HOPRD_API]',
//   default: false
// })
// .option('apiHost', {
//   string: true,
//   describe: 'Set host IP to which the API server will bind. [env: HOPRD_API_HOST]',
//   default: 'localhost'
// })
// .option('apiPort', {
//   number: true,
//   describe: 'Set host port to which the API server will bind. [env: HOPRD_API_PORT]',
//   default: 3001
// })
// .option('apiToken', {
//   string: true,
//   describe: 'A REST API token and for user authentication [env: HOPRD_API_TOKEN]',
//   default: undefined,
//   conflicts: 'testNoAuthentication'
// })
// .option('healthCheck', {
//   boolean: true,
//   describe: 'Run a health check end point on localhost:8080 [env: HOPRD_HEALTH_CHECK]',
//   default: false
// })
// .option('healthCheckHost', {
//   string: true,
//   describe: 'Updates the host for the healthcheck server [env: HOPRD_HEALTH_CHECK_HOST]',
//   default: 'localhost'
// })
// .option('healthCheckPort', {
//   number: true,
//   describe: 'Updates the port for the healthcheck server [env: HOPRD_HEALTH_CHECK_PORT]',
//   default: 8080
// })
// .option('password', {
//   string: true,
//   describe: 'A password to encrypt your keys [env: HOPRD_PASSWORD]',
//   default: ''
// })
// .option('provider', {
//   string: true,
//   describe: 'A custom RPC provider to be used for the node to connect to blockchain [env: HOPRD_PROVIDER]'
// })
// .option('identity', {
//   string: true,
//   describe: 'The path to the identity file [env: HOPRD_IDENTITY]',
//   default: DEFAULT_ID_PATH
// })
// .option('dryRun', {
//   boolean: true,
//   describe: 'List all the options used to run the HOPR node, but quit instead of starting [env: HOPRD_DRY_RUN]',
//   default: false
// })
// .option('data', {
//   string: true,
//   describe: 'manually specify the data directory to use [env: HOPRD_DATA]',
//   default: defaultDataPath
// })
// .option('init', {
//   boolean: true,
//   describe: "initialize a database if it doesn't already exist [env: HOPRD_INIT]",
//   default: false
// })
// .option('privateKey', {
//   hidden: true,
//   string: true,
//   describe: 'A private key to be used for the node [env: HOPRD_PRIVATE_KEY]',
//   default: undefined
// })
// .option('allowLocalNodeConnections', {
//   boolean: true,
//   describe: 'Allow connections to other nodes running on localhost [env: HOPRD_ALLOW_LOCAL_NODE_CONNECTIONS]',
//   default: false
// })
// .option('allowPrivateNodeConnections', {
//   boolean: true,
//   describe:
//     'Allow connections to other nodes running on private addresses [env: HOPRD_ALLOW_PRIVATE_NODE_CONNECTIONS]',
//   default: false
// })
// .option('testAnnounceLocalAddresses', {
//   hidden: true,
//   boolean: true,
//   describe: 'For testing local testnets. Announce local addresses [env: HOPRD_TEST_ANNOUNCE_LOCAL_ADDRESSES]',
//   default: false
// })
// .option('testPreferLocalAddresses', {
//   hidden: true,
//   boolean: true,
//   describe: 'For testing local testnets. Prefer local peers to remote [env: HOPRD_TEST_PREFER_LOCAL_ADDRESSES]',
//   default: false
// })
// .option('testUseWeakCrypto', {
//   hidden: true,
//   boolean: true,
//   describe: 'weaker crypto for faster node startup [env: HOPRD_TEST_USE_WEAK_CRYPTO]',
//   default: false
// })
// .option('testNoAuthentication', {
//   hidden: true,
//   boolean: true,
//   describe: 'no remote authentication for easier testing [env: HOPRD_TEST_NO_AUTHENTICATION]',
//   default: undefined
// })
// .option('testNoDirectConnections', {
//   hidden: true,
//   boolean: true,
//   describe:
//     'NAT traversal testing: prevent nodes from establishing direct TCP connections [env: HOPRD_TEST_NO_DIRECT_CONNECTIONS]',
//   default: false
// })
// .option('testNoWebRTCUpgrade', {
//   hidden: true,
//   boolean: true,
//   describe:
//     'NAT traversal testing: prevent nodes from establishing direct TCP connections [env: HOPRD_TEST_NO_WEB_RTC_UPGRADE]',
//   default: false
// })
// .option('testNoUPNP', {
//   hidden: true,
//   boolean: true,
//   describe:
//     'NAT traversal testing: disable automatic detection of external IP address using UPNP [env: HOPRD_TEST_NO_UPNP]',
//   default: false
// })
// .option('heartbeatInterval', {
//   number: true,
//   describe:
//     'Interval in milliseconds in which the availability of other nodes get measured [env: HOPRD_HEARTBEAT_INTERVAL]',
//   default: HEARTBEAT_INTERVAL
// })
// .option('heartbeatThreshold', {
//   number: true,
//   describe:
//     "Timeframe in milliseconds after which a heartbeat to another peer is performed, if it hasn't been seen since [env: HOPRD_HEARTBEAT_THRESHOLD]",
//   default: HEARTBEAT_THRESHOLD
// })
// .option('heartbeatVariance', {
//   number: true,
//   describe: 'Upper bound for variance applied to heartbeat interval in milliseconds [env: HOPRD_HEARTBEAT_VARIANCE]',
//   default: HEARTBEAT_INTERVAL_VARIANCE
// })
// .option('networkQualityThreshold', {
//   number: true,
//   describe: 'Miniumum quality of a peer connection to be considered usable [env: HOPRD_NETWORK_QUALITY_THRESHOLD]',
//   default: NETWORK_QUALITY_THRESHOLD
// })
// .option('onChainConfirmations', {
//   number: true,
//   describe: 'Number of confirmations required for on-chain transactions [env: HOPRD_ON_CHAIN_CONFIRMATIONS]',
//   default: CONFIRMATIONS
// })

pub fn parse_cli_arguments(cli_args: Vec<&str>) -> Result<Args, Error> {
    Args::try_parse_from(cli_args)
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
    pub fn parse_cli_arguments(cli_args: Vec<JsString>) -> Result<JsValue, String> {
        convert_from_jstrvec!(cli_args, cli);

        match super::parse_cli_arguments(cli) {
            Ok(args) => match serde_wasm_bindgen::to_value(&args) {
                Ok(string) => Ok(string),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    }
}
