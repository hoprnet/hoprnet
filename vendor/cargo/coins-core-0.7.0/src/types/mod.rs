//! Holds generic types useful describing transactions. The `Transaction` trait conceptualizes
//! UTXO transactions as a list of inputs and outputs, and allows implementations to define what
//! those are precisely.
//!
//! The `Ser` trait describes a simple `Read'/'Write`-based interface for binary serialization. We
//! provide implementations for several primitives (i.e `Vec<T: Ser>` and `u8`, `u32`, and 'u64`).
//!
//! A Bitcoin implementation of all types is provided in the `bitcoin` crate.

// /// Contains a set of traits useful for representing and serializing transactions.
// pub mod primitives;

/// Contains the abstract `Transaction` trait.
pub mod tx;

// pub use primitives::*;
pub use tx::*;
