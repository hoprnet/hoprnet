use std::path::PathBuf;
use async_trait::async_trait;
use hopr_crypto_types::prelude::Hash;
use hopr_primitive_types::prelude::{Address, SerializableLog};

use crate::errors::Result;

#[async_trait]
pub trait HoprDbLogOperations {
    /// Import logs-database from a snapshot directory.
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
    /// # use hopr_db_sql::HoprDbLogOperations;
    /// # async fn example(db: impl HoprDbLogOperations) -> Result<(), Box<dyn std::error::Error>> {
    /// let snapshot_dir = PathBuf::from("/tmp/snapshot_extracted");
    /// db.import_logs_db(snapshot_dir).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn import_logs_db(self, src_dir: PathBuf) -> Result<()>;


    /// Ensures that logs in this database have been created by scanning the given contract address
    /// and their corresponding topics. If the log DB is empty, the given addresses and topics
    /// are used to prime the table.
    ///
    /// # Arguments
    /// * `contract_address_topics` - list of topics for a contract address. There may be multiple topics
    /// with the same contract address.
    ///
    /// # Returns
    /// A `Result` which is `Ok(())` if the database contains correct log data,
    /// or it has been primed successfully. An `Err` is returned otherwise.
    async fn ensure_logs_origin(&self, contract_address_topics: Vec<(Address, Hash)>) -> Result<()>;

    /// Stores a single log entry in the database.
    ///
    /// # Arguments
    ///
    /// * `log` - The log entry to store, of type `SerializableLog`.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` if the operation succeeds or an error if it fails.
    async fn store_log<'a>(&'a self, log: SerializableLog) -> Result<()>;

    /// Stores multiple log entries in the database.
    ///
    /// # Arguments
    ///
    /// * `logs` - A vector of log entries to store, each of type `SerializableLog`.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `Result<()>`, each representing the result of storing an individual log entry.
    async fn store_logs(&self, logs: Vec<SerializableLog>) -> Result<Vec<Result<()>>>;

    /// Retrieves a specific log entry from the database.
    ///
    /// # Arguments
    ///
    /// * `block_number` - The block number of the log entry.
    /// * `tx_index` - The transaction index of the log entry.
    /// * `log_index` - The log index of the log entry.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `SerializableLog` if the operation succeeds or an error if it fails.
    async fn get_log(&self, block_number: u64, tx_index: u64, log_index: u64) -> Result<SerializableLog>;

    /// Retrieves multiple log entries from the database.
    ///
    /// # Arguments
    ///
    /// * `block_number` - An optional block number filter.
    /// * `block_offset` - An optional block offset filter.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec` of `SerializableLog` entries if the operation succeeds or an error if it fails.
    async fn get_logs<'a>(
        &'a self,
        block_number: Option<u64>,
        block_offset: Option<u64>,
    ) -> Result<Vec<SerializableLog>>;

    /// Retrieves the count of log entries from the database.
    ///
    /// # Arguments
    ///
    /// * `block_number` - An optional block number filter.
    /// * `block_offset` - An optional block offset filter.
    ///
    /// # Returns
    ///
    /// A `Result` containing the count of log entries if the operation succeeds or an error if it fails.
    async fn get_logs_count(&self, block_number: Option<u64>, block_offset: Option<u64>) -> Result<u64>;

    /// Retrieves block numbers of log entries from the database.
    ///
    /// # Arguments
    ///
    /// * `block_number` - An optional block number filter.
    /// * `block_offset` - An optional block offset filter.
    /// * `processed` - An optional processed filter.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec` of block numbers if the operation succeeds or an error if it fails.
    async fn get_logs_block_numbers<'a>(
        &'a self,
        block_number: Option<u64>,
        block_offset: Option<u64>,
        processed: Option<bool>,
    ) -> Result<Vec<u64>>;

    /// Marks a specific log entry as processed.
    ///
    /// # Arguments
    ///
    /// * `log` - The log entry to mark as processed, of type `SerializableLog`.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` if the operation succeeds or an error if it fails.
    async fn set_log_processed(&self, log: SerializableLog) -> Result<()>;

    /// Marks multiple log entries as processed.
    ///
    /// # Arguments
    ///
    /// * `block_number` - An optional block number filter.
    /// * `block_offset` - An optional block offset filter.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` if the operation succeeds or an error if it fails.
    async fn set_logs_processed(&self, block_number: Option<u64>, block_offset: Option<u64>) -> Result<()>;

    /// Marks multiple log entries as unprocessed.
    ///
    /// # Arguments
    ///
    /// * `block_number` - An optional block number filter.
    /// * `block_offset` - An optional block offset filter.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` if the operation succeeds or an error if it fails.
    async fn set_logs_unprocessed(&self, block_number: Option<u64>, block_offset: Option<u64>) -> Result<()>;

    /// Retrieves the last checksummed log entry from the database.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<SerializableLog>` if the operation succeeds or an error if it fails.
    async fn get_last_checksummed_log(&self) -> Result<Option<SerializableLog>>;

    /// Updates checksums for log entries in the database.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(Hash)` if the operation succeeds or an error if it fails.
    async fn update_logs_checksums(&self) -> Result<Hash>;
}
