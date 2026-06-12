/// Contains all errors thrown from this crate.
pub mod errors;

/// Contains UDP socket-related helpers.
#[cfg(feature = "network-types-runtime-tokio")]
pub mod udp;

/// Contains various networking-related types
pub mod types;

/// Various network IO-related utilities
pub mod utils;

#[cfg(feature = "network-types-capture")]
pub mod capture;

pub mod addr;
pub mod timeout;

#[doc(hidden)]
pub mod prelude {
    pub use libp2p_identity::PeerId;

    #[cfg(feature = "network-types-runtime-tokio")]
    pub use super::udp::*;
    pub use super::{addr::*, types::*};
}
