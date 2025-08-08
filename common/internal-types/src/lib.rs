//! This crate contains all types that are specific to the HOPR protocol.
//! As opposed to `hopr-primitive-types` which contains more generic types (not necessarily specific only to HOPR).

/// Contains all types related to node identities.
pub mod account;
/// Implements types for on-chain announcement of nodes.
pub mod announcement;
/// Implements types related to HOPR payment channels.
pub mod channels;
/// Enumerates all errors in this crate.
pub mod errors;
/// Types related to internal HOPR protocol logic.
pub mod protocol;
/// Implements types for tickets.
pub mod tickets;
#[doc(hidden)]
pub mod prelude {
    pub use super::{account::*, announcement::*, channels::*, errors::CoreTypesError, protocol::*, tickets::*};
}
