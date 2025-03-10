//! This Rust crate contains implementation of common cryptographic types.
//!

/// Contains error enum implementation used across other `hopr-crypto-...` crates
pub mod errors;
/// Implements [ChainKeypair](keypairs::ChainKeypair) and [OffchainKeypair](keypairs::OffchainKeypair),
/// the important representations of chain key and packet key.
pub mod keypairs;
/// Implements low-level cryptographic primitives, such as [SimpleStreamCipher](primitives::SimpleStreamCipher),
/// [SimpleDigest](primitives::SimpleDigest) and [SimpleMac](primitives::SimpleMac).
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
    pub use poly1305::universal_hash::UniversalHash;
}

#[doc(hidden)]
pub mod prelude {
    pub use super::crypto_traits;
    pub use super::errors::CryptoError;
    pub use super::keypairs::*;
    pub use super::primitives::*;
    pub use super::seal::*;
    pub use super::types::*;
    pub use super::utils::*;
    pub use super::vrf::*;

    //pub use libp2p_identity::PeerId;
}
