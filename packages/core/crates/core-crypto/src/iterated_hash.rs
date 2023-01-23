use digest::{Digest, FixedOutputReset};
use sha3::Keccak256;
use serde::{Deserialize, Serialize};
use utils_proc_macros::wasm_bindgen_if;

use crate::errors::CryptoError::{CalculationError, InvalidInputValue};
use crate::errors::Result;

#[wasm_bindgen_if(getter_with_clone)]
#[derive(Deserialize, Serialize)]
pub struct HashIteration {
    pub iteration: usize,

    #[serde(with = "serde_bytes")]
    pub intermediate: Vec<u8>,
}

pub fn iterate_hash(seed: &[u8], iterations: usize, step_size: usize) -> Result<Vec<HashIteration>> {
    if seed.len() == 0 || iterations == 0 || step_size == 0 {
        return Err(InvalidInputValue)
    }

    let mut intermediates: Vec<HashIteration> = Vec::with_capacity(iterations / step_size + 1);
    let mut current: Box<[u8]> = Box::from(seed);
    let mut hash = Keccak256::default();

    for i in 0..iterations {
        if i % step_size == 0 {
            intermediates.push(HashIteration {
                iteration: i,
                intermediate: current.to_vec()
            });
        }

        hash.update(current.as_ref());
        let new_intermediate = hash.finalize_fixed_reset().to_vec();
        current = new_intermediate.into_boxed_slice();
    }

    Ok(intermediates)
}


pub fn recover_iterated_hash<H>(hash_value: &[u8], hints: H, max_iterations: usize, step_size: usize, index_hint: Option<usize>) -> Result<HashIteration>
    where H: Fn(u32) -> Option<Box<[u8]>>
{
    if step_size == 0 || hash_value.len() == 0 || max_iterations == 0 {
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
                    return Ok(HashIteration {
                        iteration: iteration + i as usize,
                        intermediate: intermediate.to_vec()
                    });
                }

                intermediate = hash.into_boxed_slice();
            }
        }
    }

    Err(CalculationError)
}

#[cfg(test)]
mod tests {
    use crate::iterated_hash::{iterate_hash, recover_iterated_hash};

    #[test]
    fn test_iteration() {
        let seed = [1u8; 16];
        let hashes = iterate_hash(&seed, 1000, 10).unwrap();

        assert_eq!(hashes.len(), 100);

        let hint_idx = 98;

        let last = hashes.last().unwrap();
        let middle = &hashes[hint_idx];

        let recovered = recover_iterated_hash(last.intermediate.as_slice(),
                                              |it: u32| { if it == middle.iteration as u32 {
                                                  Some(middle.intermediate.clone().into_boxed_slice())
                                              } else { None } },
                                              1000,
                                              10,
                                              None)
            .unwrap();

    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::{Number, Uint8Array};
    use wasm_bindgen::JsValue;
    use wasm_bindgen::prelude::wasm_bindgen;
    use utils_misc::utils::wasm::JsResult;
    use utils_misc::ok_or_jserr;

    #[wasm_bindgen]
    pub fn iterate_hash(seed: &[u8], iterations: usize, step_size: usize) -> JsResult<JsValue> {
        let res = ok_or_jserr!(super::iterate_hash(seed, iterations, step_size))?;
        ok_or_jserr!(serde_wasm_bindgen::to_value(&res))
    }

    #[wasm_bindgen]
    pub fn recover_iterated_hash(hash_value: &[u8], hints: &js_sys::Function, max_iterations: usize, step_size: usize, index_hint: Option<usize>) -> JsResult<JsValue> {
        let res = ok_or_jserr!(super::recover_iterated_hash(hash_value, |iteration: u32| {
            hints
                .call1(&JsValue::null(), &Number::from(iteration))
                .ok()
                .map(|h| Uint8Array::from(h).to_vec().into_boxed_slice())
        }, max_iterations, step_size, index_hint))?;
        ok_or_jserr!(serde_wasm_bindgen::to_value(&res))
    }

}