use async_trait::async_trait;
use futures::{stream, StreamExt};
use sea_orm::entity::Set;
use sea_orm::query::QueryTrait;
use sea_orm::sea_query::{Expr, OnConflict, Value};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, FromQueryResult, IntoActiveModel, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use tracing::{error, trace};

use hopr_crypto_types::prelude::Hash;
use hopr_db_api::errors::{DbError, Result};
use hopr_db_api::logs::HoprDbLogOperations;
use hopr_db_entity::errors::DbEntityError;
use hopr_db_entity::prelude::{Log, LogStatus, LogTopicInfo};
use hopr_db_entity::{log, log_status, log_topic_info};
use hopr_primitive_types::prelude::*;

use crate::db::HoprDb;
use crate::errors::DbSqlError;
use crate::{HoprDbGeneralModelOperations, TargetDb};

#[derive(FromQueryResult)]
struct BlockNumber {
    block_number: Vec<u8>,
}

#[async_trait]
impl HoprDbLogOperations for HoprDb {
    async fn store_log<'a>(&'a self, log: SerializableLog) -> Result<()> {
        match self.store_logs([log].to_vec()).await {
            Ok(results) => {
                if let Some(result) = results.into_iter().next() {
                    result
                } else {
                    panic!("when inserting a log into the db, the result should be a single item")
                }
            }
            Err(e) => Err(e),
        }
    }

    async fn store_logs(&self, logs: Vec<SerializableLog>) -> Result<Vec<Result<()>>> {
        self.nest_transaction_in_db(None, TargetDb::Logs)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let results = stream::iter(logs).then(|log| async {
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
                                Err(e) => {
                                    error!("Failed to insert log into db: {:?}", e);
                                    Err(DbError::General(e.to_string()))
                                }
                            },
                            Err(e) => {
                                error!("Failed to insert log status into db: {:?}", e);
                                Err(DbError::General(e.to_string()))
                            }
                        }
                    });
                    Ok(results.collect::<Vec<_>>().await)
                })
            })
            .await
    }

    async fn get_log(&self, block_number: u64, tx_index: u64, log_index: u64) -> Result<SerializableLog> {
        let bn_enc = block_number.to_be_bytes().to_vec();
        let tx_index_enc = tx_index.to_be_bytes().to_vec();
        let log_index_enc = log_index.to_be_bytes().to_vec();

        let query = Log::find()
            .filter(log::Column::BlockNumber.eq(bn_enc))
            .filter(log::Column::TransactionIndex.eq(tx_index_enc))
            .filter(log::Column::LogIndex.eq(log_index_enc))
            .find_also_related(LogStatus);

        match query.all(self.conn(TargetDb::Logs)).await {
            Ok(mut res) => {
                if let Some((log, log_status)) = res.pop() {
                    if let Some(status) = log_status {
                        create_log(log, status).map_err(DbError::from)
                    } else {
                        Err(DbError::MissingLogStatus)
                    }
                } else {
                    Err(DbError::MissingLog)
                }
            }
            Err(e) => Err(DbError::from(DbSqlError::from(e))),
        }
    }

    async fn get_logs<'a>(
        &'a self,
        block_number: Option<u64>,
        block_offset: Option<u64>,
    ) -> Result<Vec<SerializableLog>> {
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
                Err(DbError::from(DbSqlError::from(e)))
            }
        }
    }

    async fn get_logs_count(&self, block_number: Option<u64>, block_offset: Option<u64>) -> Result<u64> {
        let min_block_number = block_number.unwrap_or(0);
        let max_block_number = block_offset.map(|v| min_block_number + v + 1);

        Log::find()
            .filter(log::Column::BlockNumber.gte(min_block_number.to_be_bytes().to_vec()))
            .apply_if(max_block_number, |q, v| {
                q.filter(log::Column::BlockNumber.lt(v.to_be_bytes().to_vec()))
            })
            .count(self.conn(TargetDb::Logs))
            .await
            .map_err(|e| DbSqlError::from(e).into())
    }

    async fn get_logs_block_numbers<'a>(
        &'a self,
        block_number: Option<u64>,
        block_offset: Option<u64>,
        processed: Option<bool>,
    ) -> Result<Vec<u64>> {
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
                DbError::from(DbSqlError::from(e))
            })
    }

    async fn set_logs_processed(&self, block_number: Option<u64>, block_offset: Option<u64>) -> Result<()> {
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
            Err(e) => Err(DbError::from(DbSqlError::from(e))),
        }
    }

    async fn set_log_processed<'a>(&'a self, mut log: SerializableLog) -> Result<()> {
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
                            Err(DbError::from(DbSqlError::from(e)))
                        }
                    }
                })
            })
            .await
    }

    async fn set_logs_unprocessed(&self, block_number: Option<u64>, block_offset: Option<u64>) -> Result<()> {
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
            Err(e) => Err(DbError::from(DbSqlError::from(e))),
        }
    }

    async fn get_last_checksummed_log(&self) -> Result<Option<SerializableLog>> {
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
            Err(e) => Err(DbError::from(DbSqlError::from(e))),
        }
    }

    async fn update_logs_checksums(&self) -> Result<Hash> {
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
                        .await
                        .map_err(|e| DbError::from(DbSqlError::from(e)))?
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
                        Err(e) => Err(DbError::from(DbSqlError::from(e))),
                    }
                })
            })
            .await
    }

    async fn ensure_logs_origin(&self, contract_address_topics: Vec<(Address, Hash)>) -> Result<()> {
        self.nest_transaction_in_db(None, TargetDb::Logs)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let log_count = Log::find()
                        .count(tx.as_ref())
                        .await
                        .map_err(|e| DbError::from(DbSqlError::from(e)))?;
                    let log_topic_count = LogTopicInfo::find()
                        .count(tx.as_ref())
                        .await
                        .map_err(|e| DbError::from(DbSqlError::from(e)))?;

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
                        .await
                        .map_err(|e| DbError::from(DbSqlError::from(e)))?;
                    } else {
                        // Check that all contract addresses and topics are in the DB
                        for (addr, topic) in contract_address_topics {
                            let log_topic_count = LogTopicInfo::find()
                                .filter(log_topic_info::Column::Address.eq(addr.to_string()))
                                .filter(log_topic_info::Column::Topic.eq(topic.to_string()))
                                .count(tx.as_ref())
                                .await
                                .map_err(|e| DbError::from(DbSqlError::from(e)))?;
                            if log_topic_count != 1 {
                                return Err(DbError::InconsistentLogs);
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
    use super::*;

    use hopr_crypto_types::prelude::{ChainKeypair, Hash, Keypair};

    #[async_std::test]
    async fn test_store_single_log() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await.unwrap();

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

    #[async_std::test]
    async fn test_store_multiple_logs() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await.unwrap();

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

    #[async_std::test]
    async fn test_store_duplicate_log() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await.unwrap();

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

    #[async_std::test]
    async fn test_set_log_processed() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await.unwrap();

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
        assert_eq!(log_db_updated.processed_at.is_some(), true);
    }

    #[async_std::test]
    async fn test_list_logs_ordered() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await.unwrap();

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

            assert!(ordered_logs.len() > 0);

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
            next_block = next_block + block_fetch_interval;
        }
    }

    #[async_std::test]
    async fn test_get_nonexistent_log() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await.unwrap();

        let result = db.get_log(999, 999, 999).await;

        assert!(result.is_err());
    }

    #[async_std::test]
    async fn test_get_logs_with_block_offset() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await.unwrap();

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

    #[async_std::test]
    async fn test_set_logs_unprocessed() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await.unwrap();

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

    #[async_std::test]
    async fn test_get_logs_block_numbers() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await.unwrap();

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

    #[async_std::test]
    async fn test_update_logs_checksums() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await.unwrap();

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
}
