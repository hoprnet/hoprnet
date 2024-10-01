use async_trait::async_trait;
use futures::stream::BoxStream;

use hopr_primitive_types::prelude::SerializableLog;

use crate::errors::Result;

#[async_trait]
pub trait HoprDbLogOperations {
    /// Retrieve acknowledged winning tickets according to the given `selector`.
    ///
    /// The optional transaction `tx` must be in the database.
    async fn store_log<'a>(&'a self, log: SerializableLog) -> Result<()>;

    async fn store_logs(&self, logs: Vec<SerializableLog>) -> Result<Vec<Result<()>>>;

    async fn get_log(&self, block_number: u64, tx_index: u64, log_index: u64) -> Result<SerializableLog>;

    async fn get_logs<'a>(
        &'a self,
        block_number: Option<u64>,
        block_offset: Option<u64>,
    ) -> Result<BoxStream<'a, SerializableLog>>;

    async fn get_logs_count(&self, block_number: Option<u64>, block_offset: Option<u64>) -> Result<u64>;

    async fn get_logs_block_numbers<'a>(
        &'a self,
        block_number: Option<u64>,
        block_offset: Option<u64>,
    ) -> Result<BoxStream<'a, u64>>;

    async fn set_log_processed(&self, log: SerializableLog) -> Result<()>;

    async fn set_logs_processed(&self, block_number: Option<u64>, block_offset: Option<u64>) -> Result<()>;

    async fn set_logs_unprocessed(&self, block_number: Option<u64>, block_offset: Option<u64>) -> Result<()>;

    async fn get_last_checksummed_log(&self) -> Result<Option<SerializableLog>>;

    async fn update_logs_checksums(&self) -> Result<()>;
}
