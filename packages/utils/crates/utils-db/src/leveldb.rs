#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::errors::DbError;
    use crate::traits::{AsyncKVStorage, BatchOperation, StorageValueIterator};
    use async_trait::async_trait;
    use futures_lite::stream::StreamExt;
    use wasm_bindgen::prelude::*;

    // https://users.rust-lang.org/t/wasm-web-sys-how-to-manipulate-js-objects-from-rust/36504
    #[wasm_bindgen]
    extern "C" {
        pub type LevelDb;

        #[wasm_bindgen(method, catch)]
        async fn has(this: &LevelDb, key: Box<[u8]>) -> Result<JsValue, JsValue>; // bool;

        #[wasm_bindgen(method, catch)]
        async fn put(this: &LevelDb, key: Box<[u8]>, value: Box<[u8]>) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        async fn get(this: &LevelDb, key: Box<[u8]>) -> Result<JsValue, JsValue>; // Option<Box<[u8]>>;

        #[wasm_bindgen(method, catch)]
        async fn remove(this: &LevelDb, key: Box<[u8]>) -> Result<JsValue, JsValue>; // Option<Box<[u8]>>;

        // https://github.com/Level/levelup#dbbatcharray-options-callback-array-form
        #[wasm_bindgen(method, catch)]
        async fn batch(this: &LevelDb, operations: js_sys::Array, wait_for_write: bool) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        fn iterValues(this: &LevelDb, prefix: js_sys::Uint8Array, suffix_length: u32) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(method, catch)]
        fn dump(this: &LevelDb, destination: String) -> Result<(), JsValue>;
    }

    pub struct LevelDbShim {
        db: LevelDb,
    }

    impl LevelDbShim {
        pub fn new(db: LevelDb) -> LevelDbShim {
            LevelDbShim { db }
        }
    }

    #[async_trait(? Send)]
    impl AsyncKVStorage for LevelDbShim {
        type Key = Box<[u8]>;
        type Value = Box<[u8]>;

        async fn get(&self, key: Self::Key) -> crate::errors::Result<Self::Value> {
            self.db
                .get(key)
                .await
                .map_err(|_| DbError::GenericError("Encountered error on DB get operation".to_string()))
                .and_then(|v| {
                    if v.is_undefined() {
                        Err(DbError::NotFound)
                    } else {
                        Ok(js_sys::Uint8Array::from(v).to_vec().into_boxed_slice())
                    }
                })
        }

        async fn set(&mut self, key: Self::Key, value: Self::Value) -> crate::errors::Result<Option<Self::Value>> {
            self.db
                .put(key, value)
                .await
                .map(|_| None) // NOTE: The LevelDB API does not allow to return an evicted value
                .map_err(|_| DbError::GenericError("Encountered error on DB put operation".to_string()))
        }

        async fn contains(&self, key: Self::Key) -> bool {
            self.db.has(key).await.map(|v| v.as_bool().unwrap()).unwrap_or(false)
        }

        async fn remove(&mut self, key: Self::Key) -> crate::errors::Result<Option<Self::Value>> {
            self.db
                .remove(key)
                .await
                .map(|v| {
                    if v.is_undefined() {
                        // NOTE: The LevelDB API does not allow to return an evicted value
                        None
                    } else {
                        Some(js_sys::Uint8Array::from(v).to_vec().into_boxed_slice())
                    }
                })
                .map_err(|_| DbError::GenericError("Encountered error on DB remove operation".to_string()))
        }

        async fn dump(&self, destination: String) -> crate::errors::Result<()> {
            self.db
                .dump(destination.clone())
                .map_err(|_| DbError::DumpError(format!("Failed to dump DB into {}", destination)))
        }

        fn iterate(
            &self,
            prefix: Self::Key,
            suffix_size: u32,
        ) -> crate::errors::Result<StorageValueIterator<Self::Value>> {
            let iterable = self
                .db
                .iterValues(js_sys::Uint8Array::from(prefix.as_ref()), suffix_size)
                .map(|v| js_sys::AsyncIterator::from(v))
                .map_err(|e| DbError::GenericError(format!("Iteration failed with an exception: {:?}", e)))?;

            let stream = wasm_bindgen_futures::stream::JsStream::from(iterable);

            Ok(Box::new(crate::types::BinaryStreamWrapper::new(stream)))
        }
        async fn batch(
            &mut self,
            operations: Vec<BatchOperation<Self::Key, Self::Value>>,
            wait_for_write: bool,
        ) -> crate::errors::Result<()> {
            let ops = operations
                .into_iter()
                .map(|op| serde_wasm_bindgen::to_value(&op))
                .collect::<Vec<_>>();

            if ops.iter().any(|i| i.is_err()) {
                Err(DbError::GenericError(
                    "Batch operation contains a deserialization error, aborting".to_string(),
                ))
            } else {
                let ops: js_sys::Array = ops
                    .into_iter()
                    .filter_map(|op| op.ok().map(|v| JsValue::from(&v)))
                    .collect();
                self.db
                    .batch(ops, wait_for_write)
                    .await
                    .map_err(|e| DbError::GenericError(format!("Batch operation failed to write data: {:?}", e)))
            }
        }
    }

    #[wasm_bindgen]
    pub async fn db_sanity_test(db: LevelDb) -> Result<bool, JsValue> {
        let key_1 = "1";
        let value_1 = "abc";
        let key_2 = "2";
        let value_2 = "def";
        let key_3 = "3";
        let value_3 = "ghi";
        let prefix = "xy";
        let prefixed_key_1 = "xya";
        let prefixed_key_2 = "xyb";
        let prefixed_key_3 = "xyc";

        let mut kv_storage = LevelDbShim::new(db);

        // if let Err(e) = kv_storage.dump("/tmp/level.db".to_owned()).await {
        //     return Err::<bool, JsValue>(
        //         JsValue::from(JsError::new(format!("Test #0 failed: {:?}", e).as_str())))
        // }

        if kv_storage.contains(key_1.as_bytes().to_vec().into_boxed_slice()).await {
            return Err::<bool, JsValue>(JsValue::from(JsError::new(
                "Test #1 failed: empty DB should not contain any data",
            )));
        }

        // ===================================

        let _ = kv_storage
            .set(
                key_1.as_bytes().to_vec().into_boxed_slice(),
                value_1.as_bytes().to_vec().into_boxed_slice(),
            )
            .await;
        if !kv_storage.contains(key_1.as_bytes().to_vec().into_boxed_slice()).await {
            return Err::<bool, JsValue>(JsValue::from(JsError::new("Test #2 failed: DB should contain the key")));
        }

        // ===================================

        let value = kv_storage
            .get(key_1.as_bytes().to_vec().into_boxed_slice())
            .await
            .unwrap();
        let value_converted = std::str::from_utf8(value.as_ref())
            .map_err(|_| JsValue::from(JsError::new("Test #3.0 failed: could not convert the get type")))?;

        if value_converted != value_1 {
            return Err::<bool, JsValue>(JsValue::from(JsError::new(
                "Test #3.1 failed: DB value after get should be equal to the one before the get",
            )));
        }

        // ===================================

        let _ = kv_storage.remove(key_1.as_bytes().to_vec().into_boxed_slice()).await;

        if kv_storage.contains(key_1.as_bytes().to_vec().into_boxed_slice()).await {
            return Err::<bool, JsValue>(JsValue::from(JsError::new(
                "Test #4 failed: removal of key from the DB failed",
            )));
        }

        // ===================================

        let batch_data = vec![
            BatchOperation::put(crate::traits::Put {
                key: key_3.as_bytes().to_vec().into_boxed_slice(),
                value: value_3.as_bytes().to_vec().into_boxed_slice(),
            }),
            BatchOperation::put(crate::traits::Put {
                key: key_2.as_bytes().to_vec().into_boxed_slice(),
                value: value_2.as_bytes().to_vec().into_boxed_slice(),
            }),
            BatchOperation::del(crate::traits::Del {
                key: key_2.as_bytes().to_vec().into_boxed_slice(),
            }),
        ];
        if let Err(e) = kv_storage.batch(batch_data, true).await {
            return Err::<bool, JsValue>(JsValue::from(JsError::new(
                format!("Test #5.0 failed: batch operation failed: {}", e.to_string()).as_str(),
            )));
        }

        // ===================================

        gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;

        // TODO: levelup api with the passed options to do an immediate write does not perform an immediate write
        if !kv_storage.contains(key_3.as_bytes().to_vec().into_boxed_slice()).await {
            return Err::<bool, JsValue>(JsValue::from(JsError::new(
                "Test #5.1 failed: the key should be present in the DB",
            )));
        }

        // ===================================

        let _ = kv_storage
            .set(
                prefixed_key_1.as_bytes().to_vec().into_boxed_slice(),
                value_1.as_bytes().to_vec().into_boxed_slice(),
            )
            .await;
        let _ = kv_storage
            .set(
                prefixed_key_2.as_bytes().to_vec().into_boxed_slice(),
                value_2.as_bytes().to_vec().into_boxed_slice(),
            )
            .await;
        let _ = kv_storage
            .set(
                prefixed_key_3.as_bytes().to_vec().into_boxed_slice(),
                value_3.as_bytes().to_vec().into_boxed_slice(),
            )
            .await;

        let expected = vec![
            value_1.as_bytes().to_vec().into_boxed_slice(),
            value_3.as_bytes().to_vec().into_boxed_slice(),
        ];

        let mut received = Vec::new();
        let mut data_stream = Box::into_pin(
            kv_storage
                .iterate(
                    prefix.as_bytes().to_vec().into_boxed_slice(),
                    (prefixed_key_1.len() - prefix.len()) as u32,
                )
                .map_err(|e| {
                    JsValue::from(JsError::new(
                        format!("Test #6.1 failed: failed to iterate over DB {:?}", e).as_str(),
                    ))
                })?,
        );

        while let Some(value) = data_stream.next().await {
            let v = value
                .map(|v| js_sys::Uint8Array::from(v.as_ref()).to_vec().into_boxed_slice())
                .map_err(|e| {
                    JsValue::from(JsError::new(
                        format!("Test #6.1 failed: failed to iterate over DB {:?}", e).as_str(),
                    ))
                })?;

            if v.as_ref() != value_2.as_bytes() {
                received.push(v);
            }
        }

        if received != expected {
            return Err::<bool, JsValue>(JsValue::from(JsError::new(
                format!("Test #6.2 failed: db content mismatch {:?} != {:?}", received, expected).as_str(),
            )));
        }

        Ok(true)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod rusty {
    use async_trait::async_trait;
    use std::cmp::Ordering;
    use std::sync::{Arc, Mutex};

    use crate::errors::DbError;
    use crate::traits::{AsyncKVStorage, BatchOperation, StorageValueIterator};
    use futures_lite::stream::iter;
    use rusty_leveldb::{DBIterator, LdbIterator, WriteBatch};

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
    pub struct RustyLevelDbShim {
        db: Arc<Mutex<rusty_leveldb::DB>>,
    }

    impl RustyLevelDbShim {
        /// Create adapter from the given Rusty LevelDB instance.
        pub fn new(db: Arc<Mutex<rusty_leveldb::DB>>) -> Self {
            Self { db }
        }
    }

    #[async_trait(?Send)]
    impl AsyncKVStorage for RustyLevelDbShim {
        type Key = Box<[u8]>;
        type Value = Box<[u8]>;

        async fn get(&self, key: Self::Key) -> crate::errors::Result<Self::Value> {
            self.db
                .lock()
                .unwrap()
                .get(&key)
                .ok_or(DbError::NotFound)
                .map(|v| v.into_boxed_slice())
        }

        async fn set(&mut self, key: Self::Key, value: Self::Value) -> crate::errors::Result<Option<Self::Value>> {
            self.db
                .lock()
                .unwrap()
                .put(&key, &value)
                .map(|_| None)
                .map_err(|e| DbError::GenericError(e.err))
        }

        async fn contains(&self, key: Self::Key) -> bool {
            self.db.lock().unwrap().get(&key).is_some()
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
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    #[cfg(not(target_arch = "wasm32"))]
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
        let prefix = "xy";
        let prefixed_key_1 = "xya";
        let prefixed_key_2 = "xyb";
        let prefixed_key_3 = "xyc";

        let opt = rusty_leveldb::in_memory();
        let mut kv_storage = crate::leveldb::rusty::RustyLevelDbShim::new(Arc::new(Mutex::new(
            rusty_leveldb::DB::open("test", opt).unwrap(),
        )));

        assert!(
            !kv_storage.contains(key_1.as_bytes().to_vec().into_boxed_slice()).await,
            "Test #1 failed: empty DB should not contain any data"
        );

        let _ = kv_storage
            .set(
                key_1.as_bytes().to_vec().into_boxed_slice(),
                value_1.as_bytes().to_vec().into_boxed_slice(),
            )
            .await;

        assert!(
            kv_storage.contains(key_1.as_bytes().to_vec().into_boxed_slice()).await,
            "Test #2 failed: DB should contain the key"
        );

        let value = kv_storage
            .get(key_1.as_bytes().to_vec().into_boxed_slice())
            .await
            .unwrap();
        let value_converted = std::str::from_utf8(value.as_ref()).unwrap();

        assert_eq!(
            value_converted, value_1,
            "Test #3 failed: DB value after get should be equal to the one before the get"
        );

        let _ = kv_storage.remove(key_1.as_bytes().to_vec().into_boxed_slice()).await;
        assert!(
            !kv_storage.contains(key_1.as_bytes().to_vec().into_boxed_slice()).await,
            "Test #4 failed: removal of key from the DB failed"
        );

        let batch_data = vec![
            BatchOperation::put(crate::traits::Put {
                key: key_3.as_bytes().to_vec().into_boxed_slice(),
                value: value_3.as_bytes().to_vec().into_boxed_slice(),
            }),
            BatchOperation::put(crate::traits::Put {
                key: key_2.as_bytes().to_vec().into_boxed_slice(),
                value: value_2.as_bytes().to_vec().into_boxed_slice(),
            }),
            BatchOperation::del(crate::traits::Del {
                key: key_2.as_bytes().to_vec().into_boxed_slice(),
            }),
        ];
        assert!(
            kv_storage.batch(batch_data, true).await.is_ok(),
            "Test #5.0 failed: batch operation failed"
        );

        // ===================================

        async_std::task::sleep(std::time::Duration::from_millis(10)).await;

        assert!(
            kv_storage.contains(key_3.as_bytes().to_vec().into_boxed_slice()).await,
            "Test #5.1 failed: the key should be present in the DB"
        );

        // ===================================

        let _ = kv_storage
            .set(
                prefixed_key_1.as_bytes().to_vec().into_boxed_slice(),
                value_1.as_bytes().to_vec().into_boxed_slice(),
            )
            .await;
        let _ = kv_storage
            .set(
                prefixed_key_2.as_bytes().to_vec().into_boxed_slice(),
                value_2.as_bytes().to_vec().into_boxed_slice(),
            )
            .await;
        let _ = kv_storage
            .set(
                prefixed_key_3.as_bytes().to_vec().into_boxed_slice(),
                value_3.as_bytes().to_vec().into_boxed_slice(),
            )
            .await;

        let expected = vec![
            value_1.as_bytes().to_vec().into_boxed_slice(),
            value_3.as_bytes().to_vec().into_boxed_slice(),
        ];

        let mut received = Vec::new();
        let mut data_stream = Box::into_pin(
            kv_storage
                .iterate(
                    prefix.as_bytes().to_vec().into_boxed_slice(),
                    (prefixed_key_1.len() - prefix.len()) as u32,
                )
                .unwrap(),
        );

        while let Some(value) = data_stream.next().await {
            let v = value.unwrap();

            if v.as_ref() != value_2.as_bytes() {
                received.push(v);
            }
        }
        assert_eq!(received, expected, "Test #6 failed: db content mismatch");
    }
}
