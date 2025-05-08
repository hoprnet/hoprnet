//! Temporal quantification.
//!
//! This submodule wraps the types in `std::time` so we can implement traits on
//! them. Each type can be converted to-and-from their respective counterparts.

mod duration;
mod instant;

pub use duration::Duration;
pub use instant::Instant;
