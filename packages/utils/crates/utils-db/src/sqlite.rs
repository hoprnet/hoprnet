use crate::traits::{AsyncKVStorage, BatchOperation, StorageValueIterator};
use async_trait::async_trait;
use sqlx::sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteStatement};
use sqlx::{Executor, SqlitePool, Statement};
use std::fmt::Debug;
use std::ops::DerefMut;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct SqliteShim<'a> {
    pool: SqlitePool,
    insert: SqliteStatement<'a>,
    delete: SqliteStatement<'a>,
    select: SqliteStatement<'a>,
    iter: SqliteStatement<'a>,
}

pub const SQL_TABLE_LABEL: &str = "1";
pub const SQL_DB_FILE_NAME: &str = "hoprd.sqlite";

impl SqliteShim<'_> {
    async fn new_from_pool(pool: SqlitePool) -> Result<Self, sqlx::Error> {
        // NOTE: the table name cannot be bound as a parameter, but since `label` is
        // a compile time constant, there should be no SQL possible injections here
        // TODO: use migrations for this
        sqlx::query(&const_format::formatcp!(
            "CREATE TABLE IF NOT EXISTS kv_{SQL_TABLE_LABEL} (key BLOB PRIMARY KEY, value BLOB)"
        ))
        .execute(&pool)
        .await?;

        let insert = pool
            .prepare(&const_format::formatcp!(
                "INSERT INTO kv_{SQL_TABLE_LABEL} VALUES (?, ?) ON CONFLICT (key) DO UPDATE SET value=excluded.value"
            ))
            .await?;

        let delete = pool
            .prepare(&const_format::formatcp!(
                "DELETE FROM kv_{SQL_TABLE_LABEL} WHERE key = ?"
            ))
            .await?;

        let select = pool
            .prepare(&const_format::formatcp!(
                "SELECT value FROM kv_{SQL_TABLE_LABEL} WHERE key = ?"
            ))
            .await?;

        let iter = pool
            .prepare(&const_format::formatcp!(
                "SELECT value FROM kv_{SQL_TABLE_LABEL} WHERE key >= ? AND key <= ? ORDER BY key ASC"
            ))
            .await?;

        // Just log information on how many entries there are
        let count: i64 = sqlx::query_scalar(&const_format::formatcp!("SELECT COUNT(*) FROM kv_{SQL_TABLE_LABEL}"))
            .fetch_one(&pool)
            .await?;
        log::debug!("initialized database with {count} existing entries");

        Ok(Self {
            pool,
            insert,
            delete,
            select,
            iter,
        })
    }

    pub async fn new(directory: &str, create_if_missing: bool) -> Self {
        let dir = Path::new(directory);
        std::fs::create_dir_all(dir).unwrap_or_else(|_| panic!("cannot create main database directory {directory}")); // hard-failure

        let pool = SqlitePool::connect_with(
            SqliteConnectOptions::default()
                .filename(dir.join(SQL_DB_FILE_NAME))
                .create_if_missing(create_if_missing)
                .journal_mode(SqliteJournalMode::Wal)
                .auto_vacuum(SqliteAutoVacuum::Full)
                .page_size(4096),
        )
        .await
        .unwrap_or_else(|e| panic!("failed to create main database: {e}"));

        Self::new_from_pool(pool)
            .await
            .unwrap_or_else(|e| panic!("cannot initialize db: {e}"))
    }

    pub async fn new_in_memory() -> Self {
        Self::new_from_pool(SqlitePool::connect(":memory:").await.unwrap())
            .await
            .unwrap_or_else(|e| panic!("cannot initialize db: {e}"))
    }
}

#[async_trait]
impl AsyncKVStorage for SqliteShim<'_> {
    type Key = Box<[u8]>;
    type Value = Box<[u8]>;

    async fn get(&self, key: Self::Key) -> crate::errors::Result<Option<Self::Value>> {
        let v: Option<Box<[u8]>> = self.select.query_scalar().bind(key).fetch_optional(&self.pool).await?;
        // Empty value is coerced to `None` to adhere with LevelDB
        Ok(v.filter(|v| v.len() > 0))
    }

    async fn set(&mut self, key: Self::Key, value: Self::Value) -> crate::errors::Result<Option<Self::Value>> {
        self.insert.query().bind(key).bind(value).execute(&self.pool).await?;
        Ok(None)
    }

    async fn contains(&self, key: Self::Key) -> crate::errors::Result<bool> {
        let v: Option<Box<[u8]>> = self.select.query_scalar().bind(key).fetch_optional(&self.pool).await?;
        Ok(v.is_some())
    }

    async fn remove(&mut self, key: Self::Key) -> crate::errors::Result<Option<Self::Value>> {
        self.delete.query().bind(key).execute(&self.pool).await?;
        Ok(None)
    }

    async fn dump(&self, _destination: String) -> crate::errors::Result<()> {
        Ok(())
    }

    async fn iterate(
        &self,
        prefix: Self::Key,
        suffix_size: u32,
    ) -> crate::errors::Result<StorageValueIterator<Self::Value>> {
        let mut first_key: Vec<u8> = prefix.clone().into();
        first_key.extend((0..suffix_size).map(|_| 0u8));

        let mut last_key: Vec<u8> = prefix.into();
        last_key.extend((0..suffix_size).map(|_| 0xffu8));

        self.iterate_range(first_key.into_boxed_slice(), last_key.into_boxed_slice())
            .await
    }

    async fn iterate_range(
        &self,
        start: Self::Key,
        end: Self::Key,
    ) -> crate::errors::Result<StorageValueIterator<Self::Value>> {
        let values: Vec<(Box<[u8]>,)> = self.iter.query_as().bind(start).bind(end).fetch_all(&self.pool).await?;

        Ok(Box::new(values.into_iter().map(|v| Ok(v.0))))
    }

    async fn batch(
        &mut self,
        operations: Vec<BatchOperation<Self::Key, Self::Value>>,
        _wait_for_write: bool,
    ) -> crate::errors::Result<()> {
        let mut tx = self.pool.begin().await?;

        for op in operations {
            match op {
                BatchOperation::del(del) => self.delete.query().bind(del.key).execute(tx.deref_mut()).await?,
                BatchOperation::put(ins) => {
                    self.insert
                        .query()
                        .bind(ins.key)
                        .bind(ins.value)
                        .execute(tx.deref_mut())
                        .await?
                }
            };
        }

        Ok(tx.commit().await?)
    }

    async fn flush(&mut self) -> crate::errors::Result<()> {
        // Does nothing in SQLite
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::sqlite::SqliteShim;

    #[async_std::test]
    async fn test_sqlite() {
        SqliteShim::new("/tmp/", true).await;
    }

    #[async_std::test]
    async fn sqlite_sanity_test() {
        use crate::traits::{AsyncKVStorage, BatchOperation};

        let key_1 = "1";
        let value_1 = "abc";
        let key_2 = "2";
        let value_2 = "def";
        let key_3 = "3";
        let value_3 = "ghi";
        let key_4 = "1";
        let prefix = "xy";
        let prefixed_key_1 = "xya";
        let prefixed_key_2 = "xyb";
        let prefixed_key_3 = "xyc";

        let mut kv_storage = super::SqliteShim::new_in_memory().await;

        assert!(
            !kv_storage.contains(key_1.as_bytes().into()).await.unwrap(),
            "Test #1 failed: empty DB should not contain any data"
        );

        let _ = kv_storage.set(key_1.as_bytes().into(), value_1.as_bytes().into()).await;

        assert!(
            kv_storage.contains(key_1.as_bytes().into()).await.unwrap(),
            "Test #2 failed: DB should contain the key"
        );

        let value = kv_storage
            .get(key_1.as_bytes().into())
            .await
            .unwrap()
            .expect("Stored empty value");
        let value_converted = std::str::from_utf8(value.as_ref()).unwrap();

        assert_eq!(
            value_converted, value_1,
            "Test #3 failed: DB value after get should be equal to the one before the get"
        );

        let _ = kv_storage.remove(key_1.as_bytes().into()).await;
        assert!(
            !kv_storage.contains(key_1.as_bytes().into()).await.unwrap(),
            "Test #4 failed: removal of key from the DB failed"
        );

        let batch_data = vec![
            BatchOperation::put(crate::traits::Put {
                key: key_3.as_bytes().into(),
                value: value_3.as_bytes().into(),
            }),
            BatchOperation::put(crate::traits::Put {
                key: key_2.as_bytes().into(),
                value: value_2.as_bytes().into(),
            }),
            BatchOperation::del(crate::traits::Del {
                key: key_2.as_bytes().into(),
            }),
        ];
        assert!(
            kv_storage.batch(batch_data, true).await.is_ok(),
            "Test #5.0 failed: batch operation failed"
        );

        // ===================================

        async_std::task::sleep(std::time::Duration::from_millis(10)).await;

        assert!(
            kv_storage.contains(key_3.as_bytes().into()).await.unwrap(),
            "Test #5.1 failed: the key should be present in the DB"
        );

        kv_storage
            .set(key_4.as_bytes().into(), Box::new([]))
            .await
            .expect("Could not write empty value");

        assert!(kv_storage.contains(key_4.as_bytes().into()).await.unwrap());

        assert_eq!(
            kv_storage.get(key_4.as_bytes().into()).await,
            Ok(None),
            "Test #6 failed: Could not read empty value from DB"
        );

        // ===================================

        let _ = kv_storage
            .set(prefixed_key_1.as_bytes().into(), value_1.as_bytes().into())
            .await;
        let _ = kv_storage
            .set(prefixed_key_2.as_bytes().into(), value_2.as_bytes().into())
            .await;
        let _ = kv_storage
            .set(prefixed_key_3.as_bytes().into(), value_3.as_bytes().into())
            .await;

        let expected = vec![value_1.as_bytes().into(), value_3.as_bytes().into()];

        let mut received = Vec::new();
        let mut data_stream = kv_storage
            .iterate(prefix.as_bytes().into(), (prefixed_key_1.len() - prefix.len()) as u32)
            .await
            .unwrap();

        while let Some(value) = data_stream.next() {
            let v = value.unwrap();

            if v.as_ref() != value_2.as_bytes() {
                received.push(v);
            }
        }
        assert_eq!(received, expected, "Test #7 failed: db content mismatch");
    }
}
