//! This crate contains basic types used throughout the entire HOPR codebase.
//! Types from this crate are not necessarily specific only to HOPR.

/// Contains various size-bounded types
pub mod bounded;
/// Enumerates all errors in this crate.
pub mod errors;
/// Implements the most primitive types, such as [U256](crate::primitives::U256) or [Address](crate::primitives::Address).
pub mod primitives;
/// Contains various implementations of Simple Moving Average.
pub mod sma;
/// Defines commonly used traits across the entire code base.
pub mod traits;

/// Approximately compares two double-precision floats.
///
/// This function first tests if the two values relatively differ by at least `epsilon`.
/// In case they are equal, the second test checks if they differ by at least two representable
/// units of precision - meaning there can be only two other floats represented in between them.
/// If both tests pass, the two values are considered (approximately) equal.
pub fn f64_approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
    float_cmp::ApproxEq::approx_eq(a, b, (epsilon, 2))
}

pub mod prelude {
    pub use super::errors::GeneralError;
    pub use super::f64_approx_eq;
    pub use super::primitives::*;
    pub use super::sma::*;
    pub use super::traits::*;

    pub use chrono::{DateTime, Utc};
}
