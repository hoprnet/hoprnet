use elliptic_curve::rand_core::OsRng;
use elliptic_curve::{Group, NonZeroScalar, ProjectivePoint};
use generic_array::{ArrayLength, GenericArray};
use k256::Secp256k1;
use rand::{Rng, RngCore};

use crate::errors::CryptoError::InvalidInputValue;
use crate::errors::Result;
use crate::types::CurvePoint;

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
pub fn random_integer(start: u64, end: Option<u64>) -> Result<u64> {
    let real_end = end.unwrap_or(MAX_RANDOM_INTEGER);

    if real_end <= start || real_end > MAX_RANDOM_INTEGER {
        Err(InvalidInputValue)
    } else {
        let bound = real_end - start;
        Ok(start + OsRng.gen_range(0..bound))
    }
}

/// Generates a random elliptic curve point on secp256k1 curve (but not a point in infinity).
/// Returns the encoded secret scalar and the corresponding point.
pub fn random_group_element() -> ([u8; 32], CurvePoint) {
    let mut scalar: NonZeroScalar<Secp256k1> = NonZeroScalar::<Secp256k1>::from_uint(1u32.into()).unwrap();
    let mut point = ProjectivePoint::<Secp256k1>::IDENTITY;
    while point.is_identity().into() {
        scalar = NonZeroScalar::<Secp256k1>::random(&mut OsRng);
        point = ProjectivePoint::<Secp256k1>::GENERATOR * scalar.as_ref();
    }
    (scalar.to_bytes().into(), point.to_affine().into())
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
    use crate::random::{random_fill, random_float, random_group_element, random_integer};
    use crate::types::CurvePoint;
    use elliptic_curve::Group;

    #[test]
    fn test_random_integer() {
        assert!(random_integer(10, None).unwrap() > 10);

        let bounded = random_integer(10, Some(20)).unwrap();
        assert!(bounded >= 10);
        assert!(bounded < 20)
    }

    #[test]
    fn test_random_float() {
        let f = random_float();
        assert!((0.0..1.0).contains(&f));
    }

    #[test]
    fn test_random_element() {
        let (scalar, point) = random_group_element();
        let is_identity: bool = point.to_projective_point().is_identity().into();
        assert!(!is_identity);
        assert_eq!(CurvePoint::from_exponent(&scalar).unwrap(), point);
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

#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::Uint8Array;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    pub fn random_float() -> f64 {
        crate::random::random_float()
    }

    #[wasm_bindgen]
    pub fn random_bounded_integer(bound: u64) -> JsResult<u64> {
        ok_or_jserr!(crate::random::random_integer(0, Some(bound)))
    }

    #[wasm_bindgen]
    pub fn random_big_integer(start: u64, end: Option<u64>) -> JsResult<u64> {
        ok_or_jserr!(crate::random::random_integer(start, end))
    }

    #[wasm_bindgen]
    pub fn random_integer(start: u32, end: Option<u32>) -> JsResult<u32> {
        ok_or_jserr!(crate::random::random_integer(start as u64, end.map(|i| i as u64)).map(|r| r as u32))
    }

    #[wasm_bindgen]
    pub fn random_fill(buffer: Uint8Array, from: usize, len: usize) {
        let mut buf = vec![0u8; buffer.length() as usize];
        buffer.copy_to(buf.as_mut_slice());

        let chunk = buf.as_mut_slice();
        crate::random::random_fill(&mut chunk[from..from + len]);
        buffer.copy_from(buf.as_slice());
    }
}
