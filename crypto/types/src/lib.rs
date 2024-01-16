//! This Rust crate contains implementation common cryptographic types.
//!
//!
//! The crate is structured into the following modules:
//! - `errors`: contains error enum implementation used across other `hopr-crypto-...` crates
//! - `keypairs`: implements `ChainKeypair` and `OffchainKeypair`, important representations of chain key and packet key
//! - `primitives`: implements low-level cryptographic primitives, such as `SimpleStreamCipher`, `Digest` and `SimpleMac`
//! - `types`: implements basic cryptography related types based on `primitives`, such as `Hash`, `PublicKey` and `Signature`
//! - `utils`: contains small utility functions used in other `hopr-crypto-...` crates
//! - `vrf`: contains implementation of Verifiable Random Function used in tickets

pub mod errors;
pub mod keypairs;
pub mod primitives;
pub mod types;
pub mod utils;
pub mod vrf;
pub mod prelude {
    pub use super::errors::CryptoError;
    pub use super::keypairs::*;
    pub use super::primitives::*;
    pub use super::types::*;
    pub use super::utils::*;
    pub use super::vrf::*;
}
