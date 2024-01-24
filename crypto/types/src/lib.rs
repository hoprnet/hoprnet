//! This Rust crate contains implementation common cryptographic types.
//!

/// Contains error enum implementation used across other `hopr-crypto-...` crates
pub mod errors;
/// Implements [ChainKeypair](keypairs::ChainKeypair) and [OffchainKeypair](keypairs::OffchainKeypairs), the important representations of chain key and packet key.
pub mod keypairs;
/// Implements low-level cryptographic primitives, such as [SimpleStreamCipher](primitives::SimpleStreamCipher), [Digest](primitives::Digest) and [SimpleMac](primitives::SimpleMac).
pub mod primitives;
/// Implements basic cryptography related types based on [primitives], such as [Hash](types::Hash), [PublicKey](types::PublicKey) and [Signature](types::Signature).
pub mod types;
/// Contains small utility functions used in other `hopr-crypto-...` crates
pub mod utils;
/// Contains implementation of Verifiable Random Function used in tickets
pub mod vrf;
#[doc(hidden)]
pub mod prelude {
    pub use super::errors::CryptoError;
    pub use super::keypairs::*;
    pub use super::primitives::*;
    pub use super::types::*;
    pub use super::utils::*;
    pub use super::vrf::*;
}
