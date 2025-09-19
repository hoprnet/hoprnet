use async_trait::async_trait;
use futures::{StreamExt, stream};
use hopr_crypto_types::prelude::Hash;
use hopr_db_entity::{
    errors::DbEntityError,
    log, log_status, log_topic_info,
    prelude::{Log, LogStatus, LogTopicInfo},
};
use hopr_primitive_types::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, FromQueryResult, IntoActiveModel, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
    entity::Set,
    query::QueryTrait,
    sea_query::{Expr, OnConflict, Value},
};
use tracing::{error, trace, warn};

use crate::{HoprDbGeneralModelOperations, HoprIndexerDb, TargetDb, errors::DbSqlError};

#[derive(FromQueryResult)]
struct BlockNumber {
    block_number: Vec<u8>,
}

#[async_trait]
pub trait HoprDbLogOperations {
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
    async fn ensure_logs_origin(&self, contract_address_topics: Vec<(Address, Hash)>) -> Result<(), DbSqlError>;

    /// Stores a single log entry in the database.
    ///
    /// # Arguments
    ///
    /// * `log` - The log entry to store, of type `SerializableLog`.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` if the operation succeeds or an error if it fails.
    async fn store_log<'a>(&'a self, log: SerializableLog) -> Result<(), DbSqlError>;

    /// Stores multiple log entries in the database.
    ///
    /// # Arguments
    ///
    /// * `logs` - A vector of log entries to store, each of type `SerializableLog`.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `Result<()>`, each representing the result of storing an individual log entry.
    async fn store_logs(&self, logs: Vec<SerializableLog>) -> Result<Vec<Result<(), DbSqlError>>, DbSqlError>;

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
    async fn get_log(&self, block_number: u64, tx_index: u64, log_index: u64) -> Result<SerializableLog, DbSqlError>;

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
    ) -> Result<Vec<SerializableLog>, DbSqlError>;

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
    async fn get_logs_count(&self, block_number: Option<u64>, block_offset: Option<u64>) -> Result<u64, DbSqlError>;

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
    ) -> Result<Vec<u64>, DbSqlError>;

    /// Marks a specific log entry as processed.
    ///
    /// # Arguments
    ///
    /// * `log` - The log entry to mark as processed, of type `SerializableLog`.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` if the operation succeeds or an error if it fails.
    async fn set_log_processed(&self, log: SerializableLog) -> Result<(), DbSqlError>;

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
    async fn set_logs_processed(&self, block_number: Option<u64>, block_offset: Option<u64>) -> Result<(), DbSqlError>;

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
    async fn set_logs_unprocessed(
        &self,
        block_number: Option<u64>,
        block_offset: Option<u64>,
    ) -> Result<(), DbSqlError>;

    /// Retrieves the last checksummed log entry from the database.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<SerializableLog>` if the operation succeeds or an error if it fails.
    async fn get_last_checksummed_log(&self) -> Result<Option<SerializableLog>, DbSqlError>;

    /// Updates checksums for log entries in the database.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(Hash)` if the operation succeeds or an error if it fails.
    async fn update_logs_checksums(&self) -> Result<Hash, DbSqlError>;
}

#[async_trait]
impl HoprDbLogOperations for HoprIndexerDb {
    async fn store_log<'a>(&'a self, log: SerializableLog) -> Result<(), DbSqlError> {
        let results = self.store_logs([log].to_vec()).await?;
        if let Some(result) = results.into_iter().next() {
            result
        } else {
            panic!("when inserting a log into the db, the result should be a single item")
        }
    }

    async fn store_logs(&self, logs: Vec<SerializableLog>) -> Result<Vec<Result<(), DbSqlError>>, DbSqlError> {
        self.nest_transaction_in_db(None, TargetDb::Logs)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let results = stream::iter(logs).then(|log| async {
                        let log_id = log.to_string();
                        let model = log::ActiveModel::from(log.clone());
                        let status_model = log_status::ActiveModel::from(log);
                        let log_status_query = LogStatus::insert(status_model).on_conflict(
                            OnConflict::columns([
                                log_status::Column::LogIndex,
                                log_status::Column::TransactionIndex,
                                log_status::Column::BlockNumber,
                            ])
                            .do_nothing()
                            .to_owned(),
                        );
                        let log_query = Log::insert(model).on_conflict(
                            OnConflict::columns([
                                log::Column::LogIndex,
                                log::Column::TransactionIndex,
                                log::Column::BlockNumber,
                            ])
                            .do_nothing()
                            .to_owned(),
                        );

                        match log_status_query.exec(tx.as_ref()).await {
                            Ok(_) => match log_query.exec(tx.as_ref()).await {
                                Ok(_) => Ok(()),
                                Err(DbErr::RecordNotInserted) => {
                                    warn!(log_id, "log already in the DB");
                                    Err(DbSqlError::LogicalError(format!(
                                        "log already exists in the DB: {log_id}"
                                    )))
                                }
                                Err(e) => {
                                    error!("Failed to insert log into db: {:?}", e);
                                    Err(DbSqlError::LogicalError(e.to_string()))
                                }
                            },
                            Err(DbErr::RecordNotInserted) => {
                                warn!(log_id, "log already in the DB");
                                Err(DbSqlError::LogicalError(format!(
                                    "log status already exists in the DB: {log_id}"
                                )))
                            }
                            Err(e) => {
                                error!(%log_id, "Failed to insert log status into db: {:?}", e);
                                Err(DbSqlError::LogicalError(e.to_string()))
                            }
                        }
                    });
                    Ok(results.collect::<Vec<_>>().await)
                })
            })
            .await
    }

    async fn get_log(&self, block_number: u64, tx_index: u64, log_index: u64) -> Result<SerializableLog, DbSqlError> {
        let bn_enc = block_number.to_be_bytes().to_vec();
        let tx_index_enc = tx_index.to_be_bytes().to_vec();
        let log_index_enc = log_index.to_be_bytes().to_vec();

        let query = Log::find()
            .filter(log::Column::BlockNumber.eq(bn_enc))
            .filter(log::Column::TransactionIndex.eq(tx_index_enc))
            .filter(log::Column::LogIndex.eq(log_index_enc))
            .find_also_related(LogStatus);

        let mut res = query.all(self.conn(TargetDb::Logs)).await?;
        if let Some((log, log_status)) = res.pop() {
            if let Some(status) = log_status {
                create_log(log, status)
            } else {
                Err(DbSqlError::MissingLogStatus)
            }
        } else {
            Err(DbSqlError::MissingLog)
        }
    }

    async fn get_logs<'a>(
        &'a self,
        block_number: Option<u64>,
        block_offset: Option<u64>,
    ) -> Result<Vec<SerializableLog>, DbSqlError> {
        let min_block_number = block_number.unwrap_or(0);
        let max_block_number = block_offset.map(|v| min_block_number + v + 1);

        let query = Log::find()
            .find_also_related(LogStatus)
            .filter(log::Column::BlockNumber.gte(min_block_number.to_be_bytes().to_vec()))
            .apply_if(max_block_number, |q, v| {
                q.filter(log::Column::BlockNumber.lt(v.to_be_bytes().to_vec()))
            })
            .order_by_asc(log::Column::BlockNumber)
            .order_by_asc(log::Column::TransactionIndex)
            .order_by_asc(log::Column::LogIndex);

        match query.all(self.conn(TargetDb::Logs)).await {
            Ok(logs) => Ok(logs
                .into_iter()
                .map(|(log, status)| {
                    if let Some(status) = status {
                        create_log(log, status).unwrap()
                    } else {
                        error!("Missing log status for log in db: {:?}", log);
                        SerializableLog::try_from(log).unwrap()
                    }
                })
                .collect()),
            Err(e) => {
                error!("Failed to get logs from db: {:?}", e);
                Err(DbSqlError::from(e))
            }
        }
    }

    async fn get_logs_count(&self, block_number: Option<u64>, block_offset: Option<u64>) -> Result<u64, DbSqlError> {
        let min_block_number = block_number.unwrap_or(0);
        let max_block_number = block_offset.map(|v| min_block_number + v + 1);

        Ok(Log::find()
            .select_only()
            .column(log::Column::BlockNumber)
            .column(log::Column::TransactionIndex)
            .column(log::Column::LogIndex)
            .filter(log::Column::BlockNumber.gte(min_block_number.to_be_bytes().to_vec()))
            .apply_if(max_block_number, |q, v| {
                q.filter(log::Column::BlockNumber.lt(v.to_be_bytes().to_vec()))
            })
            .count(self.conn(TargetDb::Logs))
            .await?)
    }

    async fn get_logs_block_numbers<'a>(
        &'a self,
        block_number: Option<u64>,
        block_offset: Option<u64>,
        processed: Option<bool>,
    ) -> Result<Vec<u64>, DbSqlError> {
        let min_block_number = block_number.unwrap_or(0);
        let max_block_number = block_offset.map(|v| min_block_number + v + 1);

        LogStatus::find()
            .select_only()
            .column(log_status::Column::BlockNumber)
            .distinct()
            .filter(log_status::Column::BlockNumber.gte(min_block_number.to_be_bytes().to_vec()))
            .apply_if(max_block_number, |q, v| {
                q.filter(log_status::Column::BlockNumber.lt(v.to_be_bytes().to_vec()))
            })
            .apply_if(processed, |q, v| q.filter(log_status::Column::Processed.eq(v)))
            .order_by_asc(log_status::Column::BlockNumber)
            .into_model::<BlockNumber>()
            .all(self.conn(TargetDb::Logs))
            .await
            .map(|res| {
                res.into_iter()
                    .map(|b| U256::from_be_bytes(b.block_number).as_u64())
                    .collect()
            })
            .map_err(|e| {
                error!("Failed to get logs block numbers from db: {:?}", e);
                DbSqlError::from(e)
            })
    }

    async fn set_logs_processed(&self, block_number: Option<u64>, block_offset: Option<u64>) -> Result<(), DbSqlError> {
        let min_block_number = block_number.unwrap_or(0);
        let max_block_number = block_offset.map(|v| min_block_number + v + 1);
        let now = Utc::now();

        let query = LogStatus::update_many()
            .col_expr(log_status::Column::Processed, Expr::value(Value::Bool(Some(true))))
            .col_expr(
                log_status::Column::ProcessedAt,
                Expr::value(Value::ChronoDateTimeUtc(Some(now.into()))),
            )
            .filter(log_status::Column::BlockNumber.gte(min_block_number.to_be_bytes().to_vec()))
            .apply_if(max_block_number, |q, v| {
                q.filter(log_status::Column::BlockNumber.lt(v.to_be_bytes().to_vec()))
            });

        match query.exec(self.conn(TargetDb::Logs)).await {
            Ok(_) => Ok(()),
            Err(e) => Err(DbSqlError::from(e)),
        }
    }

    async fn set_log_processed<'a>(&'a self, mut log: SerializableLog) -> Result<(), DbSqlError> {
        log.processed = Some(true);
        log.processed_at = Some(Utc::now());
        let log_status = log_status::ActiveModel::from(log);

        let db_tx = self.nest_transaction_in_db(None, TargetDb::Logs).await?;

        db_tx
            .perform(|tx| {
                Box::pin(async move {
                    match LogStatus::update(log_status).exec(tx.as_ref()).await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            error!("Failed to update log status in db");
                            Err(DbSqlError::from(e))
                        }
                    }
                })
            })
            .await
    }

    async fn set_logs_unprocessed(
        &self,
        block_number: Option<u64>,
        block_offset: Option<u64>,
    ) -> Result<(), DbSqlError> {
        let min_block_number = block_number.unwrap_or(0);
        let max_block_number = block_offset.map(|v| min_block_number + v + 1);

        let query = LogStatus::update_many()
            .col_expr(log_status::Column::Processed, Expr::value(Value::Bool(Some(false))))
            .col_expr(
                log_status::Column::ProcessedAt,
                Expr::value(Value::ChronoDateTimeUtc(None)),
            )
            .filter(log_status::Column::BlockNumber.gte(min_block_number.to_be_bytes().to_vec()))
            .apply_if(max_block_number, |q, v| {
                q.filter(log_status::Column::BlockNumber.lt(v.to_be_bytes().to_vec()))
            });

        match query.exec(self.conn(TargetDb::Logs)).await {
            Ok(_) => Ok(()),
            Err(e) => Err(DbSqlError::from(e)),
        }
    }

    async fn get_last_checksummed_log(&self) -> Result<Option<SerializableLog>, DbSqlError> {
        let query = LogStatus::find()
            .filter(log_status::Column::Checksum.is_not_null())
            .order_by_desc(log_status::Column::BlockNumber)
            .order_by_desc(log_status::Column::TransactionIndex)
            .order_by_desc(log_status::Column::LogIndex)
            .find_also_related(Log);

        match query.one(self.conn(TargetDb::Logs)).await {
            Ok(Some((status, Some(log)))) => {
                if let Ok(slog) = create_log(log, status) {
                    Ok(Some(slog))
                } else {
                    Ok(None)
                }
            }
            Ok(_) => Ok(None),
            Err(e) => Err(DbSqlError::from(e)),
        }
    }

    async fn update_logs_checksums(&self) -> Result<Hash, DbSqlError> {
        self.nest_transaction_in_db(None, TargetDb::Logs)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let mut last_checksum = LogStatus::find()
                        .filter(log_status::Column::Checksum.is_not_null())
                        .order_by_desc(log_status::Column::BlockNumber)
                        .order_by_desc(log_status::Column::TransactionIndex)
                        .order_by_desc(log_status::Column::LogIndex)
                        .one(tx.as_ref())
                        .await?
                        .and_then(|m| m.checksum)
                        .and_then(|c| Hash::try_from(c.as_slice()).ok())
                        .unwrap_or_default();

                    let query = LogStatus::find()
                        .filter(log_status::Column::Checksum.is_null())
                        .order_by_asc(log_status::Column::BlockNumber)
                        .order_by_asc(log_status::Column::TransactionIndex)
                        .order_by_asc(log_status::Column::LogIndex)
                        .find_also_related(Log);

                    match query.all(tx.as_ref()).await {
                        Ok(entries) => {
                            let mut entries = entries.into_iter();
                            while let Some((status, Some(log_entry))) = entries.next() {
                                let slog = create_log(log_entry.clone(), status.clone())?;
                                // we compute the hash of a single log as a combination of the block
                                // hash, TX hash, and the log index
                                let log_hash = Hash::create(&[
                                    log_entry.block_hash.as_slice(),
                                    log_entry.transaction_hash.as_slice(),
                                    log_entry.log_index.as_slice(),
                                ]);

                                let next_checksum = Hash::create(&[last_checksum.as_ref(), log_hash.as_ref()]);

                                let mut updated_status = status.into_active_model();
                                updated_status.checksum = Set(Some(next_checksum.as_ref().to_vec()));

                                match updated_status.update(tx.as_ref()).await {
                                    Ok(_) => {
                                        last_checksum = next_checksum;
                                        trace!(log = %slog, checksum = %next_checksum, "Generated log checksum");
                                    }
                                    Err(error) => {
                                        error!(%error, "Failed to update log status checksum in db");
                                        break;
                                    }
                                }
                            }
                            Ok(last_checksum)
                        }
                        Err(e) => Err(DbSqlError::from(e)),
                    }
                })
            })
            .await
    }

    async fn ensure_logs_origin(&self, contract_address_topics: Vec<(Address, Hash)>) -> Result<(), DbSqlError> {
        if contract_address_topics.is_empty() {
            return Err(DbSqlError::LogicalError(
                "contract address topics must not be empty".into(),
            ));
        }

        self.nest_transaction_in_db(None, TargetDb::Logs)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    // keep selected columns to a minimum to reduce copy overhead in db
                    let log_count = Log::find()
                        .select_only()
                        .column(log::Column::BlockNumber)
                        .column(log::Column::TransactionIndex)
                        .column(log::Column::LogIndex)
                        .count(tx.as_ref())
                        .await?;
                    let log_topic_count = LogTopicInfo::find().count(tx.as_ref()).await?;

                    if log_count == 0 && log_topic_count == 0 {
                        // Prime the DB with the values
                        LogTopicInfo::insert_many(contract_address_topics.into_iter().map(|(addr, topic)| {
                            log_topic_info::ActiveModel {
                                address: Set(addr.to_string()),
                                topic: Set(topic.to_string()),
                                ..Default::default()
                            }
                        }))
                        .exec(tx.as_ref())
                        .await?;
                    } else {
                        // Check that all contract addresses and topics are in the DB
                        for (addr, topic) in contract_address_topics {
                            let log_topic_count = LogTopicInfo::find()
                                .filter(log_topic_info::Column::Address.eq(addr.to_string()))
                                .filter(log_topic_info::Column::Topic.eq(topic.to_string()))
                                .count(tx.as_ref())
                                .await?;
                            if log_topic_count != 1 {
                                return Err(DbSqlError::InconsistentLogs);
                            }
                        }
                    }
                    Ok(())
                })
            })
            .await
    }
}

fn create_log(raw_log: log::Model, status: log_status::Model) -> crate::errors::Result<SerializableLog> {
    let log = SerializableLog::try_from(raw_log).map_err(DbSqlError::from)?;

    let checksum = if let Some(c) = status.checksum {
        let h: std::result::Result<[u8; 32], _> = c.try_into();

        if let Ok(hash) = h {
            Some(Hash::from(hash).to_hex())
        } else {
            return Err(DbSqlError::from(DbEntityError::ConversionError(
                "Invalid log checksum".into(),
            )));
        }
    } else {
        None
    };

    let log = if let Some(raw_ts) = status.processed_at {
        let ts = DateTime::<Utc>::from_naive_utc_and_offset(raw_ts, Utc);
        SerializableLog {
            processed: Some(status.processed),
            processed_at: Some(ts),
            checksum,
            ..log
        }
    } else {
        SerializableLog {
            processed: Some(status.processed),
            processed_at: None,
            checksum,
            ..log
        }
    };

    Ok(log)
}

#[cfg(test)]
mod tests {
    use hopr_crypto_types::prelude::{ChainKeypair, Hash, Keypair};

    use super::*;

    #[tokio::test]
    async fn test_store_single_log() {
        let db = HoprIndexerDb::new_in_memory(ChainKeypair::random()).await.unwrap();

        let log = SerializableLog {
            address: Address::new(b"my address 123456789"),
            topics: [Hash::create(&[b"my topic"]).into()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1u64,
            block_number: 1u64,
            block_hash: Hash::create(&[b"my block hash"]).into(),
            tx_hash: Hash::create(&[b"my tx hash"]).into(),
            log_index: 1u64,
            removed: false,
            processed: Some(false),
            ..Default::default()
        };

        db.store_log(log.clone()).await.unwrap();

        let logs = db.get_logs(None, None).await.unwrap();

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0], log);
    }

    #[tokio::test]
    async fn test_store_multiple_logs() {
        let db = HoprIndexerDb::new_in_memory(ChainKeypair::random()).await.unwrap();

        let log_1 = SerializableLog {
            address: Address::new(b"my address 123456789"),
            topics: [Hash::create(&[b"my topic"]).into()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1u64,
            block_number: 1u64,
            block_hash: Hash::create(&[b"my block hash"]).into(),
            tx_hash: Hash::create(&[b"my tx hash"]).into(),
            log_index: 1u64,
            removed: false,
            processed: Some(false),
            ..Default::default()
        };

        let log_2 = SerializableLog {
            address: Address::new(b"my address 223456789"),
            topics: [Hash::create(&[b"my topic 2"]).into()].into(),
            data: [1, 2, 3, 4, 5].into(),
            tx_index: 2u64,
            block_number: 2u64,
            block_hash: Hash::create(&[b"my block hash 2"]).into(),
            tx_hash: Hash::create(&[b"my tx hash 2"]).into(),
            log_index: 2u64,
            removed: false,
            processed: Some(true),
            ..Default::default()
        };

        db.store_log(log_1.clone()).await.unwrap();
        db.store_log(log_2.clone()).await.unwrap();

        let logs = db.get_logs(None, None).await.unwrap();

        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0], log_1);
        assert_eq!(logs[1], log_2);

        let log_2_retrieved = db
            .get_log(log_2.block_number, log_2.tx_index, log_2.log_index)
            .await
            .unwrap();

        assert_eq!(log_2, log_2_retrieved);
    }

    #[tokio::test]
    async fn test_store_duplicate_log() {
        let db = HoprIndexerDb::new_in_memory(ChainKeypair::random()).await.unwrap();

        let log = SerializableLog {
            address: Address::new(b"my address 123456789"),
            topics: [Hash::create(&[b"my topic"]).into()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1u64,
            block_number: 1u64,
            block_hash: Hash::create(&[b"my block hash"]).into(),
            tx_hash: Hash::create(&[b"my tx hash"]).into(),
            log_index: 1u64,
            removed: false,
            ..Default::default()
        };

        db.store_log(log.clone()).await.unwrap();

        db.store_log(log.clone())
            .await
            .expect_err("Expected error due to duplicate log insertion");

        let logs = db.get_logs(None, None).await.unwrap();

        assert_eq!(logs.len(), 1);
    }

    #[tokio::test]
    async fn test_set_log_processed() {
        let db = HoprIndexerDb::new_in_memory(ChainKeypair::random()).await.unwrap();

        let log = SerializableLog {
            address: Address::new(b"my address 123456789"),
            topics: [Hash::create(&[b"my topic"]).into()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1u64,
            block_number: 1u64,
            block_hash: Hash::create(&[b"my block hash"]).into(),
            tx_hash: Hash::create(&[b"my tx hash"]).into(),
            log_index: 1u64,
            removed: false,
            ..Default::default()
        };

        db.store_log(log.clone()).await.unwrap();

        let log_db = db.get_log(log.block_number, log.tx_index, log.log_index).await.unwrap();

        assert_eq!(log_db.processed, Some(false));
        assert_eq!(log_db.processed_at, None);

        db.set_log_processed(log.clone()).await.unwrap();

        let log_db_updated = db.get_log(log.block_number, log.tx_index, log.log_index).await.unwrap();

        assert_eq!(log_db_updated.processed, Some(true));
        assert!(log_db_updated.processed_at.is_some());
    }

    #[tokio::test]
    async fn test_list_logs_ordered() {
        let db = HoprIndexerDb::new_in_memory(ChainKeypair::random()).await.unwrap();

        let logs_per_tx = 3;
        let tx_per_block = 3;
        let blocks = 10;
        let start_block = 32183412;
        let base_log = SerializableLog {
            address: Address::new(b"my address 123456789"),
            topics: [Hash::create(&[b"my topic"]).into()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 0,
            block_number: 0,
            block_hash: Hash::create(&[b"my block hash"]).into(),
            tx_hash: Hash::create(&[b"my tx hash"]).into(),
            log_index: 0,
            removed: false,
            ..Default::default()
        };

        for block_offset in 0..blocks {
            for tx_index in 0..tx_per_block {
                for log_index in 0..logs_per_tx {
                    let log = SerializableLog {
                        tx_index,
                        block_number: start_block + block_offset,
                        log_index,
                        ..base_log.clone()
                    };
                    db.store_log(log).await.unwrap()
                }
            }
        }

        let block_fetch_interval = 3;
        let mut next_block = start_block;

        while next_block <= start_block + blocks {
            let ordered_logs = db.get_logs(Some(next_block), Some(block_fetch_interval)).await.unwrap();

            assert!(!ordered_logs.is_empty());

            ordered_logs.iter().reduce(|prev_log, curr_log| {
                assert!(prev_log.block_number >= next_block);
                assert!(prev_log.block_number <= (next_block + block_fetch_interval));
                assert!(curr_log.block_number >= next_block);
                assert!(curr_log.block_number <= (next_block + block_fetch_interval));
                if prev_log.block_number == curr_log.block_number {
                    if prev_log.tx_index == curr_log.tx_index {
                        assert!(prev_log.log_index < curr_log.log_index);
                    } else {
                        assert!(prev_log.tx_index < curr_log.tx_index);
                    }
                } else {
                    assert!(prev_log.block_number < curr_log.block_number);
                }
                curr_log
            });
            next_block += block_fetch_interval;
        }
    }

    #[tokio::test]
    async fn test_get_nonexistent_log() {
        let db = HoprIndexerDb::new_in_memory(ChainKeypair::random()).await.unwrap();

        let result = db.get_log(999, 999, 999).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_logs_with_block_offset() {
        let db = HoprIndexerDb::new_in_memory(ChainKeypair::random()).await.unwrap();

        let log_1 = SerializableLog {
            address: Address::new(b"my address 123456789"),
            topics: [Hash::create(&[b"topic1"]).into()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1,
            block_number: 1,
            block_hash: Hash::create(&[b"block_hash1"]).into(),
            tx_hash: Hash::create(&[b"tx_hash1"]).into(),
            log_index: 1,
            removed: false,
            processed: Some(false),
            ..Default::default()
        };

        let log_2 = SerializableLog {
            address: Address::new(b"my address 223456789"),
            topics: [Hash::create(&[b"topic2"]).into()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 2,
            block_number: 2,
            block_hash: Hash::create(&[b"block_hash2"]).into(),
            tx_hash: Hash::create(&[b"tx_hash2"]).into(),
            log_index: 2,
            removed: false,
            processed: Some(false),
            ..Default::default()
        };

        db.store_logs(vec![log_1.clone(), log_2.clone()])
            .await
            .unwrap()
            .into_iter()
            .for_each(|r| assert!(r.is_ok()));

        let logs = db.get_logs(Some(1), Some(0)).await.unwrap();

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0], log_1);
    }

    #[tokio::test]
    async fn test_set_logs_unprocessed() {
        let db = HoprIndexerDb::new_in_memory(ChainKeypair::random()).await.unwrap();

        let log = SerializableLog {
            address: Address::new(b"my address 123456789"),
            topics: [Hash::create(&[b"topic"]).into()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1,
            block_number: 1,
            block_hash: Hash::create(&[b"block_hash"]).into(),
            tx_hash: Hash::create(&[b"tx_hash"]).into(),
            log_index: 1,
            removed: false,
            processed: Some(true),
            processed_at: Some(Utc::now()),
            ..Default::default()
        };

        db.store_log(log.clone()).await.unwrap();

        db.set_logs_unprocessed(Some(1), Some(0)).await.unwrap();

        let log_db = db.get_log(log.block_number, log.tx_index, log.log_index).await.unwrap();

        assert_eq!(log_db.processed, Some(false));
        assert!(log_db.processed_at.is_none());
    }

    #[tokio::test]
    async fn test_get_logs_block_numbers() {
        let db = HoprIndexerDb::new_in_memory(ChainKeypair::random()).await.unwrap();

        let log_1 = SerializableLog {
            address: Address::new(b"my address 123456789"),
            topics: [Hash::create(&[b"topic1"]).into()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1,
            block_number: 1,
            block_hash: Hash::create(&[b"block_hash1"]).into(),
            tx_hash: Hash::create(&[b"tx_hash1"]).into(),
            log_index: 1,
            removed: false,
            processed: Some(true),
            ..Default::default()
        };

        let log_2 = SerializableLog {
            address: Address::new(b"my address 223456789"),
            topics: [Hash::create(&[b"topic2"]).into()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 2,
            block_number: 2,
            block_hash: Hash::create(&[b"block_hash2"]).into(),
            tx_hash: Hash::create(&[b"tx_hash2"]).into(),
            log_index: 2,
            removed: false,
            processed: Some(false),
            ..Default::default()
        };

        let log_3 = SerializableLog {
            address: Address::new(b"my address 323456789"),
            topics: [Hash::create(&[b"topic3"]).into()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 3,
            block_number: 3,
            block_hash: Hash::create(&[b"block_hash3"]).into(),
            tx_hash: Hash::create(&[b"tx_hash3"]).into(),
            log_index: 3,
            removed: false,
            processed: Some(false),
            ..Default::default()
        };

        db.store_logs(vec![log_1.clone(), log_2.clone(), log_3.clone()])
            .await
            .unwrap()
            .into_iter()
            .for_each(|r| assert!(r.is_ok()));

        let block_numbers_all = db.get_logs_block_numbers(None, None, None).await.unwrap();
        assert_eq!(block_numbers_all.len(), 3);
        assert_eq!(block_numbers_all, [1, 2, 3]);

        let block_numbers_first_only = db.get_logs_block_numbers(Some(1), Some(0), None).await.unwrap();
        assert_eq!(block_numbers_first_only.len(), 1);
        assert_eq!(block_numbers_first_only[0], 1);

        let block_numbers_last_only = db.get_logs_block_numbers(Some(3), Some(0), None).await.unwrap();
        assert_eq!(block_numbers_last_only.len(), 1);
        assert_eq!(block_numbers_last_only[0], 3);

        let block_numbers_processed = db.get_logs_block_numbers(None, None, Some(true)).await.unwrap();
        assert_eq!(block_numbers_processed.len(), 1);
        assert_eq!(block_numbers_processed[0], 1);

        let block_numbers_unprocessed_second = db.get_logs_block_numbers(Some(2), Some(0), Some(false)).await.unwrap();
        assert_eq!(block_numbers_unprocessed_second.len(), 1);
        assert_eq!(block_numbers_unprocessed_second[0], 2);
    }

    #[tokio::test]
    async fn test_update_logs_checksums() {
        let db = HoprIndexerDb::new_in_memory(ChainKeypair::random()).await.unwrap();

        // insert first log and update checksum
        let log_1 = SerializableLog {
            address: Address::new(b"my address 123456789"),
            topics: [Hash::create(&[b"topic"]).into()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1,
            block_number: 1,
            block_hash: Hash::create(&[b"block_hash"]).into(),
            tx_hash: Hash::create(&[b"tx_hash"]).into(),
            log_index: 1,
            removed: false,
            ..Default::default()
        };

        db.store_log(log_1.clone()).await.unwrap();

        assert!(db.get_last_checksummed_log().await.unwrap().is_none());

        db.update_logs_checksums().await.unwrap();

        let updated_log_1 = db.get_last_checksummed_log().await.unwrap().unwrap();
        assert!(updated_log_1.checksum.is_some());

        // insert two more logs and update checksums
        let log_2 = SerializableLog {
            block_number: 2,
            ..log_1.clone()
        };
        let log_3 = SerializableLog {
            block_number: 3,
            ..log_1.clone()
        };

        db.store_logs(vec![log_2.clone(), log_3.clone()])
            .await
            .unwrap()
            .into_iter()
            .for_each(|r| assert!(r.is_ok()));

        // ensure the first log is still the last updated
        assert_eq!(
            updated_log_1.clone().checksum.unwrap(),
            db.get_last_checksummed_log().await.unwrap().unwrap().checksum.unwrap()
        );

        db.update_logs_checksums().await.unwrap();

        let updated_log_3 = db.get_last_checksummed_log().await.unwrap().unwrap();

        db.get_logs(None, None).await.unwrap().into_iter().for_each(|log| {
            assert!(log.checksum.is_some());
        });

        // ensure the first log is not the last updated anymore
        assert_ne!(
            updated_log_1.clone().checksum.unwrap(),
            updated_log_3.clone().checksum.unwrap(),
        );
        assert_ne!(updated_log_1, updated_log_3);
    }

    #[tokio::test]
    async fn test_should_not_allow_inconsistent_logs_in_the_db() -> anyhow::Result<()> {
        let db = HoprIndexerDb::new_in_memory(ChainKeypair::random()).await?;
        let addr_1 = Address::new(b"my address 123456789");
        let addr_2 = Address::new(b"my 2nd address 12345");
        let topic_1 = Hash::create(&[b"my topic 1"]);
        let topic_2 = Hash::create(&[b"my topic 2"]);

        db.ensure_logs_origin(vec![(addr_1, topic_1)]).await?;

        db.ensure_logs_origin(vec![(addr_1, topic_2)])
            .await
            .expect_err("expected error due to inconsistent logs in the db");

        db.ensure_logs_origin(vec![(addr_2, topic_1)])
            .await
            .expect_err("expected error due to inconsistent logs in the db");

        db.ensure_logs_origin(vec![(addr_1, topic_1)]).await?;

        Ok(())
    }
}
