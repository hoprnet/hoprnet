//! This crate contains all types that are specific to the HOPR protocol.
//! As opposed to `hopr-primitive-types` which contains more generic types (not necessarily specific only to HOPR).

pub mod account;
pub mod acknowledgement;
pub mod announcement;
pub mod channels;
pub mod errors;
pub mod protocol;

pub mod prelude {
    pub use super::account::*;
    pub use super::acknowledgement::*;
    pub use super::announcement::*;
    pub use super::channels::*;
    pub use super::errors::CoreTypesError;
    pub use super::protocol::*;
}