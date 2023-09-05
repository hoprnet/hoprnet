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
        async fn put(this: &LevelDb, key: Box<[u8]>, value: Box<[u8]>) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        async fn get(this: &LevelDb, key: Box<[u8]>) -> Result<JsValue, LevelDBGetError>; // Option<Box<[u8]>>;

        #[wasm_bindgen(method, catch)]
        async fn remove(this: &LevelDb, key: Box<[u8]>) -> Result<JsValue, JsValue>; // Option<Box<[u8]>>;

        // https://github.com/Level/levelup#dbbatcharray-options-callback-array-form
        #[wasm_bindgen(method, catch)]
        async fn batch(this: &LevelDb, operations: Vec<JsValue>, wait_for_write: bool) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        fn iterValues(this: &LevelDb, prefix: js_sys::Uint8Array, suffix_length: u32) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(method, catch)]
        async fn dump(this: &LevelDb, destination: String) -> Result<(), JsValue>;
    }

    #[wasm_bindgen]
    extern "C" {
        pub type LevelDBGetError;

        #[wasm_bindgen(js_name = "notFound", getter, method)]
        fn not_found(this: &LevelDBGetError) -> bool;

        #[wasm_bindgen(js_name = "toString", method)]
        fn to_string(this: &LevelDBGetError) -> String;
    }

    pub struct LevelDbShim {
        db: LevelDb,
    }

    impl LevelDbShim {
        pub fn new(db: LevelDb) -> LevelDbShim {
            LevelDbShim { db }
        }
    }

    #[wasm_bindgen(getter_with_clone)]
    pub struct BatchOp {
        #[wasm_bindgen(js_name = "type")]
        pub kind: String,
        pub key: Box<[u8]>,
        pub value: Option<Box<[u8]>>,
    }

    #[async_trait(? Send)]
    impl AsyncKVStorage for LevelDbShim {
        type Key = Box<[u8]>;
        type Value = Box<[u8]>;

        async fn get(&self, key: Self::Key) -> crate::errors::Result<Option<Self::Value>> {
            match self.db.get(key).await {
                Ok(val) => Ok(if val.is_undefined() {
                    None
                } else {
                    let arr = js_sys::Uint8Array::from(val).to_vec().into_boxed_slice();
                    if arr.len() > 0 {
                        Some(arr)
                    } else {
                        None
                    }
                }),
                Err(e) => {
                    if e.not_found() {
                        Ok(None)
                    } else {
                        Err(DbError::GenericError(format!(
                            "Encountered error on DB get operation {}",
                            e.to_string()
                        )))
                    }
                }
            }
        }

        async fn set(&mut self, key: Self::Key, value: Self::Value) -> crate::errors::Result<Option<Self::Value>> {
            self.db
                .put(key, value)
                .await
                .map(|_| None) // NOTE: The LevelDB API does not allow to return an evicted value
                .map_err(|_| DbError::GenericError("Encountered error on DB put operation".to_string()))
        }

        async fn contains(&self, key: Self::Key) -> crate::errors::Result<bool> {
            match self.db.get(key).await {
                Ok(_) => Ok(true),
                Err(e) => {
                    if e.not_found() {
                        Ok(false)
                    } else {
                        Err(DbError::GenericError(format!(
                            "Encountered error on DB get operation {}",
                            e.to_string()
                        )))
                    }
                }
            }
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
                .await
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
                .map(js_sys::AsyncIterator::from)
                .map_err(|e| DbError::GenericError(format!("Iteration failed with an exception: {:?}", e)))?;

            let stream = wasm_bindgen_futures::stream::JsStream::from(iterable);

            Ok(Box::new(crate::types::wasm::BinaryStreamWrapper::new(stream)))
        }

        async fn batch(
            &mut self,
            operations: Vec<BatchOperation<Self::Key, Self::Value>>,
            wait_for_write: bool,
        ) -> crate::errors::Result<()> {
            let ops = operations
                .into_iter()
                .map(|op| match op {
                    BatchOperation::del(key) => JsValue::from(BatchOp {
                        kind: "del".into(),
                        key: key.key,
                        value: None,
                    }),
                    BatchOperation::put(put) => JsValue::from(BatchOp {
                        kind: "put".into(),
                        key: put.key,
                        value: Some(put.value),
                    }),
                })
                .collect::<Vec<_>>();

            self.db
                .batch(ops, wait_for_write)
                .await
                .map_err(|e| DbError::GenericError(format!("Batch operation failed to write data: {:?}", e)))
        }
    }

    #[wasm_bindgen]
    pub async fn db_sanity_test(db: LevelDb) -> Result<(), JsValue> {
        let key_1 = "1";
        let value_1 = "abc";
        let key_2 = "2";
        let value_2 = "def";
        let key_3 = "3";
        let value_3 = "ghi";
        let prefix = "xy";
        let key_4 = "4";
        let prefixed_key_1 = "xya";
        let prefixed_key_2 = "xyb";
        let prefixed_key_3 = "xyc";

        let mut kv_storage = LevelDbShim::new(db);

        // if let Err(e) = kv_storage.dump("/tmp/level.db".to_owned()).await {
        //     return Err::<bool, JsValue>(
        //         JsValue::from(JsError::new(format!("Test #0 failed: {:?}", e).as_str())))
        // }

        if kv_storage.contains(key_1.as_bytes().into()).await? {
            return Err("Test #1 failed: empty DB should not contain any data"
                .to_string()
                .into());
        }

        // ===================================

        let _ = kv_storage.set(key_1.as_bytes().into(), value_1.as_bytes().into()).await;
        if !kv_storage.contains(key_1.as_bytes().into()).await? {
            return Err("Test #2 failed: DB should contain the key".to_string().into());
        }

        // ===================================

        let value = kv_storage
            .get(key_1.as_bytes().into())
            .await
            .unwrap()
            .expect("Stored empty value");
        let value_converted = std::str::from_utf8(value.as_ref())
            .map_err(|_| JsValue::from(JsError::new("Test #3.0 failed: could not convert the get type")))?;

        if value_converted != value_1 {
            return Err(
                "Test #3.1 failed: DB value after get should be equal to the one before the get"
                    .to_string()
                    .into(),
            );
        }

        // ===================================

        let _ = kv_storage.remove(key_1.as_bytes().into()).await;

        if kv_storage.contains(key_1.as_bytes().into()).await? {
            return Err("Test #4 failed: removal of key from the DB failed".to_string().into());
        }

        // ===================================

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
        if let Err(e) = kv_storage.batch(batch_data, true).await {
            return Err(format!("Test #5.0 failed: batch operation failed: {e}").into());
        }

        // ===================================

        gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;

        // TODO: levelup api with the passed options to do an immediate write does not perform an immediate write
        if !kv_storage.contains(key_3.as_bytes().into()).await? {
            return Err("Test #5.1 failed: the key should be present in the DB"
                .to_string()
                .into());
        }

        // ===================================

        kv_storage
            .set(key_4.as_bytes().into(), Box::new([]))
            .await
            .expect("Could not write empty value");

        if !kv_storage.contains(key_4.as_bytes().into()).await? {
            return Err("Test #5.2 failed: it should be possible to store empty values"
                .to_string()
                .into());
        }

        if !kv_storage.get(key_4.as_bytes().into()).await.is_ok_and(|o| o.is_none()) {
            return Err("Test #6 failed: could not read empty value from DB".to_string().into());
        }

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
                .map_err(|e| JsValue::from(format!("Test #7.1 failed: failed to iterate over DB {:?}", e)))?,
        );

        while let Some(value) = data_stream.next().await {
            let v = value
                .map(|v| js_sys::Uint8Array::from(v.as_ref()).to_vec().into_boxed_slice())
                .map_err(|e| JsValue::from(format!("Test #7.1 failed: failed to iterate over DB {:?}", e)))?;

            if v.as_ref() != value_2.as_bytes() {
                received.push(v);
            }
        }

        if received != expected {
            return Err(format!("Test #7.2 failed: db content mismatch {:?} != {:?}", received, expected).into());
        }

        Ok(())
    }
}
