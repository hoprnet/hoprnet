use hopr_primitive_types::prelude::HoprBalance;
use hopr_protocol_hopr::SurbStoreConfig;
use validator::Validate;

/// Configuration of the P2P protocols.
#[cfg_attr(feature = "serde", cfg_eval::cfg_eval, serde_with::serde_as)]
#[derive(Debug, smart_default::SmartDefault, Validate, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(deny_unknown_fields)
)]
pub struct ProtocolConfig {
    /// Winning probability that gets printed on any outgoing tickets.
    /// If not set, the network value is used.
    #[validate(range(min = 0.0, max = 1.0))]
    pub outgoing_ticket_winning_prob: Option<f64>,
    #[cfg_attr(feature = "serde", serde_as(as = "Option<serde_with::DisplayFromStr>"))]
    /// Possible override of the network outgoing ticket price.
    pub outgoing_ticket_price: Option<HoprBalance>,
    /// Configuration of the SURB store.
    pub surb_store: SurbStoreConfig,
}
