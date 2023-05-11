use async_trait::async_trait;
use js_sys::Uint8Array;

use futures_lite::stream::{Stream,StreamExt};
use crate::errors::DbError;
use crate::traits::{AsyncKVStorage, BatchOperation};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// https://users.rust-lang.org/t/wasm-web-sys-how-to-manipulate-js-objects-from-rust/36504
#[cfg(feature = "wasm")]
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

    async fn dump(&self, destination: String) -> crate::errors::Result<()> {
        self.db
            .dump(destination.clone())
            .map_err(|_| DbError::DumpError(format!("Failed to dump DB into {}", destination)))
    }

    // async fn iterate(&self, prefix: Self::Key, suffix_size: u32) -> crate::errors::Result<Box<dyn Stream>> {
    //     let iterable = self.db
    //         .iterValues(js_sys::Uint8Array::from(prefix.as_ref()), suffix_size)
    //         .map(|v| js_sys::AsyncIterator::from(v))
    //         .map_err(|e| DbError::GenericError(format!("Iteration failed with an exception: {:?}", e)))?;
    //
    //     let mut out = Vec::with_capacity(10);
    //
    //     let mut stream = wasm_bindgen_futures::stream::JsStream::from(iterable);
    //
    //     while let Some(value) = stream.next().await {
    //         let v: Self::Value = value.map(|v| js_sys::Uint8Array::from(v).to_vec().into_boxed_slice())
    //             .map_err(|e| DbError::GenericError(format!("Failed to poll next value: {:?}", e)))?;
    //
    //         if (*filter)(v.clone()) {
    //             out.push(v);
    //         }
    //     }
    //
    //     Ok(out)
    // }

    async fn get_more(&self, prefix: Self::Key, suffix_size: u32, filter: Box<dyn Fn(Self::Key) -> bool>) -> crate::errors::Result<Vec<Self::Value>> {
        let iterable = self.db
            .iterValues(js_sys::Uint8Array::from(prefix.as_ref()), suffix_size)
            .map(|v| js_sys::AsyncIterator::from(v))
            .map_err(|e| DbError::GenericError(format!("Iteration failed with an exception: {:?}", e)))?;

        let mut out = Vec::with_capacity(10);

        let mut stream = wasm_bindgen_futures::stream::JsStream::from(iterable);

        while let Some(value) = stream.next().await {
            let v: Self::Value = value.map(|v| js_sys::Uint8Array::from(v).to_vec().into_boxed_slice())
                .map_err(|e| DbError::GenericError(format!("Failed to poll next value: {:?}", e)))?;

            if (*filter)(v.clone()) {
                out.push(v);
            }
        }

        Ok(out)
    }
}

#[cfg(feature = "wasm")]
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
    // if !kv_storage.contains(key_3.as_bytes().to_vec().into_boxed_slice()).await {
    //     return Err::<bool, JsValue>(
    //         JsValue::from(JsError::new("Test #5.1 failed: the key should be present in the DB")))
    // }

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

    let received = kv_storage.get_more(
        prefix.as_bytes().to_vec().into_boxed_slice(),
        (prefixed_key_1.len() - prefix.len()) as u32,
        Box::new(|x| x.as_ref() != value_2.as_bytes())
    ).await;

    let expected = Ok(vec![
        value_1.as_bytes().to_vec().into_boxed_slice(),
        // value_2.as_bytes().to_vec().into_boxed_slice(),
        value_3.as_bytes().to_vec().into_boxed_slice()
    ]);

    if received != expected {
        return Err::<bool, JsValue>(
            JsValue::from(JsError::new(format!("Test #6 failed: db content mismatch {:?} != {:?}", received, expected).as_str())))
    }

    Ok(true)
}
