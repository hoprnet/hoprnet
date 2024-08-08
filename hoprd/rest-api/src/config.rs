use std::str::FromStr;

use hopr_lib::HostConfig;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

pub const DEFAULT_API_HOST: &str = "127.0.0.1";
pub const DEFAULT_API_PORT: u16 = 3001;
pub const MINIMAL_API_TOKEN_LENGTH: usize = 8;

fn validate_api_auth(token: &Auth) -> Result<(), ValidationError> {
    match &token {
        Auth::None => Ok(()),
        Auth::Token(token) => {
            if token.len() >= MINIMAL_API_TOKEN_LENGTH {
                Ok(())
            } else {
                Err(ValidationError::new("The API token is too short"))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Serialize, Deserialize)]
pub enum Auth {
    #[default]
    None,
    Token(String),
}

#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Api {
    /// Selects whether the REST API is enabled
    #[serde(default)]
    pub enable: bool,
    /// Auth enum holding the API auth configuration
    #[validate(custom(function = "validate_api_auth"))]
    #[serde(default)]
    pub auth: Auth,
    /// Host and port combination where the REST API should be located
    #[validate(nested)]
    #[serde(default = "default_api_host")]
    #[default(default_api_host())]
    pub host: HostConfig,
}

#[inline]
fn default_api_host() -> HostConfig {
    HostConfig::from_str(format!("{DEFAULT_API_HOST}:{DEFAULT_API_PORT}").as_str()).unwrap()
}
