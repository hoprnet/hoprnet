use proc_macro_regex::regex;
use utils_log::error;

use serde::{Serialize, Deserialize};
use validator::{Validate, ValidationError};

use core_misc::constants::{
    DEFAULT_HEARTBEAT_INTERVAL, DEFAULT_HEARTBEAT_INTERVAL_VARIANCE, DEFAULT_HEARTBEAT_THRESHOLD,
    DEFAULT_NETWORK_QUALITY_THRESHOLD, DEFAULT_MAX_PARALLEL_CONNECTIONS, DEFAULT_MAX_PARALLEL_CONNECTION_PUBLIC_RELAY,
};

use utils_proc_macros::wasm_bindgen_if;

pub const DEFAULT_API_HOST: &str = "localhost";
pub const DEFAULT_API_PORT: u16 = 3001;

pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 9091;

pub const DEFAULT_HEALTH_CHECK_HOST: &str = "localhost";
pub const DEFAULT_HEALTH_CHECK_PORT: u16 = 8080;

pub const MINIMAL_API_TOKEN_LENGTH: usize = 8;


fn validate_ipv4_address(s: &str) -> Result<(), ValidationError> {
    if validator::validate_ip(s) {
        Ok(())
    } else {
        error!("Validation failed: '{}' is not a valid IPv4", s);
        Err(ValidationError::new("Invalid IPv4 address provided"))
    }
}

fn validate_api_token(token: Option<&str>) -> Result<(), ValidationError>{
    // TODO: should be only alphanumeric?
    if token.is_some() && token.unwrap().len() < MINIMAL_API_TOKEN_LENGTH {
        Err(ValidationError::new("The validation token is too short"))
    } else {
        Ok(())
    }
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Host {
    #[validate(custom="validate_ipv4_address")]
    pub ip: String,
    pub port: u16,
}

fn parse_host(s: &str) -> Result<Host, String> {
    if !validator::validate_ip_v4(s) {
        return Err(format!(
            "Given string {} is not a valid host, Example: {}:{}",
            s,
            DEFAULT_HOST.to_string(),
            DEFAULT_PORT.to_string()
        ));
    }

    Host::from_ipv4_host_string(s)
}

impl Host {
    fn from_ipv4_host_string(s: &str) -> Result<Self, String> {
        let (ip, str_port) = match s.split_once(":") {
            None => return Err(format!("Invalid host")),
            Some(split) => split,
        };

        let port = u16::from_str_radix(str_port, 10).map_err(|e| e.to_string())?;

        Ok(Self {
            ip: ip.to_owned(),
            port,
        })
    }
}

impl ToString for Host {
    fn to_string(&self) -> String {
        format!("{}:{}", self.ip, self.port)
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


#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Auth {
    None,
    Token
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Api {
    pub enabled: bool,
    pub auth: Auth,
    pub token: Option<String>,
    pub host: Host,
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct HealthCheck {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Features {
    pub interval: u32,
    pub threshold: u32,
    pub variance: u32,
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Heartbeat {
    pub interval: u32,
    pub threshold: u32,
    pub variance: u32,
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Network {
    pub announce: bool,
    pub heartbeat: Heartbeat,
    pub allow_local_node_connections: bool,
    pub allow_private_node_connections: bool,
    pub max_parallel_connections: u32,
    pub network_quality_threshold: f32,
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Chain {
    pub provider: Option<String>,
    pub check_unrealized_balance: bool,
    pub on_chain_confirmations: u32,
}


#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize,  Validate, Clone)]
pub struct Strategy {
    pub name: Option<String>,
    pub max_auto_channels: Option<u32>,
    pub auto_redeem_tickets: bool,
}


#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Identity {
    pub file: String,       // path
    pub password: Option<String>,
    pub private_key: Option<Box<[u8]>>,
}

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Db {
    pub data: String,       // data dir path
    pub init: bool,
    pub force_init: bool,
}



#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Testing {
    pub announce_local_addresses: bool,
    pub prefer_local_addresses: bool,
    pub use_weak_crypto: bool,
    pub no_direct_connections: bool,
    pub no_webrtc_upgrade: bool,
    pub local_mode_stun: bool,
}


#[wasm_bindgen_if(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HoprdConfig {
    pub host: Host,
    pub db: Db,
    pub api: Api,
    pub network: Network,
    pub healthcheck: HealthCheck,
    pub features: Features,
    pub environment: String,
    pub chain: Chain,

    pub test: Testing,

    // arguments that are not options, but rather a config behavior
    pub dry_run: bool,
}
