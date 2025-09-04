use hopr_primitive_types::prelude::HoprBalance;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use validator::Validate;

/// Configuration of the P2P protocols.
#[serde_as]
#[derive(Debug, smart_default::SmartDefault, Serialize, Deserialize, Validate, Copy, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProtocolConfig {
    /// Winning probability that gets printed on any outgoing tickets.
    /// If not set, the network value is used.
    #[validate(range(min = 0.0, max = 1.0))]
    pub outgoing_ticket_winning_prob: Option<f64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    /// Possible override of the network outgoing ticket price.
    pub outgoing_ticket_price: Option<HoprBalance>,
}
