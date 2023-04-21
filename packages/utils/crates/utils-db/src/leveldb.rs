use async_trait::async_trait;

use crate::errors::DbError;
use crate::traits::AsyncKVStorage;


#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// https://users.rust-lang.org/t/wasm-web-sys-how-to-manipulate-js-objects-from-rust/36504
#[cfg(feature = "wasm")]
#[wasm_bindgen]
extern "C" {
    pub type LevelDb;

    #[wasm_bindgen(method, catch)]
    async fn has(this: &LevelDb, key: Box<[u8]>) -> Result<JsValue, JsValue>;     // bool;

    #[wasm_bindgen(method, catch)]
    async fn put(this: &LevelDb, key: Box<[u8]>, value: Box<[u8]>) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch)]
    async fn get(this: &LevelDb, key: Box<[u8]>) -> Result<JsValue, JsValue>;          // Option<Box<[u8]>>;

    #[wasm_bindgen(method, catch)]
    async fn remove(this: &LevelDb, key: Box<[u8]>) -> Result<JsValue, JsValue>;       // Option<Box<[u8]>>;

    // https://github.com/Level/levelup#dbbatcharray-options-callback-array-form
    // #[wasm_bindgen(method)]
    // async fn batch(this: &LevelDb, operations: Vec<String>);

    #[wasm_bindgen(method, catch)]
    fn dump(this: &LevelDb, destination: String) -> Result<(), JsValue>;
}

pub struct LevelDbShim
{
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
            .get(key).await
            .map_err(|_| DbError::GenericError("Encountered error on DB get operation".to_string()))
            .and_then(|v| {
                if v.is_undefined() {
                    Err(DbError::NotFound)
                } else {
                    Ok(js_sys::Uint8Array::new(&v).to_vec().into_boxed_slice())
                }
            })
    }

    async fn set(&mut self, key: Self::Key, value: Self::Value) -> crate::errors::Result<Option<Self::Value>> {
        self.db
            .put(key, value).await
            .map(|v| None)              // NOTE: The LevelDB API does not allow to return an evicted value
            .map_err(|_| DbError::GenericError("Encountered error on DB put operation".to_string()))
    }

    async fn contains(&self, key: Self::Key) -> bool {
        self.db
            .has(key).await
            .map(|v| v.as_bool().unwrap())
            .unwrap_or(false)
    }

    async fn remove(&mut self, key: Self::Key) -> crate::errors::Result<Option<Self::Value>> {
        self.db
            .remove(key).await
            .map(|v| {
                if v.is_undefined() {
                    // NOTE: The LevelDB API does not allow to return an evicted value
                    None
                } else {
                    Some(js_sys::Uint8Array::new(&v).to_vec().into_boxed_slice())
                }
            })
            .map_err(|_| DbError::GenericError("Encountered error on DB remove operation".to_string()))
    }

    async fn dump(&self, destination: String) -> crate::errors::Result<()> {
        self.db
            .dump(destination.clone())
            .map_err(|_| DbError::DumpError(format!("Failed to dump DB into {}", destination)))
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn db_sanity_test(db: LevelDb) -> Result<bool, JsValue> {
    let key_1 = "1";
    let value_1 = "abc";
    // let key_2 = "2";
    // let value_2 = "def";

    let mut kv_storage = LevelDbShim::new(db);
    if kv_storage.contains(key_1.as_bytes().to_vec().into_boxed_slice()).await {
        return Err::<bool, JsValue>(
            JsValue::from(JsError::new("Test #1 failed: empty DB should not contain any data")))
    }

    kv_storage.set(key_1.as_bytes().to_vec().into_boxed_slice(), value_1.as_bytes().to_vec().into_boxed_slice()).await;
    if !kv_storage.contains(key_1.as_bytes().to_vec().into_boxed_slice()).await {
        return Err::<bool, JsValue>(
            JsValue::from(JsError::new("Test #2 failed: DB should contain the key")))
    }

    let value = kv_storage.get(key_1.as_bytes().to_vec().into_boxed_slice()).await.unwrap();
    let value_converted = std::str::from_utf8(value.as_ref())
        .map_err(|_| JsValue::from(JsError::new("Test #3.0 failed: could not convert the get type")))?;

    if value_converted != value_1 {
        return Err::<bool, JsValue>(
            JsValue::from(JsError::new("Test #3.1 failed: DB value after get should be equal to the one before the get")))
    }

    kv_storage.remove(key_1.as_bytes().to_vec().into_boxed_slice());

    if !kv_storage.contains(key_1.as_bytes().to_vec().into_boxed_slice()).await {
        return Err::<bool, JsValue>(
            JsValue::from(JsError::new("Test #4 failed: removal of key from the DB failed")))
    }

    Ok(true)
}