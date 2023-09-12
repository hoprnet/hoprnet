use std::str::FromStr;

use serde::{Serialize, Deserialize};
use validator::{Validate, ValidationError};

use crate::Strategies;


#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct StrategyConfig {
    #[validate(custom = "validate_strategy_name")]
    pub name: String,
    pub max_auto_channels: Option<u32>,
    pub auto_redeem_tickets: bool,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            name: "passive".to_owned(),
            max_auto_channels: None,
            auto_redeem_tickets: true,
        }
    }
}

fn validate_strategy_name(s: &str) -> Result<(), ValidationError> {
    Strategies::from_str(s)
        .map(|_| ())
        .map_err(|_| ValidationError::new("Invalid strategy name"))
}