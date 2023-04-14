use crate::errors::CryptoError::{CalculationError, InvalidInputValue};
use crate::errors::Result;
use crate::primitives::{DigestLike, EthDigest};

/// Contains the complete hash iteration progression
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IteratedHash {
    pub hash: Box<[u8]>,
    pub intermediates: Vec<Intermediate>,
}

/// Contains the intermediate result in the hash iteration progression
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Intermediate {
    pub iteration: usize,
    pub intermediate: Box<[u8]>,
}

/// Performs hash iteration progression from the given seed, the total number of iteration and step size.
/// The Keccak256 digest is used to perform the hash iteration.
pub fn iterate_hash(seed: &[u8], iterations: usize, step_size: usize) -> IteratedHash {
    assert!(!seed.is_empty() && step_size > 0 && iterations > step_size);

    let mut intermediates: Vec<Intermediate> = Vec::with_capacity(iterations / step_size + 1);
    let mut intermediate: Box<[u8]> = EthDigest::create_output();

    let mut hash = EthDigest::default();

    // Unroll the first iteration, because it uses different input length
    hash.update(seed);
    hash.finalize_into(&mut intermediate);
    intermediates.push(Intermediate {
        iteration: 0,
        intermediate: seed.into(),
    });

    for iteration in 1..iterations {
        hash.update(intermediate.as_ref());

        if iteration % step_size == 0 {
            intermediates.push(Intermediate {
                iteration,
                intermediate: intermediate.clone(),
            });
        }

        hash.finalize_into(&mut intermediate);
    }

    IteratedHash {
        hash: intermediate,
        intermediates,
    }
}

/// Recovers the iterated hash pre-image for the given hash value.
/// Hints can be given if some intermediates in the progression are known via the hints lookup function.
/// The number of iterations and step size correspond to the values with which the progression was originally created.
/// The Keccak256 digest is used to perform the hash iteration.
pub fn recover_iterated_hash<H>(
    hash_value: &[u8],
    hints: H,
    max_iterations: usize,
    step_size: usize,
    index_hint: Option<usize>,
) -> Result<Intermediate>
where
    H: Fn(usize) -> Option<Box<[u8]>>,
{
    if step_size == 0 || hash_value.is_empty() || max_iterations == 0 {
        return Err(InvalidInputValue);
    }

    let closest_intermediate = index_hint.unwrap_or(max_iterations - (max_iterations % step_size));
    let mut digest = EthDigest::default();

    for i in (0..=closest_intermediate).step_by(step_size) {
        // Check if we can get a hint for the current index
        let pos = closest_intermediate - i;
        if let Some(mut intermediate) = hints(pos) {
            for iteration in 0..step_size {
                // Compute the hash of current intermediate
                digest.update(intermediate.as_ref());
                let hash = digest.finalize();

                // Is the computed hash the one we're looking for ?
                if hash.len() == hash_value.len() && hash.as_ref() == hash_value {
                    return Ok(Intermediate {
                        iteration: iteration + pos,
                        intermediate,
                    });
                }

                intermediate = hash;
            }
        }
    }

    Err(CalculationError)
}

#[cfg(test)]
mod tests {
    use crate::iterated_hash::{iterate_hash, recover_iterated_hash};
    use crate::types::Hash;
    use hex_literal::hex;
    use utils_types::traits::BinarySerializable;

    #[test]
    fn test_iteration() {
        let seed = [0u8; 16];
        let final_hash = iterate_hash(&seed, 1000, 10);
        assert_eq!(final_hash.intermediates.len(), 100);

        let hint = &final_hash.intermediates[98]; // hint is at iteration num. 980
        assert_eq!(980, hint.iteration);

        let expected = hex!("a380d145d8612d33912494f1b36571c0b59b9bd459e6bb7d5ea05946be4c256b");
        assert_eq!(&expected, hint.intermediate.as_ref())
    }

    #[test]
    fn test_recovery() {
        let hint_idx = 980;
        let hint_hash = hex!("a380d145d8612d33912494f1b36571c0b59b9bd459e6bb7d5ea05946be4c256b");
        let target = hex!("16db2cf9913ccaf4976c825d1fd7b8f8e6510f479390c3272ed4e27fa015a537");

        let recovered = recover_iterated_hash(
            &target,
            |i| {
                if i == hint_idx {
                    Some(Box::new(hint_hash))
                } else {
                    None
                }
            },
            1000,
            10,
            None,
        )
        .unwrap();

        assert_eq!(recovered.iteration, hint_idx + 7);
        assert_eq!(
            Hash::create(&[recovered.intermediate.as_ref()]).serialize().as_ref(),
            &target
        );
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::iterated_hash::Intermediate;
    use js_sys::{Number, Uint8Array};
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen]
    pub struct IteratedHash {
        w: super::IteratedHash,
    }

    #[wasm_bindgen]
    impl IteratedHash {
        pub fn hash(&self) -> Box<[u8]> {
            self.w.hash.clone()
        }

        pub fn count_intermediates(&self) -> usize {
            self.w.intermediates.len()
        }

        pub fn intermediate(&self, index: usize) -> Option<Intermediate> {
            self.w.intermediates.get(index).cloned()
        }
    }

    #[wasm_bindgen]
    pub fn iterate_hash(seed: &[u8], iterations: usize, step_size: usize) -> IteratedHash {
        IteratedHash {
            w: super::iterate_hash(seed, iterations, step_size),
        }
    }

    #[wasm_bindgen]
    pub fn recover_iterated_hash(
        hash_value: &[u8],
        hints: &js_sys::Function,
        max_iterations: usize,
        step_size: usize,
        index_hint: Option<usize>,
    ) -> JsResult<Intermediate> {
        ok_or_jserr!(super::recover_iterated_hash(
            hash_value,
            |iteration: usize| {
                hints
                    .call1(&JsValue::null(), &Number::from(iteration as u32))
                    .ok()
                    .and_then(|preimage| {
                        let arr = Uint8Array::from(preimage);
                        if !arr.is_undefined() {
                            Some(arr.to_vec().into_boxed_slice())
                        } else {
                            None
                        }
                    })
            },
            max_iterations,
            step_size,
            index_hint
        ))
    }
}
