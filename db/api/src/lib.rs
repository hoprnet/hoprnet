//! Crate for abstracting the required DB behavior of a HOPR node.
//!
//! Functionality defined here is meant to be used mostly by other higher-level crates.

pub mod errors;
pub mod info;
pub mod logs;
pub mod peers;
pub mod protocol;
pub mod resolver;
pub mod tickets;

use crate::{
    logs::HoprDbLogOperations, peers::HoprDbPeersOperations, protocol::HoprDbProtocolOperations,
    resolver::HoprDbResolverOperations, tickets::HoprDbTicketOperations,
};

/// Convenience trait that contains all HOPR DB operation interfaces.
pub trait HoprDbAllAbstractedOperations:
    HoprDbTicketOperations
    + HoprDbPeersOperations
    + HoprDbResolverOperations
    + HoprDbProtocolOperations
    + HoprDbLogOperations
{
}

#[doc(hidden)]
pub mod prelude {
    pub use super::*;
    pub use crate::errors::*;
    pub use crate::info::*;
    pub use crate::logs::*;
    pub use crate::peers::*;
    pub use crate::protocol::*;
    pub use crate::resolver::*;
    pub use crate::tickets::*;
}
