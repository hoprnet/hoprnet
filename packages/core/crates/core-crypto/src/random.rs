use elliptic_curve::rand_core::OsRng;
use elliptic_curve::sec1::ToEncodedPoint;
use k256::NonZeroScalar;
use crate::errors::CryptoError::InvalidInputValue;

use crate::errors::Result;

use rand::{Rng, RngCore};

pub const MAX_RANDOM_INTEGER: u64 = 9007199254740991;

pub fn random_float() -> f64 {
    OsRng.gen()
}

pub fn random_bounded_integer(bound: u64) -> u64 {
    OsRng.gen_range(0..bound)
}

pub fn random_integer(start: u64, end: Option<u64>) -> Result<u64> {

    let real_end = end.unwrap_or(MAX_RANDOM_INTEGER);

    if real_end <= start || real_end > MAX_RANDOM_INTEGER {
        Err(InvalidInputValue)
    }
    else {
        Ok(start + random_bounded_integer(real_end - start))
    }
}

pub fn random_group_element(compressed: bool) -> (Box<[u8]>,Box<[u8]>) {

    let scalar = NonZeroScalar::random(&mut OsRng);
    let point = k256::ProjectivePoint::GENERATOR * scalar.as_ref();

    let encoded = point.to_encoded_point(compressed);

    (scalar.to_bytes().as_slice().into() ,encoded.as_bytes().into())
}

pub fn random_fill(buffer: &mut [u8], from: usize, len: usize) {
    assert!(from + len <= buffer.len());
    let mut range = &buffer[from..from+len];
    OsRng.fill_bytes(&mut range);
}

pub mod wasm {

}