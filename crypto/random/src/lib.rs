use generic_array::{ArrayLength, GenericArray};
use rand::{Rng, RngCore};

pub use rand::rngs::OsRng;

/// Maximum random integer that can be generated.
/// This is the last positive 64-bit value in the two's complement representation.
pub const MAX_RANDOM_INTEGER: u64 = 9007199254740991;

/// Generates a random float uniformly distributed between 0 (inclusive) and 1 (exclusive).
#[inline]
pub fn random_float() -> f64 {
    OsRng.gen()
}

/// Generates random unsigned integer which is at least `start` and optionally strictly less than `end`.
/// If `end` is not given, `MAX_RANDOM_INTEGER` value is used.
/// The caller must make sure that 0 <= `start` < `end` <= `MAX_RANDOM_INTEGER`, otherwise the function will panic.
pub fn random_integer(start: u64, end: Option<u64>) -> u64 {
    let real_end = end.unwrap_or(MAX_RANDOM_INTEGER);

    assert!(real_end > start && real_end <= MAX_RANDOM_INTEGER, "invalid bounds");

    let bound = real_end - start;
    start + OsRng.gen_range(0..bound)
}

/// Fills the specific number of bytes starting from the given offset in the given buffer.
#[inline]
pub fn random_fill(buffer: &mut [u8]) {
    OsRng.fill_bytes(buffer);
}

/// Allocates array of the given size and fills it with random bytes
pub fn random_bytes<const T: usize>() -> [u8; T] {
    let mut ret = [0u8; T];
    random_fill(&mut ret);
    ret
}

/// Allocates `GenericArray` of the given size and fills it with random bytes
pub fn random_array<L: ArrayLength<u8>>() -> GenericArray<u8, L> {
    let mut ret = GenericArray::default();
    random_fill(&mut ret);
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_integer() {
        assert!(random_integer(10, None) > 10);

        let bounded = random_integer(10, Some(20));
        assert!(bounded >= 10);
        assert!(bounded < 20)
    }

    #[test]
    fn test_random_float() {
        let f = random_float();
        assert!((0.0..1.0).contains(&f));
    }
    
    #[test]
    fn test_random_fill() {
        let mut buffer = [0u8; 10];
        // 7 bytes with indices 2,3,4,5,6,7,8 will be filled with random bytes, other stay zero
        random_fill(&mut buffer[2..9]);
        assert_eq!(0, buffer[0]);
        assert_eq!(0, buffer[1]);
        assert_eq!(0, buffer[9]);
    }
}
