mod node_db;

mod peers;

mod protocol;

mod tickets;

mod cache;
mod ticket_manager;
mod safe_info;

use hopr_db_api::prelude::{DbError, OpenTransaction};
pub use node_db::{HoprNodeDb, HoprNodeDbConfig};
use sea_orm::TransactionTrait;

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

pub struct NodeDbTx(pub(crate) sea_orm::DatabaseTransaction);

impl NodeDbTx {
    pub async fn new(db: &sea_orm::DatabaseConnection) -> Result<Self, DbError> {
        db.begin().await.map(Self).map_err(|e| DbError::SqlError(e.into()))
    }
}

impl AsRef<sea_orm::DatabaseTransaction> for NodeDbTx {
    fn as_ref(&self) -> &sea_orm::DatabaseTransaction {
        &self.0
    }
}

impl OpenTransaction for NodeDbTx {
    async fn commit(self) -> Result<(), DbError> {
        self.0.commit().await.map_err(|e| DbError::SqlError(e.into()))
    }

    async fn rollback(self) -> Result<(), DbError> {
        self.0.rollback().await.map_err(|e| DbError::SqlError(e.into()))
    }
}
