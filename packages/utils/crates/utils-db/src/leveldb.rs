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

    // #[wasm_bindgen(constructor)]
    // fn new(public_key: String) -> WrappedLevelDb;

    #[wasm_bindgen(method, catch)]
    async fn init(this: &LevelDb, initialize: bool, dbPath: String, force_create: bool, environment_id: String) -> Result<(), JsValue>;

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

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn db_sanity_test(db: LevelDb) -> Result<bool, JsValue> {
    let key_1 = "1";
    let value_1 = "abc";
    // let key_2 = "2";
    // let value_2 = "def";

    let mut kv_storage = LevelDB::new(db);
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

pub struct LevelDB
{
    db: LevelDb,
}

impl LevelDB {
    pub fn new(db: LevelDb) -> LevelDB {
        LevelDB { db }
    }
}

#[async_trait(? Send)]
impl AsyncKVStorage for LevelDB {
    type Key = Box<[u8]>;
    type Value = Box<[u8]>;

    async fn get(&self, key: Self::Key) -> Option<Self::Value> {
        match self.db.get(key).await {
            Ok(v) => {
                if v.is_undefined() {
                    None
                } else {
                    Some(js_sys::Uint8Array::new(&v).to_vec().into_boxed_slice())
                }
            }
            Err(_) => {
                None
            }
        }
    }

    async fn set(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value> {
        let _ = self.db.put(key, value).await;

        // NOTE: The LevelDB API does not allow to return an evicted value
        None
    }

    async fn contains(&self, key: Self::Key) -> bool {
        match self.db.has(key).await {
            Ok(v) => v.as_bool().unwrap(),
            Err(_) => false,
        }
    }

    async fn remove(&mut self, key: Self::Key) -> Option<Self::Value> {
        let _ = self.db.remove(key).await;
        // NOTE: The LevelDB API does not allow to return an evicted value
        None
    }

    async fn dump(&self, destination: String) -> crate::errors::Result<()> {
        self.db.dump(destination.clone())
            .map_err(|_| DbError::DumpError(format!("Failed to dump DB into {}", destination)))
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_leveldb() {
        assert!(true);
    }
}