use digest::{Digest, FixedOutputReset};
use sha3::Keccak256;
use crate::errors::CryptoError::{CalculationError, InvalidInputValue};
use crate::errors::Result;

pub struct HashIteration {
    pub iteration: usize,
    pub intermediate: Box<[u8]>
}

pub fn iterate_hash(seed: &[u8], iterations: usize, step_size: usize) -> Result<Vec<HashIteration>> {
    let mut intermediates: Vec<HashIteration> = Vec::with_capacity(iterations);

    if seed.len() == 0 || iterations == 0 {
        return Err(InvalidInputValue)
    }

    let mut current: Box<[u8]> = Box::from(seed);
    let mut hash = Keccak256::default();

    for i in 0..iterations {
        if i % step_size == 0 {
            intermediates.push(HashIteration {
                iteration: i,
                intermediate: current.clone()
            });
        }

        hash.update(current.as_ref());
        let new_intermediate = hash.finalize_fixed_reset().to_vec();
        current = new_intermediate.into_boxed_slice();
    }

    intermediates.push( HashIteration {
        iteration: iterations,
        intermediate: current.clone()
    });

    Ok(intermediates)
}


pub fn recover_iterated_hash<H>(hash_value: &[u8], hints: H, max_iterations: usize, step_size: usize, index_hint: Option<usize>) -> Result<HashIteration>
    where H: Fn(u32) -> Option<Box<[u8]>>
{
    if step_size == 0 {
        return Err(InvalidInputValue)
    }

    let closest_intermediate = index_hint.unwrap_or(max_iterations - ( max_iterations % step_size ) );
    let mut digest = Keccak256::default();

    for i in (0..closest_intermediate as u32 + 1).step_by(step_size).rev() {
        // Check if we can get a hint for the current index
        if let Some(mut intermediate) = hints(i) {
            for iteration in 0..step_size {
                // Compute the hash of current intermediate
                digest.update(intermediate.as_ref());
                let hash = digest.finalize_fixed_reset().to_vec();

                // Is the computed hash the one we're looking for ?
                if hash.len() == hash_value.len() && hash == hash_value {
                    return Ok(HashIteration { iteration, intermediate });
                }

                intermediate = hash.into_boxed_slice();
            }
        }
    }

    Err(CalculationError)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_iteration() {

    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::wasm_bindgen;

    pub struct HashIteration {
        w: super::HashIteration
    }


}