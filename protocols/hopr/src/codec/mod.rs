mod decoder;
mod encoder;

pub use decoder::HoprDecoder;
pub use encoder::HoprEncoder;

fn validate_outgoing_ticket_price(
    price: &hopr_primitive_types::balance::HoprBalance,
) -> Result<(), validator::ValidationError> {
    if price.is_zero() {
        Err(validator::ValidationError::new("outgoing_ticket_price cannot be zero"))
    } else {
        Ok(())
    }
}

fn default_outgoing_win_prob() -> Option<hopr_internal_types::prelude::WinningProbability> {
    Some(hopr_internal_types::prelude::WinningProbability::ALWAYS)
}

/// Configuration of [`HoprEncoder`] and [`HoprDecoder`].
#[cfg_attr(feature = "serde", cfg_eval::cfg_eval, serde_with::serde_as)]
#[derive(Clone, Copy, Debug, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct HoprCodecConfig {
    /// Optional price of outgoing tickets.
    ///
    /// If not set, the network default will be used, which is the minimum allowed ticket price in the HOPR network.
    #[cfg_attr(
        feature = "serde",
        serde(default),
        serde_as(as = "Option<serde_with::DisplayFromStr>")
    )]
    #[validate(custom(function = "validate_outgoing_ticket_price"))]
    pub outgoing_ticket_price: Option<hopr_primitive_types::balance::HoprBalance>,
    /// Optional probability of winning an outgoing ticket.
    ///
    /// If not set, the network default will be used, which is the minimum allowed winning probability in the HOPR
    /// network.
    ///
    /// The default is [`WinningProbability::ALWAYS`](hopr_internal_types::prelude::WinningProbability::ALWAYS).
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_outgoing_win_prob"),
        serde_as(as = "Option<serde_with::DisplayFromStr>")
    )]
    #[default(default_outgoing_win_prob())]
    pub outgoing_win_prob: Option<hopr_internal_types::prelude::WinningProbability>,
}

impl PartialEq for HoprCodecConfig {
    fn eq(&self, other: &Self) -> bool {
        self.outgoing_ticket_price.eq(&other.outgoing_ticket_price)
            && match (self.outgoing_win_prob, other.outgoing_win_prob) {
                (Some(a), Some(b)) => a.approx_eq(&b),
                (None, None) => true,
                _ => false,
            }
    }
}
