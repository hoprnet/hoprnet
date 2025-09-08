//! This crate defines the Application layer protocol as in [RFC-0014](https://github.com/hoprnet/rfc/blob/main/rfcs/RFC-0003-application-protocol/0014-application-protocol.md).
//!
//! This typically runs on top of the HOPR packet protocol (as described in
//! [RFC-0003](https://github.com/hoprnet/rfc/blob/main/rfcs/RFC-0003-hopr-packet-protocol/0003-hopr-packet-protocol.md))

/// Errors that can occur during the application layer protocol.
pub mod errors;
/// The Application layer protocol version 1.
pub mod v1;

pub mod prelude {
    pub use crate::v1::{
        ApplicationData, ApplicationDataIn, ApplicationDataOut, IncomingPacketInfo, OutgoingPacketInfo, ReservedTag,
        Tag,
    };
}
