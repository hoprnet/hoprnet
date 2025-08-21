//! This crate contains basic types used throughout the entire HOPR codebase.
//! Types from this crate are not necessarily specific only to HOPR.

/// Contains implementations for the token and coin balance types.
pub mod balance;
/// Contains various size-bounded types
pub mod bounded;
/// Lists all errors in this crate.
pub mod errors;
/// Implements the most primitive types, such as [U256](crate::primitives::U256) or
/// [Address](crate::primitives::Address).
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

/// Converts the given `data` into a hex string, removing the middle part of the string if it is
/// longer than `M` of hex characters.
///
/// The returned string is at most `M` characters-long.
pub fn to_hex_shortened<const M: usize>(data: &impl AsRef<[u8]>) -> String {
    let num_chars = M.max(4);
    let data = data.as_ref();
    if data.len() * 2 > M {
        format!(
            "{}..{}",
            hex::encode(&data[0..(num_chars - 2) / 4]),
            hex::encode(&data[data.len() - (num_chars - 2) / 4..])
        )
    } else {
        hex::encode(data)
    }
}

pub mod prelude {
    pub use chrono::{DateTime, Utc};

    pub use super::{
        balance::*, errors::GeneralError, f64_approx_eq, primitives::*, sma::*, to_hex_shortened, traits::*,
    };
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn test_to_hex_shortened() {
        assert_eq!(&to_hex_shortened::<0>(&hex!("deadbeefcafe")), "..");
        assert_eq!(&to_hex_shortened::<1>(&hex!("deadbeefcafe")), "..");
        assert_eq!(&to_hex_shortened::<2>(&hex!("deadbeefcafe")), "..");
        assert_eq!(&to_hex_shortened::<3>(&hex!("deadbeefcafe")), "..");
        assert_eq!(&to_hex_shortened::<4>(&hex!("deadbeefcafe")), "..");
        assert_eq!(&to_hex_shortened::<5>(&hex!("deadbeefcafe")), "..");
        assert_eq!(&to_hex_shortened::<6>(&hex!("deadbeefcafe")), "de..fe");
        assert_eq!(&to_hex_shortened::<7>(&hex!("deadbeefcafe")), "de..fe");
        assert_eq!(&to_hex_shortened::<8>(&hex!("deadbeefcafe")), "de..fe");
        assert_eq!(&to_hex_shortened::<9>(&hex!("deadbeefcafe")), "de..fe");
        assert_eq!(&to_hex_shortened::<10>(&hex!("deadbeefcafe")), "dead..cafe");
        assert_eq!(&to_hex_shortened::<11>(&hex!("deadbeefcafe")), "dead..cafe");
        assert_eq!(&to_hex_shortened::<12>(&hex!("deadbeefcafe")), "deadbeefcafe");
    }
}
