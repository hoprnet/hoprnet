//! This Rust crate contains implementation of the Sphinx packet format for the HOPR protocol.
//!
//!
//! The crate is structured into the following modules:
//! - `derivation`: contains simple key derivation functions for different purposes
//! - `ec_groups`: implementation of `SphinxSuite` trait, see the chapter below.
//! - `prg`: implementation of a pseudo-random generator function used in SPHINX packet header construction
//! - `prp`: implementation of the Lioness wide-block cipher using Chacha20 and Blake2b256
//! - `routing`: implements the SPHINX header
//! - `shared_keys`: derivation of shared keys for SPHINX header (see below)
//!
//! ## SPHINX shared keys derivation
//! The architecture of the SPHINX shared key derivation is done generically, so it can work with any elliptic curve group for which CDH problem is
//! hard. The generic Sphinx implementation only requires one to implement the `SphinxSuite` trait.
//! The trait requires to have the following building blocks:
//! - elliptic curve group (`GroupElement`) and corresponding the scalar type (`Scalar`)
//! - type representing public and private keypair and their conversion to `Scalar` and `GroupElement` (by the means of the corresponding `From` trait implementation)
//!
//! Currently, there are the following `SphinxSuite` implementations :
//! - `Secp256k1Suite`: deprecated, used in previous HOPR versions
//! - `Ed25519Suite`: simple implementation using Ed25519, used for testing
//! - `X25519Suite`: currently used, implemented using Curve25519 Montgomery curve for faster computation
//!
//! The implementation can be easily extended for different elliptic curves (or even arithmetic multiplicative groups).
//! In particular, as soon as there's way to represent `Ed448` PeerIDs, it would be easy to create e.g. `X448Suite`.

pub mod derivation;
pub mod ec_groups;
pub mod prg;
pub mod prp;
pub mod routing;
pub mod shared_keys;
