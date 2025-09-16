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
pub mod tx;

use crate::{
    peers::HoprDbPeersOperations, protocol::HoprDbProtocolOperations,
    resolver::HoprDbResolverOperations, tickets::HoprDbTicketOperations,
};

/// Convenience trait that contains all HOPR DB operation interfaces.
pub trait HoprNodeDbAllAbstractedOperations:
    HoprDbTicketOperations
    + HoprDbPeersOperations
    + HoprDbProtocolOperations
{
}

#[doc(hidden)]
pub mod prelude {
    pub use super::*;
    pub use crate::{errors::*, info::*, logs::*, peers::*, protocol::*, resolver::*, tickets::*, tx::*};
}
