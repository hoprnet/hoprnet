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

/// Configuration of [`HoprEncoder`] and [`HoprDecoder`].
#[derive(Clone, Copy, Debug, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HoprCodecConfig {
    /// Optional price of outgoing tickets.
    ///
    /// If not set, the network default will be used, which is the minimum allowed ticket price in the HOPR network.
    #[validate(custom(function = "validate_outgoing_ticket_price"))]
    pub outgoing_ticket_price: Option<hopr_primitive_types::balance::HoprBalance>,
    /// Optional probability of winning an outgoing ticket.
    ///
    /// If not set, the network default will be used, which is the minimum allowed winning probability in the HOPR
    /// network.
    ///
    /// The default is [`WinningProbability::ALWAYS`](hopr_internal_types::prelude::WinningProbability::ALWAYS).
    #[default(Some(hopr_internal_types::prelude::WinningProbability::ALWAYS))]
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
