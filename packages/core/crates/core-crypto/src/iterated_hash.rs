use generic_array::GenericArray;
use serde::{Deserialize, Serialize};

use crate::errors::CryptoError::{CalculationError, InvalidInputValue};
use crate::errors::Result;
use crate::primitives::{DigestLike, EthDigest, SimpleMac};

/// Contains the complete hash iteration progression
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IteratedHash {
    pub hash: [u8; SimpleMac::SIZE],
    pub intermediates: Vec<Intermediate>,
}

/// Contains the intermediate result in the hash iteration progression
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Intermediate {
    pub iteration: usize,
    pub intermediate: Box<[u8]>,
}

/// Performs hash iteration progression from the given seed, the total number of iteration and step size.
/// The Keccak256 digest is used to perform the hash iteration.
/// As this is potentially a long running function, it is asynchronous and yields at every `step_size` iterations.
pub async fn iterate_hash(seed: &[u8], iterations: usize, step_size: usize) -> IteratedHash {
    assert!(!seed.is_empty() && step_size > 0 && iterations > step_size);

    let mut intermediates: Vec<Intermediate> = Vec::with_capacity(iterations / step_size + 1);

    let mut hash = EthDigest::default();

    // Unroll the first iteration, because it uses different input length
    hash.update(seed);
    let mut intermediate = hash.finalize();

    intermediates.push(Intermediate {
        iteration: 0,
        intermediate: seed.into(),
    });

    for iteration in 1..iterations {
        hash.update(intermediate.as_ref());

        if iteration % step_size == 0 {
            intermediates.push(Intermediate {
                iteration,
                intermediate: intermediate.as_slice().into(),
            });
            async_std::task::yield_now().await;
        }

        hash.finalize_into(&mut intermediate);
    }

    IteratedHash {
        hash: intermediate.into(),
        intermediates,
    }
}

/// Recovers the iterated hash pre-image for the given hash value.
/// Hints can be given if some intermediates in the progression are known via the hints lookup function.
/// The number of iterations and step size correspond to the values with which the progression was originally created.
/// The Keccak256 digest is used to perform the hash iteration.
pub async fn recover_iterated_hash<F>(
    hash_value: &[u8],
    hints: &impl Fn(usize) -> F,
    max_iterations: usize,
    step_size: usize,
    index_hint: Option<usize>,
) -> Result<Intermediate>
where
    F: futures::Future<Output = Option<Box<[u8]>>>,
{
    if step_size == 0 || hash_value.is_empty() || max_iterations == 0 {
        return Err(InvalidInputValue);
    }

    let closest_intermediate = index_hint.unwrap_or(max_iterations - (max_iterations % step_size));
    let mut digest = EthDigest::default();

    for i in (0..=closest_intermediate).step_by(step_size) {
        // Check if we can get a hint for the current index
        let pos = closest_intermediate - i;
        if let Some(fetched_intermediate) = hints(pos).await {
            let mut intermediate = *GenericArray::from_slice(fetched_intermediate.as_ref());
            for iteration in 0..step_size {
                // Compute the hash of current intermediate
                digest.update(intermediate.as_ref());
                let hash = digest.finalize();

                // Is the computed hash the one we're looking for ?
                if hash.len() == hash_value.len() && hash.as_slice() == hash_value {
                    return Ok(Intermediate {
                        iteration: iteration + pos,
                        intermediate: intermediate.as_slice().into(),
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

    #[async_std::test]
    async fn test_iteration() {
        let seed = [0u8; 16];
        let final_hash = iterate_hash(&seed, 1000, 10).await;
        assert_eq!(final_hash.intermediates.len(), 100);

        let hint = &final_hash.intermediates[98]; // hint is at iteration num. 980
        assert_eq!(980, hint.iteration);

        let expected = hex!("a380d145d8612d33912494f1b36571c0b59b9bd459e6bb7d5ea05946be4c256b");
        assert_eq!(&expected, hint.intermediate.as_ref())
    }

    #[async_std::test]
    async fn test_recovery() {
        let hint_idx = 980;
        let target = hex!("16db2cf9913ccaf4976c825d1fd7b8f8e6510f479390c3272ed4e27fa015a537");

        let recovered = recover_iterated_hash(
            &target,
            &|i: usize| async move {
                if i == hint_idx {
                    Some(hex!("a380d145d8612d33912494f1b36571c0b59b9bd459e6bb7d5ea05946be4c256b").into())
                } else {
                    None
                }
            },
            1000,
            10,
            None,
        )
        .await
        .unwrap();

        assert_eq!(recovered.iteration, hint_idx + 7);
        assert_eq!(
            Hash::create(&[recovered.intermediate.as_ref()]).to_bytes().as_ref(),
            &target
        );
    }
}
