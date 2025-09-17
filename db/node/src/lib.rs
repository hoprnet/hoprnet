mod node_db;

mod peers;

mod protocol;

mod tickets;

mod cache;
mod errors;
mod ticket_manager;

pub use node_db::{HoprNodeDb, HoprNodeDbConfig};

/// Primary key used in tables that contain only a single row.
pub const SINGULAR_TABLE_FIXED_ID: i32 = 1;

/// Shorthand for the `chrono` based timestamp type used in the database.
pub type DbTimestamp = chrono::DateTime<chrono::Utc>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TargetNodeDb {
    #[default]
    /// Peers database.
    Peers,
    /// Tickets database.
    Tickets,
}
