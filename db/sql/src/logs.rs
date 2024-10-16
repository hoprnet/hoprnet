use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::{stream, StreamExt, TryStreamExt};
use sea_orm::entity::Set;
use sea_orm::query::QueryTrait;
use sea_orm::sea_query::{Expr, OnConflict, Value};
use sea_orm::{ColumnTrait, DbErr, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use tracing::{error, trace};

use hopr_crypto_types::prelude::Hash;
use hopr_db_api::errors::{DbError, Result};
use hopr_db_api::logs::HoprDbLogOperations;
use hopr_db_entity::errors::DbEntityError;
use hopr_db_entity::prelude::{Log, LogStatus};
use hopr_db_entity::{log, log_status};
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
                                Err(DbErr::RecordNotInserted) => {
                                    error!("Failed to insert log into db");
                                    Err(DbError::from(DbSqlError::from(DbErr::RecordNotInserted)))
                                }
                                Err(e) => Err(DbError::General(e.to_string())),
                            },
                            Err(DbErr::RecordNotInserted) => {
                                error!("Failed to insert log status into db");
                                Err(DbError::from(DbSqlError::from(DbErr::RecordNotInserted)))
                            }
                            Err(e) => Err(DbError::General(e.to_string())),
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
    ) -> Result<BoxStream<'a, SerializableLog>> {
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

        Ok(Box::pin(async_stream::stream! {
            match query.stream(self.conn(TargetDb::Logs)).await {
                Ok(mut stream) => {
                    while let Ok(Some(object)) = stream.try_next().await {
                        match object {
                            (log, Some(log_status)) => {
                                if let Ok(slog) = create_log(log, log_status) {
                                    yield slog
                                }
                            },
                            (log, None) => {
                                error!("Missing log status for log in db: {:?}", log);
                                if let Ok(slog) = SerializableLog::try_from(log) {
                                    yield slog
                                }
                            }
                        }
                    }
                },
                Err(e) => error!("Failed to get logs from db: {:?}", e),
            }
        }))
    }

    async fn get_logs_count(&self, block_number: Option<u64>, block_offset: Option<u64>) -> Result<u64> {
        let min_block_number = block_number.unwrap_or(0);
        let max_block_number = block_offset.map(|v| min_block_number + v + 1);

        let query = Log::find()
            .filter(log::Column::BlockNumber.gte(min_block_number.to_be_bytes().to_vec()))
            .apply_if(max_block_number, |q, v| {
                q.filter(log::Column::BlockNumber.lt(v.to_be_bytes().to_vec()))
            })
            .order_by_asc(log::Column::BlockNumber)
            .order_by_asc(log::Column::TransactionIndex)
            .order_by_asc(log::Column::LogIndex);

        match query.count(self.conn(TargetDb::Logs)).await {
            Ok(count) => Ok(count),
            Err(e) => Err(DbError::from(DbSqlError::from(e))),
        }
    }

    async fn get_logs_block_numbers<'a>(
        &'a self,
        block_number: Option<u64>,
        block_offset: Option<u64>,
    ) -> Result<BoxStream<'a, u64>> {
        let min_block_number = block_number.unwrap_or(0);
        let max_block_number = block_offset.map(|v| min_block_number + v + 1);

        let query = Log::find()
            .select_only()
            .column(log::Column::BlockNumber)
            .distinct()
            .filter(log::Column::BlockNumber.gte(min_block_number.to_be_bytes().to_vec()))
            .apply_if(max_block_number, |q, v| {
                q.filter(log::Column::BlockNumber.lt(v.to_be_bytes().to_vec()))
            })
            .order_by_asc(log::Column::BlockNumber)
            .into_model::<BlockNumber>();

        Ok(Box::pin(async_stream::stream! {
            match query.stream(self.conn(TargetDb::Logs)).await {
                Ok(mut stream) => {
                    while let Some(Ok(object)) = stream.next().await {
                        yield U256::from_be_bytes(object.block_number).as_u64()
                }},
                Err(e) => error!("Failed to get logs block numbers from db: {:?}", e),
            }
        }))
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
                        Err(DbErr::RecordNotUpdated) => {
                            error!("Failed to update log status in db");
                            Err(DbError::from(DbSqlError::UpdateLogStatusError))
                        }
                        Err(e) => Err(DbError::from(DbSqlError::from(e))),
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
            .col_expr(log_status::Column::ProcessedAt, Expr::value(Value::String(None)))
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

        match query.clone().one(self.conn(TargetDb::Logs)).await {
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

    async fn update_logs_checksums(&self) -> Result<()> {
        let last_log = self.get_last_checksummed_log().await?;
        let mut last_checksum = last_log.map_or(Hash::default(), |log| {
            Hash::from_hex(log.checksum.unwrap().as_str()).unwrap()
        });
        let db_tx = self.nest_transaction_in_db(None, TargetDb::Logs).await?;

        db_tx
            .perform(|tx| {
                Box::pin(async move {
                    let query = LogStatus::find()
                        .filter(log_status::Column::Checksum.is_null())
                        .order_by_asc(log_status::Column::BlockNumber)
                        .order_by_asc(log_status::Column::TransactionIndex)
                        .order_by_asc(log_status::Column::LogIndex)
                        .find_also_related(Log);

                    match query.all(tx.as_ref()).await {
                        Ok(mut entries) => {
                            while let Some((status, Some(log_entry))) = entries.pop() {
                                let slog = create_log(log_entry.clone(), status.clone())?;
                                // we compute the has of a single log as a combination of the block
                                // hash, tx hash and log index
                                let log_hash = Hash::create(&[
                                    log_entry.block_hash.as_slice(),
                                    log_entry.transaction_hash.as_slice(),
                                    log_entry.log_index.as_slice(),
                                ]);
                                let next_checksum = Hash::create(&[last_checksum.as_ref(), log_hash.as_ref()]);
                                let updated_status = log_status::ActiveModel {
                                    checksum: Set(Some(next_checksum.as_ref().to_vec())),
                                    ..status.into()
                                };
                                match LogStatus::update(updated_status).exec(tx.as_ref()).await {
                                    Ok(_) => {
                                        last_checksum = next_checksum;
                                        trace!("Generated log checksum {next_checksum} @ {slog}");
                                    }
                                    Err(e) => {
                                        error!("Failed to update log status checksum in db: {:?}", e);
                                        break;
                                    }
                                }
                            }
                            Ok(())
                        }
                        Err(e) => Err(DbError::from(DbSqlError::from(e))),
                    }
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

    use futures::StreamExt;
    use hopr_crypto_types::prelude::{ChainKeypair, Hash, Keypair};

    #[async_std::test]
    async fn test_store_single_log() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await.unwrap();

        let log = SerializableLog {
            address: Hash::create(&[b"my address"]).to_hex(),
            topics: [Hash::create(&[b"my topic"]).to_hex()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1u64,
            block_number: 1u64,
            block_hash: Hash::create(&[b"my block hash"]).to_hex(),
            tx_hash: Hash::create(&[b"my tx hash"]).to_hex(),
            log_index: 1u64,
            removed: false,
            processed: Some(false),
            ..Default::default()
        };

        db.store_log(log.clone()).await.unwrap();

        let logs = db
            .get_logs(None, None)
            .await
            .unwrap()
            .collect::<Vec<SerializableLog>>()
            .await;

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0], log);
    }

    #[async_std::test]
    async fn test_store_multiple_logs() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await.unwrap();

        let log_1 = SerializableLog {
            address: Hash::create(&[b"my address"]).to_hex(),
            topics: [Hash::create(&[b"my topic"]).to_hex()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1u64,
            block_number: 1u64,
            block_hash: Hash::create(&[b"my block hash"]).to_hex(),
            tx_hash: Hash::create(&[b"my tx hash"]).to_hex(),
            log_index: 1u64,
            removed: false,
            processed: Some(false),
            ..Default::default()
        };

        let log_2 = SerializableLog {
            address: Hash::create(&[b"my address 2"]).to_hex(),
            topics: [Hash::create(&[b"my topic 2"]).to_hex()].into(),
            data: [1, 2, 3, 4, 5].into(),
            tx_index: 2u64,
            block_number: 2u64,
            block_hash: Hash::create(&[b"my block hash 2"]).to_hex(),
            tx_hash: Hash::create(&[b"my tx hash 2"]).to_hex(),
            log_index: 2u64,
            removed: false,
            processed: Some(true),
            ..Default::default()
        };

        db.store_log(log_1.clone()).await.unwrap();
        db.store_log(log_2.clone()).await.unwrap();

        let logs = db
            .get_logs(None, None)
            .await
            .unwrap()
            .collect::<Vec<SerializableLog>>()
            .await;

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
            address: Hash::create(&[b"my address"]).to_hex(),
            topics: [Hash::create(&[b"my topic"]).to_hex()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1u64,
            block_number: 1u64,
            block_hash: Hash::create(&[b"my block hash"]).to_hex(),
            tx_hash: Hash::create(&[b"my tx hash"]).to_hex(),
            log_index: 1u64,
            removed: false,
            ..Default::default()
        };

        db.store_log(log.clone()).await.unwrap();

        db.store_log(log.clone())
            .await
            .expect_err("should not store duplicate log");

        let logs = db
            .get_logs(None, None)
            .await
            .unwrap()
            .collect::<Vec<SerializableLog>>()
            .await;

        assert_eq!(logs.len(), 1);
    }

    #[async_std::test]
    async fn test_set_log_processed() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await.unwrap();

        let log = SerializableLog {
            address: Hash::create(&[b"my address"]).to_hex(),
            topics: [Hash::create(&[b"my topic"]).to_hex()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1u64,
            block_number: 1u64,
            block_hash: Hash::create(&[b"my block hash"]).to_hex(),
            tx_hash: Hash::create(&[b"my tx hash"]).to_hex(),
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
            address: Hash::create(&[b"my address"]).to_hex(),
            topics: [Hash::create(&[b"my topic"]).to_hex()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 0,
            block_number: 0,
            block_hash: Hash::create(&[b"my block hash"]).to_hex(),
            tx_hash: Hash::create(&[b"my tx hash"]).to_hex(),
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
            let ordered_logs = db
                .get_logs(Some(next_block), Some(block_fetch_interval))
                .await
                .unwrap()
                .collect::<Vec<SerializableLog>>()
                .await;

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
            address: Hash::create(&[b"address1"]).to_hex(),
            topics: [Hash::create(&[b"topic1"]).to_hex()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1,
            block_number: 1,
            block_hash: Hash::create(&[b"block_hash1"]).to_hex(),
            tx_hash: Hash::create(&[b"tx_hash1"]).to_hex(),
            log_index: 1,
            removed: false,
            processed: Some(false),
            ..Default::default()
        };

        let log_2 = SerializableLog {
            address: Hash::create(&[b"address2"]).to_hex(),
            topics: [Hash::create(&[b"topic2"]).to_hex()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 2,
            block_number: 2,
            block_hash: Hash::create(&[b"block_hash2"]).to_hex(),
            tx_hash: Hash::create(&[b"tx_hash2"]).to_hex(),
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

        let logs = db
            .get_logs(Some(1), Some(0))
            .await
            .unwrap()
            .collect::<Vec<SerializableLog>>()
            .await;

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0], log_1);
    }

    #[async_std::test]
    async fn test_set_logs_unprocessed() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await.unwrap();

        let log = SerializableLog {
            address: Hash::create(&[b"address"]).to_hex(),
            topics: [Hash::create(&[b"topic"]).to_hex()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1,
            block_number: 1,
            block_hash: Hash::create(&[b"block_hash"]).to_hex(),
            tx_hash: Hash::create(&[b"tx_hash"]).to_hex(),
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
            address: Hash::create(&[b"address1"]).to_hex(),
            topics: [Hash::create(&[b"topic1"]).to_hex()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1,
            block_number: 1,
            block_hash: Hash::create(&[b"block_hash1"]).to_hex(),
            tx_hash: Hash::create(&[b"tx_hash1"]).to_hex(),
            log_index: 1,
            removed: false,
            processed: Some(false),
            ..Default::default()
        };

        let log_2 = SerializableLog {
            address: Hash::create(&[b"address2"]).to_hex(),
            topics: [Hash::create(&[b"topic2"]).to_hex()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 2,
            block_number: 2,
            block_hash: Hash::create(&[b"block_hash2"]).to_hex(),
            tx_hash: Hash::create(&[b"tx_hash2"]).to_hex(),
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

        let block_numbers = db
            .get_logs_block_numbers(Some(1), Some(0))
            .await
            .unwrap()
            .collect::<Vec<u64>>()
            .await;

        assert_eq!(block_numbers.len(), 1);
        assert_eq!(block_numbers[0], 1);
    }

    #[async_std::test]
    async fn test_update_logs_checksums() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await.unwrap();

        // insert first log and update checksum
        let log_1 = SerializableLog {
            address: Hash::create(&[b"address"]).to_hex(),
            topics: [Hash::create(&[b"topic"]).to_hex()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1,
            block_number: 1,
            block_hash: Hash::create(&[b"block_hash"]).to_hex(),
            tx_hash: Hash::create(&[b"tx_hash"]).to_hex(),
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

        db.get_logs(None, None)
            .await
            .unwrap()
            .collect::<Vec<SerializableLog>>()
            .await
            .into_iter()
            .for_each(|log| {
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
