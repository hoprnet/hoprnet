mod decoder;
mod encoder;
mod errors;
mod surb_store;
mod tbf;
mod ticket_processing;
mod traits;
mod types;

pub use decoder::HoprDecoder;
pub use encoder::HoprEncoder;
pub use errors::*;
pub use surb_store::{MemorySurbStore, SurbStoreConfig};
pub use ticket_processing::{HoprTicketProcessor, HoprTicketProcessorConfig, TicketIndexTracker};
pub use traits::*;
pub use types::*;

/// Configuration of [`HoprEncoder`] and [`HoprDecoder`].
#[derive(Clone, Copy, Debug, smart_default::SmartDefault)]
pub struct HoprCodecConfig {
    pub outgoing_ticket_price: Option<hopr_primitive_types::balance::HoprBalance>,
    #[default(Some(hopr_internal_types::prelude::WinningProbability::ALWAYS))]
    pub outgoing_win_prob: Option<hopr_internal_types::prelude::WinningProbability>,
    pub channels_dst: hopr_crypto_types::prelude::Hash,
}

pub mod prelude {
    pub use super::*;
}
