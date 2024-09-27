use async_stream::stream;
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use futures::TryStreamExt;
use sea_orm::query::QueryTrait;
use sea_orm::sea_query::{Expr, OnConflict, Value};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, ModelTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Related, StreamTrait,
};
use std::str::FromStr;
use tracing::error;

use hopr_db_api::errors::{DbError, Result};
use hopr_db_api::logs::HoprDbLogOperations;
use hopr_db_entity::errors::DbEntityError;
use hopr_db_entity::prelude::{Log, LogStatus};
use hopr_db_entity::{log, log_status};
use hopr_primitive_types::prelude::*;

use crate::db::HoprDb;
use crate::errors::DbSqlError;
use crate::TargetDb;
use crate::{HoprDbGeneralModelOperations, OptTx};

#[async_trait]
impl HoprDbLogOperations for HoprDb {
    async fn store_log<'a>(&'a self, log: SerializableLog) -> Result<()> {
        let model = log::ActiveModel::from(log.clone());
        let status_model = log_status::ActiveModel::from(log);

        self.nest_transaction_in_db(None, TargetDb::Logs)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    match log_status::Entity::insert(status_model)
                        .on_conflict(
                            OnConflict::columns([
                                log_status::Column::LogIndex,
                                log_status::Column::TransactionIndex,
                                log_status::Column::BlockNumber,
                            ])
                            .do_nothing()
                            .to_owned(),
                        )
                        .exec(tx.as_ref())
                        .await
                    {
                        Ok(_) => {
                            match log::Entity::insert(model)
                                .on_conflict(
                                    OnConflict::columns([
                                        log::Column::LogIndex,
                                        log::Column::TransactionIndex,
                                        log::Column::BlockNumber,
                                    ])
                                    .do_nothing()
                                    .to_owned(),
                                )
                                .exec(tx.as_ref())
                                .await
                            {
                                Ok(_) => Ok(()),
                                Err(DbErr::RecordNotInserted) => {
                                    error!("Failed to insert log status into db");
                                    Err(DbError::from(DbSqlError::from(DbErr::RecordNotInserted)))
                                }
                                Err(e) => Err(DbError::General(e.to_string())),
                            }
                        }
                        Err(DbErr::RecordNotInserted) => {
                            error!("Failed to insert log into db");
                            Err(DbError::from(DbSqlError::from(DbErr::RecordNotInserted)))
                        }
                        Err(e) => Err(DbError::General(e.to_string())),
                    }
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
                        Ok(create_log(log, status))
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

        let query = Log::find()
            .find_also_related(LogStatus)
            .filter(log::Column::BlockNumber.gte(min_block_number.to_be_bytes().to_vec()))
            .apply_if(block_offset, |q, v| {
                q.filter(log::Column::BlockNumber.lt((min_block_number + v).to_be_bytes().to_vec()))
            })
            .order_by_asc(log::Column::BlockNumber)
            .order_by_asc(log::Column::TransactionIndex)
            .order_by_asc(log::Column::LogIndex);

        Ok(Box::pin(stream! {
            match query.stream(self.conn(TargetDb::Logs)).await {
                Ok(mut stream) => {
                    while let Ok(Some(object)) = stream.try_next().await {
                        match object {
                            (log, Some(log_status)) => yield create_log(log, log_status),
                            (log, None) => {
                                error!("Missing log status for log in db: {:?}", log);
                                yield SerializableLog::from(log)
                            }
                        }
                    }
                },
                Err(e) => error!("Failed to get logs from db: {:?}", e),
            }
        }))
    }

    async fn set_log_processed<'a>(&'a self, mut log: SerializableLog) -> Result<()> {
        log.processed = Some(true);
        log.processed_at = Some(Utc::now());
        let log_status = log_status::ActiveModel::from(log);

        let db_tx = self.nest_transaction_in_db(None, TargetDb::Logs).await?;

        db_tx
            .perform(|tx| {
                Box::pin(async move {
                    match log_status::Entity::update(log_status).exec(tx.as_ref()).await {
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

        let query = LogStatus::update_many()
            .col_expr(log_status::Column::Processed, Expr::value(Value::Bool(Some(false))))
            .col_expr(log_status::Column::ProcessedAt, Expr::value(Value::String(None)))
            .filter(log::Column::BlockNumber.gte(min_block_number.to_be_bytes().to_vec()))
            .apply_if(block_offset, |q, v| {
                q.filter(log::Column::BlockNumber.lt((min_block_number + v).to_be_bytes().to_vec()))
            });

        match query.exec(self.conn(TargetDb::Logs)).await {
            Ok(_) => Ok(()),
            Err(e) => Err(DbError::from(DbSqlError::from(e))),
        }
    }
}

fn create_log(raw_log: log::Model, status: log_status::Model) -> SerializableLog {
    let log = SerializableLog::from(raw_log);
    if let Some(raw_ts) = status.processed_at {
        let ts = DateTime::<Utc>::from_str(raw_ts.as_str())
            .map_err(|e| {
                error!("Failed to decode processed_at {} of log status {}", raw_ts, log);
                e
            })
            .ok();
        SerializableLog {
            processed: Some(status.processed),
            processed_at: ts,
            ..log
        }
    } else {
        SerializableLog {
            processed: Some(status.processed),
            processed_at: None,
            ..log
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::DbSqlError;
    use crate::errors::DbSqlError::DecodingError;
    use crate::HoprDbGeneralModelOperations;
    use futures::{stream, StreamExt};
    use hopr_crypto_types::prelude::{ChainKeypair, Hash, Keypair, OffchainKeypair};
    use hopr_internal_types::prelude::AccountType::NotAnnounced;

    #[async_std::test]
    async fn test_store_single_log() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

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
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

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
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

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
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

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
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let logs_per_tx = 5;
        let tx_per_block = 10;
        let blocks = 100;
        let start_block = 32183412;

        for block_offset in 0..blocks {
            for tx_index in 0..tx_per_block {
                for log_index in 0..logs_per_tx {
                    let log = SerializableLog {
                        address: Hash::create(&[b"my address"]).to_hex(),
                        topics: [Hash::create(&[b"my topic"]).to_hex()].into(),
                        data: [1, 2, 3, 4].into(),
                        tx_index,
                        block_number: start_block + block_offset,
                        block_hash: Hash::create(&[b"my block hash"]).to_hex(),
                        tx_hash: Hash::create(&[b"my tx hash"]).to_hex(),
                        log_index,
                        removed: false,
                        ..Default::default()
                    };
                    db.store_log(log).await.unwrap()
                }
            }
        }

        let block_fetch_interval = 842;
        let mut next_block = start_block;

        while next_block <= start_block + blocks {
            let ordered_logs = db
                .get_logs(Some(next_block), Some(block_fetch_interval))
                .await
                .unwrap()
                .collect::<Vec<SerializableLog>>()
                .await;

            ordered_logs.iter().reduce(|prev_log, curr_log| {
                assert!(prev_log.block_number >= next_block);
                assert!(prev_log.block_number < (next_block + block_fetch_interval));
                assert!(curr_log.block_number >= next_block);
                assert!(curr_log.block_number < (next_block + block_fetch_interval));
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
}
