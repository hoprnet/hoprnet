/// Contains all errors thrown from this crate.
pub mod errors;

/// Contains UDP socket-related helpers.
#[cfg(feature = "runtime-tokio")]
pub mod udp;

/// Contains various networking-related types
pub mod types;

/// Various network IO-related utilities
pub mod utils;

#[cfg(feature = "capture")]
pub mod capture;
mod timeout;

#[doc(hidden)]
pub mod prelude {
    pub use libp2p_identity::PeerId;

    #[cfg(feature = "runtime-tokio")]
    pub use super::udp::*;
    pub use super::{timeout::*, types::*};
}
