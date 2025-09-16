//! Crate for accessing database(s) of a HOPR node.
//!
//! Functionality defined here is meant to be used mostly by other higher-level crates.
//! The crate provides database operations across multiple SQLite databases for scalability
//! and supports importing logs database snapshots for fast synchronization.

pub mod accounts;
mod cache;
pub mod channels;
pub mod corrupted_channels;
pub mod db;
pub mod errors;
pub mod info;
pub mod logs;
pub mod peers;
pub mod protocol;
pub mod registry;
pub mod resolver;
mod ticket_manager;
pub mod tickets;

use std::path::PathBuf;

use async_trait::async_trait;
use futures::future::BoxFuture;
pub use hopr_db_api as api;
use hopr_db_api::{
    logs::HoprDbLogOperations, peers::HoprDbPeersOperations, protocol::HoprDbProtocolOperations,
    resolver::HoprDbResolverOperations, tickets::HoprDbTicketOperations,
};
use sea_orm::{ConnectionTrait, TransactionTrait};
pub use sea_orm::{DatabaseConnection, DatabaseTransaction};

use crate::{
    accounts::HoprDbAccountOperations,
    channels::HoprDbChannelOperations,
    corrupted_channels::HoprDbCorruptedChannelOperations,
    db::HoprDb,
    errors::{DbSqlError, Result},
    info::HoprDbInfoOperations,
    registry::HoprDbRegistryOperations,
};

/// Primary key used in tables that contain only a single row.
pub const SINGULAR_TABLE_FIXED_ID: i32 = 1;

/// Shorthand for the `chrono` based timestamp type used in the database.
pub type DbTimestamp = chrono::DateTime<chrono::Utc>;

/// Represents an already opened transaction.
/// This is a thin wrapper over [DatabaseTransaction].
/// The wrapping behavior is needed to allow transaction agnostic functionalities
/// of the DB traits.
#[derive(Debug)]
pub struct OpenTransaction(DatabaseTransaction, TargetDb);

impl OpenTransaction {
    /// Executes the given `callback` inside the transaction
    /// and commits the transaction if it succeeds or rollbacks otherwise.
    #[tracing::instrument(level = "trace", name = "Sql::perform_in_transaction", skip_all, err)]
    pub async fn perform<F, T, E>(self, callback: F) -> std::result::Result<T, E>
    where
        F: for<'c> FnOnce(&'c OpenTransaction) -> BoxFuture<'c, std::result::Result<T, E>> + Send,
        T: Send,
        E: std::error::Error + From<DbSqlError>,
    {
        let start = std::time::Instant::now();
        let res = callback(&self).await;

        if res.is_ok() {
            self.commit().await?;
        } else {
            self.rollback().await?;
        }

        tracing::trace!(
            elapsed_ms = start.elapsed().as_millis(),
            was_successful = res.is_ok(),
            "transaction completed",
        );

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
/// Useful for transaction nesting (see [`HoprDbGeneralModelOperations::nest_transaction`]).
pub type OptTx<'a> = Option<&'a OpenTransaction>;

/// When Sqlite is used as a backend, model needs to be split
/// into 4 different databases to avoid locking the database.
/// On Postgres backend, these should actually point to the same database.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TargetDb {
    #[default]
    /// Indexer database.
    Index,
    /// Acknowledged winning ticket database.
    Tickets,
    /// Network peers database
    Peers,
    /// RPC logs database
    Logs,
}

#[async_trait]
pub trait HoprDbGeneralModelOperations {
    /// Returns reference to the database connection.
    /// Can be used in case transaction is not needed, but
    /// users should aim to use [`HoprDbGeneralModelOperations::begin_transaction`]
    /// and [`HoprDbGeneralModelOperations::nest_transaction`] as much as possible.
    fn conn(&self, target_db: TargetDb) -> &DatabaseConnection;

    /// Creates a new transaction.
    async fn begin_transaction_in_db(&self, target: TargetDb) -> Result<OpenTransaction>;

    /// Import logs database from a snapshot directory.
    ///
    /// Replaces all data in the current logs database with data from a snapshot's
    /// `hopr_logs.db` file. This is used for fast synchronization during node startup.
    ///
    /// # Process
    ///
    /// 1. Attaches the source database from the snapshot directory
    /// 2. Clears existing data from all logs-related tables
    /// 3. Copies all data from the snapshot database
    /// 4. Detaches the source database
    ///
    /// All operations are performed within a single transaction for atomicity.
    ///
    /// # Arguments
    ///
    /// * `src_dir` - Directory containing the extracted snapshot with `hopr_logs.db`
    ///
    /// # Returns
    ///
    /// `Ok(())` on successful import, or [`DbSqlError::Construction`] if the source
    /// database doesn't exist or the import operation fails.
    ///
    /// # Errors
    ///
    /// - Returns error if `hopr_logs.db` is not found in the source directory
    /// - Returns error if SQLite ATTACH, data transfer, or DETACH operations fail
    /// - All database errors are wrapped in [`DbSqlError::Construction`]
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::path::PathBuf;
    /// # use hopr_db_sql::HoprDbGeneralModelOperations;
    /// # async fn example(db: impl HoprDbGeneralModelOperations) -> Result<(), Box<dyn std::error::Error>> {
    /// let snapshot_dir = PathBuf::from("/tmp/snapshot_extracted");
    /// db.import_logs_db(snapshot_dir).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn import_logs_db(self, src_dir: PathBuf) -> Result<()>;

    /// Same as [`HoprDbGeneralModelOperations::begin_transaction_in_db`] with default [TargetDb].
    async fn begin_transaction(&self) -> Result<OpenTransaction> {
        self.begin_transaction_in_db(Default::default()).await
    }

    /// Creates a nested transaction inside the given transaction.
    ///
    /// If `None` is given, behaves exactly as [`HoprDbGeneralModelOperations::begin_transaction`].
    ///
    /// This method is useful for creating APIs that should be agnostic whether they are being
    /// run from an existing transaction or without it (via [OptTx]).
    ///
    /// If `tx` is `Some`, the `target_db` must match with the one in `tx`. In other words,
    /// nesting across different databases is forbidden and the method will panic.
    async fn nest_transaction_in_db(&self, tx: OptTx<'_>, target_db: TargetDb) -> Result<OpenTransaction> {
        if let Some(t) = tx {
            assert_eq!(t.1, target_db, "attempt to create nest into tx from a different db");
            Ok(OpenTransaction(t.as_ref().begin().await?, target_db))
        } else {
            self.begin_transaction_in_db(target_db).await
        }
    }

    /// Same as [`HoprDbGeneralModelOperations::nest_transaction_in_db`] with default [TargetDb].
    async fn nest_transaction(&self, tx: OptTx<'_>) -> Result<OpenTransaction> {
        self.nest_transaction_in_db(tx, Default::default()).await
    }
}

#[async_trait]
impl HoprDbGeneralModelOperations for HoprDb {
    /// Retrieves raw database connection to the given [DB](TargetDb).
    fn conn(&self, target_db: TargetDb) -> &DatabaseConnection {
        match target_db {
            TargetDb::Index => self.index_db.read_only(), // TODO: no write access needed here, deserves better
            // wrapping
            TargetDb::Tickets => &self.tickets_db,
            TargetDb::Peers => &self.peers_db,
            TargetDb::Logs => &self.logs_db,
        }
    }

    /// Starts a new transaction in the given [DB](TargetDb).
    async fn begin_transaction_in_db(&self, target_db: TargetDb) -> Result<OpenTransaction> {
        match target_db {
            TargetDb::Index => Ok(OpenTransaction(
                self.index_db.read_write().begin_with_config(None, None).await?, /* TODO: cannot estimate intent,
                                                                                  * must be readwrite */
                target_db,
            )),
            // TODO: when adding Postgres support, redirect `Tickets` and `Peers` into `self.db`
            TargetDb::Tickets => Ok(OpenTransaction(
                self.tickets_db.begin_with_config(None, None).await?,
                target_db,
            )),
            TargetDb::Peers => Ok(OpenTransaction(
                self.peers_db.begin_with_config(None, None).await?,
                target_db,
            )),
            TargetDb::Logs => Ok(OpenTransaction(
                self.logs_db.begin_with_config(None, None).await?,
                target_db,
            )),
        }
    }

    async fn import_logs_db(self, src_dir: PathBuf) -> Result<()> {
        let src_db_path = src_dir.join("hopr_logs.db");
        if !src_db_path.exists() {
            return Err(DbSqlError::Construction(format!(
                "Source logs database file does not exist: {}",
                src_db_path.display()
            )));
        }

        let sql = format!(
            r#"
            ATTACH DATABASE '{}' AS source_logs;
            BEGIN TRANSACTION;
            DELETE FROM log;
            DELETE FROM log_status;
            DELETE FROM log_topic_info;
            INSERT INTO log_topic_info SELECT * FROM source_logs.log_topic_info;
            INSERT INTO log_status SELECT * FROM source_logs.log_status;
            INSERT INTO log SELECT * FROM source_logs.log;
            COMMIT;
            DETACH DATABASE source_logs;
        "#,
            src_db_path.to_string_lossy().replace("'", "''")
        );

        let logs_conn = self.conn(TargetDb::Logs);

        logs_conn
            .execute_unprepared(sql.as_str())
            .await
            .map_err(|e| DbSqlError::Construction(format!("Failed to import logs data: {e}")))?;

        Ok(())
    }
}

/// Convenience trait that contain all HOPR DB operations crates.
pub trait HoprDbAllOperations:
    HoprDbGeneralModelOperations
    + HoprDbAccountOperations
    + HoprDbChannelOperations
    + HoprDbCorruptedChannelOperations
    + HoprDbInfoOperations
    + HoprDbLogOperations
    + HoprDbPeersOperations
    + HoprDbProtocolOperations
    + HoprDbRegistryOperations
    + HoprDbResolverOperations
    + HoprDbTicketOperations
{
}

#[doc(hidden)]
pub mod prelude {
    pub use hopr_db_api::{logs::*, peers::*, protocol::*, resolver::*, tickets::*};

    pub use super::*;
    pub use crate::{accounts::*, channels::*, corrupted_channels::*, db::*, errors::*, info::*, registry::*};
}
