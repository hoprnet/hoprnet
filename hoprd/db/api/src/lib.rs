//! Crate for accessing database(s) of a HOPR node.
//! Functionality defined here is meant to be used mostly by other higher-level crates.

pub mod db;
pub mod errors;
pub mod metadata;

pub use sea_orm::DatabaseConnection;
pub use sea_orm::DatabaseTransaction;

use crate::metadata::HoprdDbMetadataOperations;
use async_trait::async_trait;
use futures::future::BoxFuture;
use sea_orm::TransactionTrait;

use crate::db::HoprdDb;
use crate::errors::{DbError, Result};

/// Primary key used in tables that contain only a single row.
pub const SINGULAR_TABLE_FIXED_ID: i32 = 1;

/// Shorthand for the `chrono` based timestamp type used in the database.
pub type DbTimestamp = chrono::DateTime<chrono::Utc>;

/// Represents an already opened transaction.
/// This is a thin wrapper over [DatabaseTransaction].
/// The wrapping behavior is needed to allow transaction agnostic functionalities
/// of the DB traits.
#[derive(Debug)]
pub struct OpenTransaction(DatabaseTransaction);

impl OpenTransaction {
    /// Executes the given `callback` inside the transaction
    /// and commits the transaction if it succeeds or rollbacks otherwise.
    pub async fn perform<F, T, E>(self, callback: F) -> std::result::Result<T, E>
    where
        F: for<'c> FnOnce(&'c OpenTransaction) -> BoxFuture<'c, std::result::Result<T, E>> + Send,
        T: Send,
        E: std::error::Error + From<DbError>,
    {
        let res = callback(&self).await;

        if res.is_ok() {
            self.commit().await?;
        } else {
            self.rollback().await?;
        }
        res
    }

    /// Commits the transaction.
    pub async fn commit(self) -> Result<()> {
        Ok(self.0.commit().await?)
    }

    /// Rollbacks the transaction.
    pub async fn rollback(self) -> Result<()> {
        Ok(self.0.rollback().await?)
    }
}

impl AsRef<DatabaseTransaction> for OpenTransaction {
    fn as_ref(&self) -> &DatabaseTransaction {
        &self.0
    }
}

impl From<OpenTransaction> for DatabaseTransaction {
    fn from(value: OpenTransaction) -> Self {
        value.0
    }
}

/// Shorthand for optional transaction.
/// Useful for transaction nesting (see [`HoprdDbGeneralModelOperations::nest_transaction`]).
pub type OptTx<'a> = Option<&'a OpenTransaction>;

#[async_trait]
pub trait HoprdDbGeneralModelOperations {
    /// Returns reference to the database connection.
    /// Can be used in case transaction is not needed, but
    /// users should aim to use [`HoprdDbGeneralModelOperations::begin_transaction`]
    /// and [`HoprdDbGeneralModelOperations::nest_transaction`] as much as possible.
    fn conn(&self) -> &DatabaseConnection;

    /// Creates a new transaction.
    async fn begin_transaction_in_db(&self) -> Result<OpenTransaction>;

    /// Same as [`HoprdDbGeneralModelOperations::begin_transaction_in_db`] with default [TargetDb].
    async fn begin_transaction(&self) -> Result<OpenTransaction> {
        self.begin_transaction_in_db().await
    }

    /// Creates a nested transaction inside the given transaction.
    ///
    /// If `None` is given, behaves exactly as [`HoprdDbGeneralModelOperations::begin_transaction`].
    ///
    /// This method is useful for creating APIs that should be agnostic whether they are being
    /// run from an existing transaction or without it (via [OptTx]).
    ///
    /// If `tx` is `Some`, the `target_db` must match with the one in `tx`. In other words,
    /// nesting across different databases is forbidden and the method will panic.
    async fn nest_transaction_in_db(&self, tx: OptTx<'_>) -> Result<OpenTransaction> {
        if let Some(t) = tx {
            Ok(OpenTransaction(t.as_ref().begin().await?))
        } else {
            self.begin_transaction_in_db().await
        }
    }

    /// Same as [`HoprdDbGeneralModelOperations::nest_transaction_in_db`] with default [TargetDb].
    async fn nest_transaction(&self, tx: OptTx<'_>) -> Result<OpenTransaction> {
        self.nest_transaction_in_db(tx).await
    }
}

#[async_trait]
impl HoprdDbGeneralModelOperations for HoprdDb {
    /// Retrieves raw database connection to the given [DB](TargetDb).
    fn conn(&self) -> &DatabaseConnection {
        &self.metadata
    }

    /// Starts a new transaction in the given [DB](TargetDb).
    async fn begin_transaction_in_db(&self) -> Result<OpenTransaction> {
        Ok(OpenTransaction(self.metadata.begin_with_config(None, None).await?))
    }
}

/// Convenience trait that contain all HOPR DB operations crates.
pub trait HoprdDbAllOperations: HoprdDbGeneralModelOperations + HoprdDbMetadataOperations {}

#[doc(hidden)]
pub mod prelude {
    pub use super::*;
    pub use crate::metadata::*;
}
