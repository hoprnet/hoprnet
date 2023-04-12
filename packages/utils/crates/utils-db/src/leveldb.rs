use crate::traits::AsyncKVStorage;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// https://users.rust-lang.org/t/wasm-web-sys-how-to-manipulate-js-objects-from-rust/36504
#[cfg(feature = "wasm")]
#[wasm_bindgen]
extern "C" {
    type LevelDb;

    #[wasm_bindgen(constructor)]
    fn new(public_key: String) -> LevelDb;

    #[wasm_bindgen]
    async fn init(this: &LevelDb, initialize: bool, dbPath: String, force_create: bool, environment_id: String);

    #[wasm_bindgen(method)]
    async fn has(this: &LevelDb, key: Box<[u8]>) -> JsValue;     // bool;

    #[wasm_bindgen(method)]
    async fn put(this: &LevelDb, key: Box<[u8]>, value: Box<[u8]>);

    #[wasm_bindgen(method)]
    async fn get(this: &LevelDb, key: Box<[u8]>) -> JsValue;          // Option<Box<[u8]>>;

    #[wasm_bindgen(method)]
    async fn remove(this: &LevelDb, key: Box<[u8]>) -> JsValue;       // Option<Box<[u8]>>;

    // https://github.com/Level/levelup#dbbatcharray-options-callback-array-form
    // #[wasm_bindgen(method)]
    // async fn batch(this: &LevelDb, operations: Vec<String>);

    #[wasm_bindgen(method)]
    fn dump(this: &LevelDb, destination: String);
}

pub struct LevelDB<K, V>
    where
        K: std::cmp::Eq + std::hash::Hash,
        V: Clone,
{
    data: std::collections::hash_map::HashMap<K, V>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leveldb() {
        assert!(true);
    }
}