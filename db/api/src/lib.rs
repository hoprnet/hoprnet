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

pub use sea_orm::DatabaseConnection;
pub use sea_orm::DatabaseTransaction;

use async_trait::async_trait;
use futures::future::BoxFuture;
use sea_orm::{ConnectionTrait, EntityTrait, TransactionTrait};
use hopr_db_entity::chain_info::Model;

use crate::db::HoprDb;
use crate::errors::Result;

pub type DbTimestamp = chrono::DateTime<chrono::Utc>;

#[async_trait]
pub trait HoprDbGeneralModelOperations {
    fn conn(&self) -> &DatabaseConnection;

    async fn begin_transaction(&self) -> Result<DatabaseTransaction>;

    async fn transaction<F, T, E>(&self, callback: F) -> Result<T>
    where
        F: for<'a> FnOnce(&'a DatabaseTransaction) -> BoxFuture<'a, std::result::Result<T, E>> + Send,
        T: Send,
        E: std::error::Error + Send;
}

#[async_trait]
impl HoprDbGeneralModelOperations for HoprDb {
    fn conn(&self) -> &DatabaseConnection {
        &self.db
    }

    async fn begin_transaction(&self) -> Result<DatabaseTransaction> {
        Ok(self.db.begin_with_config(None, None).await?)
    }

    async fn transaction<F, T, E>(&self, callback: F) -> Result<T>
    where
        F: for<'a> FnOnce(&'a DatabaseTransaction) -> BoxFuture<'a, std::result::Result<T, E>> + Send,
        T: Send,
        E: std::error::Error + Send,
    {
        Ok(self.db.transaction_with_config(callback, None, None).await?)
    }
}
