#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::errors::DbError;
    use crate::traits::{BatchOperation, KVStorage};
    use async_trait::async_trait;
    use wasm_bindgen::prelude::*;

    // https://users.rust-lang.org/t/wasm-web-sys-how-to-manipulate-js-objects-from-rust/36504
    #[wasm_bindgen]
    extern "C" {
        pub type Sqlite;

        #[wasm_bindgen(method, catch)]
        fn put(this: &Sqlite, key: Box<[u8]>, value: Box<[u8]>) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        fn get(this: &Sqlite, key: Box<[u8]>) -> Result<JsValue, JsValue>; // Option<Box<[u8]>>;

        #[wasm_bindgen(method, catch)]
        fn remove(this: &Sqlite, key: Box<[u8]>) -> Result<JsValue, JsValue>; // Option<Box<[u8]>>;

        #[wasm_bindgen(method, catch)]
        fn batch(this: &Sqlite, operations: Vec<JsValue>) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        fn iterValues(this: &Sqlite, prefix: js_sys::Uint8Array, suffix_length: u32) -> Result<Vec<JsValue>, JsValue>;
    }

    pub struct SqliteShim {
        db: Sqlite,
    }

    impl SqliteShim {
        pub fn new(db: Sqlite) -> SqliteShim {
            SqliteShim { db }
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
    impl KVStorage for SqliteShim {
        type Key = Box<[u8]>;
        type Value = Box<[u8]>;

        async fn get(&self, key: Self::Key) -> crate::errors::Result<Option<Self::Value>> {
            match self.db.get(key.clone()) {
                Ok(val) => Ok(if val.is_undefined() {
                    None
                } else {
                    let arr = js_sys::Uint8Array::from(val).to_vec().into_boxed_slice();
                    Some(arr)
                }),
                Err(e) => Err(DbError::GenericError(
                    "Encountered error on DB get operation".to_string(),
                )),
            }
        }

        async fn set(&mut self, key: Self::Key, value: Self::Value) -> crate::errors::Result<Option<Self::Value>> {
            self.db
                .put(key.clone(), value)
                .map(|_| None)
                .map_err(|_| DbError::GenericError("Encountered error on DB put operation".to_string()))
        }

        async fn contains(&self, key: Self::Key) -> crate::errors::Result<bool> {
            match self.get(key).await {
                Ok(Some(_)) => Ok(true),
                Ok(None) => Ok(false),
                Err(e) => Err(e),
            }
        }

        async fn remove(&mut self, key: Self::Key) -> crate::errors::Result<Option<Self::Value>> {
            self.db
                .remove(key.clone())
                .map(|v| {
                    if v.is_undefined() {
                        None
                    } else {
                        Some(js_sys::Uint8Array::from(v).to_vec().into_boxed_slice())
                    }
                })
                .map_err(|_| DbError::GenericError("Encountered error on DB remove operation".to_string()))
        }

        async fn iterate(&self, prefix: Self::Key, suffix_size: u32) -> crate::errors::Result<Vec<Self::Value>> {
            return self
                .db
                .iterValues(js_sys::Uint8Array::from(prefix.as_ref()), suffix_size)
                .map(|v| {
                    v.into_iter()
                        .map(|e| js_sys::Uint8Array::from(e).to_vec().into_boxed_slice())
                        .collect()
                })
                .map_err(|e| DbError::GenericError(format!("Iteration failed with an exception: {:?}", e)));
        }

        async fn batch(
            &mut self,
            operations: Vec<BatchOperation<Self::Key, Self::Value>>,
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
                .batch(ops)
                .map_err(|e| DbError::GenericError(format!("Batch operation failed to write data: {:?}", e)))
        }
    }

    #[wasm_bindgen]
    pub async fn db_sanity_test(db: Sqlite) -> Result<(), JsValue> {
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

        let mut kv_storage = SqliteShim::new(db);

        // if let Err(e) = kv_storage.dump("/tmp/level.db".to_owned()).await {
        //     return Err::<bool, JsValue>(
        //         JsValue::from(JsError::new(format!("Test #0 failed: {:?}", e).as_str())))
        // }

        if kv_storage.contains(key_1.as_bytes().into()).await.unwrap() {
            return Err("Test #1 failed: empty DB should not contain any data"
                .to_string()
                .into());
        }

        // ===================================

        let _ = kv_storage.set(key_1.as_bytes().into(), value_1.as_bytes().into());
        if !kv_storage.contains(key_1.as_bytes().into()).await.unwrap() {
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

        let _ = kv_storage.remove(key_1.as_bytes().into());

        if kv_storage.contains(key_1.as_bytes().into()).await.unwrap() {
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
        if let Err(e) = kv_storage.batch(batch_data).await {
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

        if !kv_storage.contains(key_4.as_bytes().into()).await.unwrap() {
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
        let data = kv_storage
            .iterate(prefix.as_bytes().into(), (prefixed_key_1.len() - prefix.len()) as u32)
            .await
            .map_err(|e| JsValue::from(format!("Test #7.1 failed: failed to iterate over DB {:?}", e)))?;

        for value in data.into_iter() {
            let v = js_sys::Uint8Array::from(value.as_ref()).to_vec().into_boxed_slice();
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
