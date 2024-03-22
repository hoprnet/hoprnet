pub mod channels;
pub mod db;
pub mod errors;

#[cfg(feature = "peers")]
pub mod peers;

#[cfg(feature = "ticket")]
pub mod tickets;

#[cfg(feature = "accounts")]
pub mod accounts;

#[cfg(feature = "registry")]
pub mod registry;

#[cfg(feature = "resolver")]
pub mod resolver;

#[cfg(feature = "info")]
pub mod info;
pub mod ticket_manager;

pub const SINGULAR_TABLE_FIXED_ID: i32 = 1;

pub use sea_orm::DatabaseConnection;
pub use sea_orm::DatabaseTransaction;

use crate::accounts::HoprDbAccountOperations;
use crate::channels::HoprDbChannelOperations;
use async_trait::async_trait;
use futures::future::BoxFuture;
use sea_orm::TransactionTrait;

use crate::db::HoprDb;
use crate::errors::{DbError, Result};
use crate::info::HoprDbInfoOperations;
use crate::peers::HoprDbPeersOperations;
use crate::registry::HoprDbRegistryOperations;
use crate::resolver::HoprDbResolverOperations;
use crate::tickets::HoprDbTicketOperations;

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
/// Useful for transaction nesting (see [`HoprDbGeneralModelOperations::nest_transaction`]).
pub type OptTx<'a> = Option<&'a OpenTransaction>;

/// When Sqlite is used as a backend, model needs to be split
/// into 3 different databases to avoid locking the database.
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
    fn conn(&self, target_db: TargetDb) -> &DatabaseConnection {
        match target_db {
            TargetDb::Index => &self.db,
            TargetDb::Tickets => &self.tickets_db,
            TargetDb::Peers => &self.peers_db,
        }
    }

    async fn begin_transaction_in_db(&self, target_db: TargetDb) -> Result<OpenTransaction> {
        match target_db {
            TargetDb::Index => Ok(OpenTransaction(self.db.begin_with_config(None, None).await?, target_db)),
            // TODO: when adding Postgres support, redirect `Tickets` and `Peers` into `self.db`
            TargetDb::Tickets => Ok(OpenTransaction(
                self.tickets_db.begin_with_config(None, None).await?,
                target_db,
            )),
            TargetDb::Peers => Ok(OpenTransaction(
                self.peers_db.begin_with_config(None, None).await?,
                target_db,
            )),
        }
    }
}

pub trait HoprDbAllOperations:
    HoprDbGeneralModelOperations
    + HoprDbAccountOperations
    + HoprDbChannelOperations
    + HoprDbInfoOperations
    + HoprDbRegistryOperations
    + HoprDbTicketOperations
    + HoprDbPeersOperations
    + HoprDbResolverOperations
{
}
