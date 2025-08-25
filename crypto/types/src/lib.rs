//! This Rust crate contains implementation of common cryptographic types.

/// Contains error enum implementation used across other `hopr-crypto-...` crates
pub mod errors;
/// Implements [ChainKeypair](keypairs::ChainKeypair) and [OffchainKeypair](keypairs::OffchainKeypair),
/// the important representations of chain key and packet key.
pub mod keypairs;
pub mod lioness;
/// Re-exports of low-level cryptographic primitives.
pub mod primitives;
/// Enables randomized encryption (sealing)
/// and decryption of data using [`OffchainKeypair`](keypairs::OffchainKeypair).
pub mod seal;
/// Implements basic cryptography-related types based on [primitives], such as [Hash](types::Hash),
/// [PublicKey](types::PublicKey) and [Signature](types::Signature).
pub mod types;
/// Contains small utility functions used in the other `hopr-crypto-...` crates
pub mod utils;
/// Contains implementation of Verifiable Random Function used in tickets
pub mod vrf;

/// Re-exports from the generic cryptographic traits.
pub mod crypto_traits {
    pub use cipher::{
        BlockSizeUser, Iv, IvSizeUser, Key, KeyInit, KeyIvInit, KeySizeUser, StreamCipher, StreamCipherSeek,
    };
    use cipher::Block;
    pub use digest::{Digest, FixedOutput, FixedOutputReset, Output, OutputSizeUser, Update};
    pub use hopr_crypto_random::Randomizable;
    pub use poly1305::universal_hash::UniversalHash;

    /// Pseudo-random permutation (PRP)
    pub trait PRP: BlockSizeUser {
        /// Forward permutation
        fn forward(&self, data: &mut Block<Self>);
        /// Inverse permutation
        fn inverse(&self, data: &mut Block<Self>);
    }
}

#[doc(hidden)]
pub mod prelude {
    pub use libp2p_identity::PeerId;

    pub use super::{
        crypto_traits, errors::CryptoError, keypairs::*, primitives::*, seal::*, types::*, utils::*, vrf::*,
    };
}
