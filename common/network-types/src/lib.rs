/// Contains all errors thrown from this crate.
pub mod errors;

pub mod session;

/// Contains UDP socket-related helpers.
#[cfg(feature = "runtime-tokio")]
pub mod udp;

/// Contains various networking-related types
pub mod types;

#[doc(hidden)]
pub mod prelude {
    pub use crate::session::*;

    #[cfg(feature = "runtime-tokio")]
    pub use crate::udp::*;

    pub use crate::types::*;
}
