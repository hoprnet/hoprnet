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
//! and [GroupElement](shared_keys::GroupElement) (by the means of the corresponding `From` trait implementation)
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
/// Implementation of a pseudo-random generator function used in SPHINX packet header construction
mod prg;
/// Implementation of the Lioness wide-block cipher using Chacha20 and Blake2b256
pub mod prp;
/// Implementation of the SPHINX header format
pub mod routing;
/// Derivation of shared keys for SPHINX header
pub mod shared_keys;
