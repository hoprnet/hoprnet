use digest::{Digest, FixedOutputReset};
use sha3::Keccak256;

use utils_proc_macros::wasm_bindgen_if;

use crate::errors::CryptoError::{CalculationError, InvalidInputValue};
use crate::errors::Result;

pub struct IteratedHash {
    pub hash: Box<[u8]>,
    pub intermediates: Vec<Intermediate>
}

#[wasm_bindgen_if(getter_with_clone)]
pub struct Intermediate {
    pub iteration: usize,
    pub intermediate: Box<[u8]>,
}

pub fn iterate_hash(seed: &[u8], iterations: usize, step_size: usize) -> Result<IteratedHash> {
    if seed.len() == 0 || iterations == 0 || step_size == 0 {
        return Err(InvalidInputValue)
    }

    let mut intermediates: Vec<Intermediate> = Vec::with_capacity(iterations / step_size + 1);
    let mut intermediate: Box<[u8]> = Box::from(seed);
    let mut hash = Keccak256::default();

    for i in 0..iterations {
        hash.update(intermediate.as_ref());

        if i % step_size == 0 {
            intermediates.push(Intermediate {
                iteration: i,
                intermediate
            });
        }

        let new_intermediate = hash.finalize_fixed_reset().to_vec();
        intermediate = new_intermediate.into_boxed_slice();
    }

    Ok(IteratedHash {
        hash: intermediate,
        intermediates
    })
}


pub fn recover_iterated_hash<H>(hash_value: &[u8], hints: H, max_iterations: usize, step_size: usize, index_hint: Option<usize>) -> Result<Intermediate>
    where H: Fn(usize) -> Option<Box<[u8]>>
{
    if step_size == 0 || hash_value.len() == 0 || max_iterations == 0 {
        return Err(InvalidInputValue)
    }

    let closest_intermediate = index_hint.unwrap_or(max_iterations - ( max_iterations % step_size ) );
    let mut digest = Keccak256::default();

    for i in (0..=closest_intermediate).step_by(step_size) {
        // Check if we can get a hint for the current index
        let pos = closest_intermediate - i;
        if let Some(mut intermediate) = hints(pos) {
            for iteration in 0..(step_size + i){
                // Compute the hash of current intermediate
                digest.update(intermediate.as_ref());
                let hash = digest.finalize_fixed_reset().to_vec();

                // Is the computed hash the one we're looking for ?
                if hash.len() == hash_value.len() && hash == hash_value {
                    return Ok(Intermediate {
                        iteration: iteration + pos,
                        intermediate
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
        let final_hash = iterate_hash(&seed, 1000, 10).unwrap();

        assert_eq!(final_hash.intermediates.len(), 100);
        let final_src = final_hash.intermediates.last().unwrap();

        let hint_idx = 98; // hint is at iteration num. 980
        let hint = &final_hash.intermediates[hint_idx];
        assert_eq!(980, hint.iteration);

        let recovered = recover_iterated_hash(final_hash.hash.as_ref(),
                                              |it: usize| { if it == hint.iteration {
                                                  Some(hint.intermediate.clone())
                                              } else { None } },
                                              1000,
                                              10,
                                              None)
            .unwrap();

        assert_eq!(final_src.iteration, recovered.iteration);
        assert_eq!(final_src.intermediate.as_ref(), recovered.intermediate.as_ref());

    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    /*use js_sys::{Number, Uint8Array};
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
    }*/

}