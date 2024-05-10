//! # core-packet
//!
//! This crate contains the main packet processing functionality for the HOPR protocol.
//! It implements the following important protocol building blocks:
//!
//! - SPHINX packet format (module [packet])
//! - Proof of Relay (module [por])
//!
//! Finally, it also implements a utility function which is used to validate tickets (module `validation`).
//! The ticket validation functionality is dependent on `chain-db`.
//!
//! The currently used implementation is selected using the `CurrentSphinxSuite` type in the `packet` module.
//!
//! The implementation can be easily extended for different elliptic curves (or even arithmetic multiplicative groups).
//! In particular, as soon as there's way to represent `Ed448` PeerIDs, it would be easy to create e.g. `X448Suite`.
//!

/// Implements the overlay packet intermediary object.
pub mod chain;
/// Enumerates all errors in this crate.
pub mod errors;
/// Implements SPHINX packet format.
pub mod packet;
/// Implements the Proof of Relay.
pub mod por;
/// Implements ticket validation logic.
pub mod validation;
