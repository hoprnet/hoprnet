/// Contains all errors thrown from this crate.
pub mod errors;

pub mod session;

/// Contains UDP socket related helpers.
#[cfg(feature = "runtime-tokio")]
pub mod udp;

#[doc(hidden)]
pub mod prelude {
    pub use crate::session::*;

    #[cfg(feature = "runtime-tokio")]
    pub use crate::udp::*;
}
