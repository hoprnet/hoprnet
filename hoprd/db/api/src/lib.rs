//! Crate for accessing database(s) of a HOPR node.
//! Functionality defined here is meant to be used mostly by other higher-level crates.

pub mod aliases;
pub mod db;
pub mod errors;

pub use sea_orm::DatabaseConnection;
pub use sea_orm::DatabaseTransaction;

use crate::aliases::HoprdDbAliasesOperations;

/// Convenience trait that contain all HOPR DB operations crates.
pub trait HoprdDbAllOperations: HoprdDbAliasesOperations {}

#[doc(hidden)]
pub mod prelude {
    pub use super::*;
    pub use crate::aliases::*;
}
