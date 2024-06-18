/// Contains all errors thrown from this crate.
pub mod errors;

pub mod session;

#[doc(hidden)]
pub mod prelude {
    pub use crate::session::*;
}
