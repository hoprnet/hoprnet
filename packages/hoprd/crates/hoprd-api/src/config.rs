use std::str::FromStr;

use hopr_lib::HostConfig;
use serde::{Serialize, Deserialize};
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
                Err(ValidationError::new("The validation token is too short"))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Auth {
    None,
    Token(String),
}

#[derive(Debug, Validate, Serialize, Deserialize, Clone, PartialEq)]
pub struct Api {
    pub enable: bool,
    /// Auth enum holding the API auth configuration
    #[validate(custom = "validate_api_auth")]
    pub auth: Auth,
    #[validate]
    pub host: HostConfig,
}

impl Default for Api {
    fn default() -> Self {
        Self {
            enable: false,
            auth: Auth::None,
            host: HostConfig::from_str(format!("{DEFAULT_API_HOST}:{DEFAULT_API_PORT}").as_str()).unwrap(),
        }
    }
}