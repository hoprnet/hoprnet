//! This Rust crate contains implementation of common random number generation functions.
//! All functions and types from this crate supply cryptographically secure random numbers.
//!
//! Instead of relying on external crates, all HOPR crates in this monorepo should
//! exclusively rely on randomness functions only from this crate.

use generic_array::{ArrayLength, GenericArray};
use rand::CryptoRng;
pub use rand::{Rng, RngCore};

/// Maximum random integer that can be generated.
/// This is the last positive 64-bit value in the two's complement representation.
pub const MAX_RANDOM_INTEGER: u64 = 9007199254740991;

/// Gets the default cryptographically secure random number generator.
///
/// **WARNING** On debug builds with the ` fixed-rng ` feature enabled during
/// compilation, this function will return an RNG with a fixed seed, which is *NOT SECURE*!
/// This is reserved for deterministic testing.
#[cfg(all(debug_assertions, feature = "fixed-rng"))]
#[inline]
pub fn rng() -> impl RngCore + CryptoRng {
    use rand::SeedableRng;
    rand::rngs::StdRng::from_seed([
        0x5f, 0x57, 0xce, 0x2a, 0x84, 0x14, 0x7e, 0x88, 0x43, 0x56, 0x44, 0x56, 0x7f, 0x90, 0x4f, 0xb2, 0x04, 0x6b,
        0x18, 0x42, 0x75, 0x69, 0xbe, 0x53, 0xb2, 0x29, 0x78, 0xbd, 0xf3, 0x0a, 0xda, 0xba,
    ])
}

/// Gets the default cryptographically secure random number generator.
#[cfg(any(not(debug_assertions), not(feature = "fixed-rng")))]
#[inline]
pub fn rng() -> impl RngCore + CryptoRng {
    rand::rngs::OsRng
}

/// Returns `true` if the build is using an **insecure** RNG with a fixed seed.
///
/// See also [`rng`].
#[inline]
pub const fn is_rng_fixed() -> bool {
    cfg!(debug_assertions) && cfg!(feature = "fixed-rng")
}

/// Generates a random float uniformly distributed between 0 (inclusive) and 1 (exclusive).
#[inline]
pub fn random_float() -> f64 {
    rng().r#gen()
}

/// Generates a random float uniformly distributed in the given range.
#[inline]
pub fn random_float_in_range(range: std::ops::Range<f64>) -> f64 {
    rng().gen_range(range)
}

/// Generates a random unsigned integer which is at least `start` and optionally strictly less than `end`.
/// If `end` is not given, the ` MAX_RANDOM_INTEGER ` value is used.
/// The caller must make sure that 0 <= `start` < `end` <= `MAX_RANDOM_INTEGER`, otherwise the function will panic.
pub fn random_integer(start: u64, end: Option<u64>) -> u64 {
    let real_end = end.unwrap_or(MAX_RANDOM_INTEGER);

    assert!(
        real_end > start && real_end <= MAX_RANDOM_INTEGER,
        "bounds must be 0 < {start} < {real_end} <= {MAX_RANDOM_INTEGER}"
    );

    let bound = real_end - start;
    start + rng().gen_range(0..bound)
}

/// Fills the specific number of bytes starting from the given offset in the given buffer.
#[inline]
pub fn random_fill(buffer: &mut [u8]) {
    rng().fill_bytes(buffer);
}

/// Allocates an array of the given size and fills it with random bytes
pub fn random_bytes<const T: usize>() -> [u8; T] {
    let mut ret = [0u8; T];
    random_fill(&mut ret);
    ret
}

/// Allocates `GenericArray` of the given size and fills it with random bytes
pub fn random_array<L: ArrayLength>() -> GenericArray<u8, L> {
    let mut ret = GenericArray::default();
    random_fill(&mut ret);
    ret
}

/// Trait for types that can be randomly generated.
pub trait Randomizable {
    /// Generates random value of this type using a cryptographically strong RNG.
    fn random() -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_integer() {
        assert!(random_integer(10, None) > 10);

        let bounded = random_integer(10, Some(20));
        assert!((10..20).contains(&bounded));
    }

    #[test]
    fn test_random_float() {
        let f = random_float();
        assert!((0.0..1.0).contains(&f));
    }

    #[test]
    fn test_random_fill() {
        let mut buffer = [0u8; 10];
        // 7 bytes with indices 2,3,4,5,6,7,8 will be filled with random bytes, the other stay zero
        random_fill(&mut buffer[2..9]);
        assert_eq!(0, buffer[0]);
        assert_eq!(0, buffer[1]);
        assert_eq!(0, buffer[9]);
    }
}
