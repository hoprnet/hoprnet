use std::future::Future;
use crate::errors::DbError;

#[async_trait::async_trait]
pub trait OpenTransaction {
    async fn perform<F, Fut, T, E>(self, callback: F) -> Result<T, E>
    where
        F: for<'c> FnOnce(&'c Self) -> Fut + Send,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send,
        E: std::error::Error + From<DbError> + Send,
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
    async fn commit(self) -> Result<(), DbError>;

    /// Rollbacks the transaction.
    async fn rollback(self) -> Result<(), DbError>;

}