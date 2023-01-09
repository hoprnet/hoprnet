use digest::{Digest, FixedOutputReset};
use sha3::Keccak256;
use crate::errors::CryptoError::InvalidInputValue;
use crate::errors::Result;

pub fn iterate_hash(seed: &[u8], iterations: usize, step_size: usize) -> Result<Vec<Box<[u8]>>> {
    let mut intermediates: Vec<Box<[u8]>> = Vec::with_capacity(iterations);

    if seed.len() == 0 || iterations == 0 {
        return Err(InvalidInputValue)
    }

    let mut current: Box<[u8]> = Box::from(seed);
    let mut hash = Keccak256::default();

    for i in 0..iterations {
        if i % step_size == 0 {
            intermediates.push(current.clone());
        }

        hash.update(current.as_ref());
        let new_intermediate = hash.finalize_fixed_reset().to_vec();
        current = new_intermediate.into_boxed_slice();
    }

    intermediates.push(current.clone());

    Ok(intermediates)
}