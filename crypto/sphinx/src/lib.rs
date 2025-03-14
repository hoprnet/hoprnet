//! This Rust crate contains implementation of the Sphinx packet format for the HOPR protocol.
//!
//! ## SPHINX shared keys derivation
//! The architecture of the SPHINX shared key derivation is done generically, so it can work with any
//! elliptic curve group for which CDH problem is hard. The generic Sphinx implementation only
//! requires one to implement the `SphinxSuite` trait.
//!
//! The trait requires to have the following building blocks:
//! - elliptic curve group ([GroupElement](shared_keys::GroupElement)) and corresponding the scalar type ([Scalar](shared_keys::Scalar))
//! - type representing public and private keypair and their conversion to [Scalar](shared_keys::Scalar)
//!   and [GroupElement](shared_keys::GroupElement) (by the means of the corresponding `From` trait implementation)
//!
//! Currently, there are the following [SphinxSuite](crate::shared_keys::SphinxSuite) implementations :
//! - `Secp256k1Suite`: deprecated, used in previous HOPR versions
//! - `Ed25519Suite`: simple implementation using Ed25519, used for testing
//! - [X25519Suite](crate::ec_groups::X25519Suite) currently used, implemented using Curve25519 Montgomery curve for faster computation
//!
//! The implementation can be easily extended for different elliptic curves (or even arithmetic multiplicative groups).
//! In particular, as soon as there's way to represent `Ed448` PeerIDs, it would be easy to create e.g. an `X448Suite`.

/// Contains simple key derivation functions for different purposes
pub mod derivation;
/// Implementations of `SphinxSuite` trait for different elliptic curve groups
pub mod ec_groups;
/// Contains various errors returned from this crate.
pub mod errors;
/// Contains the main implementation of a SPHINX packet.
pub mod packet;
/// Implementation of the SPHINX header format
pub mod routing;
/// Derivation of shared keys for SPHINX header
pub mod shared_keys;
/// Contains Return Path and SURB-related types
pub mod surb;

pub mod prelude {
    pub use crate::ec_groups::*;
    pub use crate::packet::{ForwardedMetaPacket, MetaPacket, MetaPacketRouting};
    pub use crate::routing::SphinxHeaderSpec;
    pub use crate::shared_keys::{SharedKeys, SharedSecret, SphinxSuite};
    pub use crate::surb::*;
}

#[cfg(test)]
pub(crate) mod tests {
    use std::marker::PhantomData;
    use std::num::{NonZero, NonZeroUsize};

    use hopr_crypto_types::prelude::*;
    use hopr_primitive_types::errors::GeneralError;
    use hopr_primitive_types::prelude::*;

    use crate::routing::SphinxHeaderSpec;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(crate) struct WrappedBytes<const N: usize>(pub [u8; N]);

    impl<const N: usize> Default for WrappedBytes<N> {
        fn default() -> Self {
            Self([0u8; N])
        }
    }

    impl<'a, const N: usize> TryFrom<&'a [u8]> for WrappedBytes<N> {
        type Error = GeneralError;

        fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
            value
                .try_into()
                .map(Self)
                .map_err(|_| GeneralError::ParseError("WrappedBytes".into()))
        }
    }

    impl<const N: usize> AsRef<[u8]> for WrappedBytes<N> {
        fn as_ref(&self) -> &[u8] {
            &self.0
        }
    }

    impl<const N: usize> BytesRepresentable for WrappedBytes<N> {
        const SIZE: usize = N;
    }

    pub(crate) struct TestSpec<K, const HOPS: usize, const RELAYER_DATA: usize>(PhantomData<K>);
    impl<K, const HOPS: usize, const RELAYER_DATA: usize> SphinxHeaderSpec for TestSpec<K, HOPS, RELAYER_DATA>
    where
        K: AsRef<[u8]> + for<'a> TryFrom<&'a [u8], Error = GeneralError> + BytesRepresentable + Clone,
    {
        const MAX_HOPS: NonZeroUsize = NonZero::new(HOPS).unwrap();
        type KeyId = K;
        type Pseudonym = SimplePseudonym;
        type RelayerData = WrappedBytes<RELAYER_DATA>;
        type SurbReceiverData = WrappedBytes<53>;
        type PRG = ChaCha20;
        type UH = Poly1305;
    }
}
