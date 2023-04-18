use std::fmt::{Error, Pointer};
use async_trait::async_trait;

use crate::errors::DbError;
use crate::traits::AsyncKVStorage;


#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// https://users.rust-lang.org/t/wasm-web-sys-how-to-manipulate-js-objects-from-rust/36504
#[cfg(feature = "wasm")]
#[wasm_bindgen]
extern "C" {
    type WrappedLevelDb;

    #[wasm_bindgen(constructor)]
    fn new(public_key: String) -> WrappedLevelDb;

    #[wasm_bindgen(method,catch)]
    async fn init(this: &WrappedLevelDb, initialize: bool, dbPath: String, force_create: bool, environment_id: String) -> Result<(), JsValue>;

    #[wasm_bindgen(method,catch)]
    async fn has(this: &WrappedLevelDb, key: Box<[u8]>) -> Result<JsValue, JsValue>;     // bool;

    #[wasm_bindgen(method,catch)]
    async fn put(this: &WrappedLevelDb, key: Box<[u8]>, value: Box<[u8]>) -> Result<(), JsValue>;

    #[wasm_bindgen(method,catch)]
    async fn get(this: &WrappedLevelDb, key: Box<[u8]>) -> Result<JsValue, JsValue>;          // Option<Box<[u8]>>;

    #[wasm_bindgen(method,catch)]
    async fn remove(this: &WrappedLevelDb, key: Box<[u8]>) -> Result<JsValue, JsValue>;       // Option<Box<[u8]>>;

    // https://github.com/Level/levelup#dbbatcharray-options-callback-array-form
    // #[wasm_bindgen(method)]
    // async fn batch(this: &LevelDb, operations: Vec<String>);

    #[wasm_bindgen(method,catch)]
    fn dump(this: &WrappedLevelDb, destination: String) -> Result<(), JsValue>;
}

pub struct LevelDB
{
    db: WrappedLevelDb,
}

#[async_trait(?Send)]
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
        self.db.put(key, value).await;

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
        self.db.remove(key).await;
        // NOTE: The LevelDB API does not allow to return an evicted value
        None
    }

    async fn dump(&self, destination: String) -> crate::errors::Result<()> {
        self.db.dump(destination.clone())
            .map_err(|e| DbError::DumpError(format!("Failed to dump DB into {}", destination)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leveldb() {
        assert!(true);
    }
}