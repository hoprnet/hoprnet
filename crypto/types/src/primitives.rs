use blake2::Blake2s256;
use digest::{FixedOutputReset, Output, OutputSizeUser, Update};
use generic_array::GenericArray;
use sha3::Keccak256;
use typenum::Unsigned;

use crate::utils::SecretValue;

pub use chacha20::ChaCha20;
pub use poly1305::Poly1305;

/// Represents a 256-bit secret key of fixed length.
/// The value is auto-zeroized on drop.
pub type SecretKey = SecretValue<typenum::U32>;

/// Represents a 128-bit secret key of fixed length.
/// The value is auto-zeroized on drop.
pub type SecretKey16 = SecretValue<typenum::U16>;

/// Generalization of digest-like operation (MAC, Digest,...)
/// Defines the `update` and `finalize` operations to produce digest value of arbitrary data.
pub trait DigestLike<T>
where
    T: Update + FixedOutputReset + OutputSizeUser,
{
    /// Length of the digest in bytes
    const SIZE: usize = T::OutputSize::USIZE;

    /// Access to the internal state of the digest-like operation.
    fn internal_state(&mut self) -> &mut T;

    /// Update the internal state of the digest-like using the given input data.
    fn update(&mut self, data: &[u8]) {
        self.internal_state().update(data);
    }

    /// Retrieve the final digest value into a prepared buffer and reset this instance so it could be reused for
    /// a new computation.
    fn finalize_into(&mut self, out: &mut [u8]) {
        assert_eq!(Self::SIZE, out.len(), "invalid output size");
        let output = Output::<T>::from_mut_slice(out);
        self.internal_state().finalize_into_reset(output);
    }

    /// Retrieve the final digest value and reset this instance so it could be reused for
    /// a new computation.
    fn finalize(&mut self) -> GenericArray<u8, T::OutputSize> {
        let mut output = Output::<T>::default();
        self.finalize_into(&mut output);
        output
    }
}

/// Simple digest computation wrapper.
/// Use `new`, `update` and `finalize` triplet to produce hash of arbitrary data.
/// Currently, this instance is using Blake2s256.
#[derive(Default, Clone)]
pub struct SimpleDigest(Blake2s256);

impl DigestLike<Blake2s256> for SimpleDigest {
    fn internal_state(&mut self) -> &mut Blake2s256 {
        &mut self.0
    }
}

/// Computation wrapper for a digest that's compatible with Ethereum digests.
/// Use `new`, `update` and `finalize` triplet to produce hash of arbitrary data.
/// Currently, this instance is using Keccak256.
#[derive(Default, Clone)]
pub struct EthDigest(Keccak256);

impl DigestLike<Keccak256> for EthDigest {
    fn internal_state(&mut self) -> &mut Keccak256 {
        &mut self.0
    }
}
