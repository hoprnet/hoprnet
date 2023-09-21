use std::str::FromStr;

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};
use utils_types::primitives::Balance;
use crate::Strategies;

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct StrategyConfig {
    #[validate(custom = "validate_strategy_name")]
    pub name: String,

    /// Maximum number of opened channels the strategy should maintain.
    /// Defaults to square-root of the sampled network size.
    pub max_auto_channels: Option<u32>,
    pub auto_redeem_tickets: bool,

    /// A quality threshold between 0 and 1 used to determine whether the strategy should open channel with the peer.
    /// Defaults to 0.5
    pub network_quality_threshold: Option<f64>,

    /// A stake of tokens that should be allocated to a channel opened by the strategy.
    /// Defaults to 0.1 HOPR
    pub new_channel_stake: Option<Balance>,

    /// A minimum channel token stake. If reached, the channel will be closed and re-opened with `new_channel_stake`.
    /// Defaults to 0.01 HOPR
    pub minimum_channel_balance: Option<Balance>,

    /// Minimum token balance of the node. When reached, the strategy will not open any new channels.
    /// Defaults to 0.01 HOPR
    pub minimum_node_balance: Option<Balance>,

    /// If set, the strategy will aggressively close channels (even with peers above the `network_quality_threshold`)
    /// if the number of opened outgoing channels (regardless if opened by the strategy or manually) exceeds the
    /// `max_channels` limit.
    /// Defaults to true
    pub enforce_max_channels: Option<bool>,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            name: "passive".to_owned(),
            max_auto_channels: None,
            auto_redeem_tickets: true,
            network_quality_threshold: None,
            new_channel_stake: None,
            minimum_channel_balance: None,
            minimum_node_balance: None,
            enforce_max_channels: None,
        }
    }
}

fn validate_strategy_name(s: &str) -> Result<(), ValidationError> {
    Strategies::from_str(s)
        .map(|_| ())
        .map_err(|_| ValidationError::new("Invalid strategy name"))
}
