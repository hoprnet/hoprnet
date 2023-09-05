use async_trait::async_trait;
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

use crate::errors::DbError;
use crate::traits::{AsyncKVStorage, BatchOperation, StorageValueIterator};
use futures_lite::stream::iter;
use rusty_leveldb::{DBIterator, LdbIterator, StatusCode, WriteBatch};

struct RustyLevelDbIterator {
    iter: DBIterator,
    first_key: Box<[u8]>,
    last_key: Box<[u8]>,
}

impl RustyLevelDbIterator {
    pub fn new(iter: DBIterator, prefix: &[u8], suffix_len: usize) -> Self {
        let mut first_key: Vec<u8> = prefix.into();
        first_key.extend((0..suffix_len).map(|_| 0u8));

        let mut last_key: Vec<u8> = prefix.into();
        last_key.extend((0..suffix_len).map(|_| 0xffu8));

        // This implementation does not use the `seek` method, because it is not working properly
        Self {
            iter,
            first_key: first_key.into_boxed_slice(),
            last_key: last_key.into_boxed_slice(),
        }
    }
}

impl Iterator for RustyLevelDbIterator {
    type Item = crate::errors::Result<Box<[u8]>>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((key, value)) = self.iter.next() {
            let upper_bound = key.as_slice().cmp(&self.last_key);
            let lower_bound = key.as_slice().cmp(&self.first_key);
            if upper_bound != Ordering::Greater && lower_bound != Ordering::Less {
                return Some(Ok(value.into_boxed_slice()));
            } else if upper_bound == Ordering::Greater {
                return None;
            }
        }
        None
    }
}

/// Adapter for Rusty Level DB database.
#[derive(Clone)]
pub struct RustyLevelDbShim {
    db: Arc<Mutex<rusty_leveldb::DB>>,
}

impl RustyLevelDbShim {
    /// Create adapter from the given Rusty LevelDB instance.
    pub fn new(path: &str) -> Self {
        if path == ":memory" {
            Self {
                db: Arc::new(Mutex::new(rusty_leveldb::DB::open("hopr", rusty_leveldb::in_memory())
                    .expect("failed to create DB")))
            }
        } else {
            todo!("when nodejs_env is implemented, change to FS")
        }
    }
}

#[async_trait(?Send)]
impl AsyncKVStorage for RustyLevelDbShim {
    type Key = Box<[u8]>;
    type Value = Box<[u8]>;

    async fn get(&self, key: Self::Key) -> crate::errors::Result<Option<Self::Value>> {
        let mut db = self.db.lock().unwrap();

        let snapshot = db.get_snapshot();
        match db.get_at(&snapshot, &key) {
            Ok(Some(val)) => Ok(if val.len() > 0 {
                Some(val.into_boxed_slice())
            } else {
                None
            }),
            Ok(None) => Ok(None),
            Err(e) => Err(if e.code == StatusCode::NotFound {
                DbError::NotFound
            } else {
                DbError::GenericError(e.to_string())
            }),
        }
    }

    async fn set(&mut self, key: Self::Key, value: Self::Value) -> crate::errors::Result<Option<Self::Value>> {
        self.db
            .lock()
            .unwrap()
            .put(&key, &value)
            .map(|_| None)
            .map_err(|e| DbError::GenericError(e.err))
    }

    async fn contains(&self, key: Self::Key) -> crate::errors::Result<bool> {
        Ok(self.db.lock().unwrap().get(&key).is_some())
    }

    async fn remove(&mut self, key: Self::Key) -> crate::errors::Result<Option<Self::Value>> {
        self.db
            .lock()
            .unwrap()
            .delete(&key)
            .map(|_| None)
            .map_err(|e| DbError::GenericError(e.err))
    }

    async fn dump(&self, _destination: String) -> crate::errors::Result<()> {
        Ok(())
    }

    fn iterate(
        &self,
        prefix: Self::Key,
        suffix_size: u32,
    ) -> crate::errors::Result<StorageValueIterator<Self::Value>> {
        let i = self
            .db
            .lock()
            .unwrap()
            .new_iter()
            .map_err(|e| DbError::GenericError(e.err))?;
        Ok(Box::new(iter(RustyLevelDbIterator::new(
            i,
            &prefix,
            suffix_size as usize,
        ))))
    }

    async fn batch(
        &mut self,
        operations: Vec<BatchOperation<Self::Key, Self::Value>>,
        wait_for_write: bool,
    ) -> crate::errors::Result<()> {
        let mut wb = WriteBatch::new();
        for op in operations {
            match op {
                BatchOperation::del(x) => wb.delete(&x.key),
                BatchOperation::put(x) => wb.put(&x.key, &x.value),
            }
        }

        self.db
            .lock()
            .unwrap()
            .write(wb, wait_for_write)
            .map_err(|e| DbError::GenericError(e.err))
    }
}


#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    #[async_std::test]
    async fn rusty_leveldb_sanity_test() {
        use crate::traits::{AsyncKVStorage, BatchOperation};
        use futures_lite::StreamExt;

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

        let opt = rusty_leveldb::in_memory();
        let mut kv_storage = crate::rusty::RustyLevelDbShim::new(":memory");

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
        let mut data_stream = Box::into_pin(
            kv_storage
                .iterate(prefix.as_bytes().into(), (prefixed_key_1.len() - prefix.len()) as u32)
                .unwrap(),
        );

        while let Some(value) = data_stream.next().await {
            let v = value.unwrap();

            if v.as_ref() != value_2.as_bytes() {
                received.push(v);
            }
        }
        assert_eq!(received, expected, "Test #7 failed: db content mismatch");
    }
}
