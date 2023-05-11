use serde::{de::DeserializeOwned, Serialize};
use std::ops::Deref;

use crate::errors::{DbError, Result};
use crate::traits::BinaryAsyncKVStorage;

pub struct Batch {
    pub ops: Vec<crate::traits::BatchOperation<Box<[u8]>, Box<[u8]>>>,
}

pub fn serialize_to_bytes<S: Serialize>(s: &S) -> Result<Vec<u8>> {
    bincode::serialize(&s).map_err(|e| DbError::SerializationError(e.to_string()))
}

impl Batch {
    pub fn new() -> Self {
        Self {
            ops: Vec::with_capacity(10),
        }
    }

    pub fn put<T: Serialize, U: Serialize>(&mut self, key: T, value: U) {
        let key: Box<[u8]> = bincode::serialize(&key).unwrap().into_boxed_slice();
        let value: Box<[u8]> = bincode::serialize(&value).unwrap().into_boxed_slice();

        self.ops
            .push(crate::traits::BatchOperation::put(crate::traits::Put { key, value }));
    }

    pub fn del<T: Serialize>(&mut self, key: T) {
        let key: Box<[u8]> = bincode::serialize(&key).unwrap().into_boxed_slice();

        self.ops
            .push(crate::traits::BatchOperation::del(crate::traits::Del { key }));
    }
}

#[derive(Debug, Clone)]
pub struct Key {
    key: Box<[u8]>,
}

impl Key {
    pub fn new<T: Serialize>(object: &T) -> Result<Self> {
        let key = bincode::serialize(&object)
            .map_err(|e| DbError::SerializationError(e.to_string()))?
            .into_boxed_slice();
        Ok(Self { key })
    }

    pub fn new_from_str(object: &str) -> Result<Self> {
        let key = bincode::serialize(&object)
            .map_err(|e| DbError::SerializationError(e.to_string()))?
            .into_boxed_slice();
        Ok(Self { key })
    }

    pub fn new_with_prefix<T: Serialize>(object: &T, prefix: &str) -> Result<Self> {
        let key = bincode::serialize(&object)
            .map_err(|e| DbError::SerializationError(e.to_string()))?
            .into_boxed_slice();

        let mut result = Vec::with_capacity(prefix.len() + key.as_ref().len());
        result.extend_from_slice(prefix.as_bytes().as_ref());
        result.extend_from_slice(key.as_ref());

        Ok(Self {
            key: result.into_boxed_slice(),
        })
    }
}

impl Into<Box<[u8]>> for Key {
    fn into(self) -> Box<[u8]> {
        self.key
    }
}

impl Deref for Key {
    type Target = Box<[u8]>;

    fn deref(&self) -> &Self::Target {
        &self.key
    }
}

pub struct DB<T: BinaryAsyncKVStorage> {
    backend: T,
}

impl<T: BinaryAsyncKVStorage> DB<T> {
    pub fn new(backend: T) -> Self {
        DB::<T> { backend }
    }

    pub async fn contains(&self, key: Key) -> bool {
        self.backend.contains(key.into()).await
    }

    pub async fn get<V: DeserializeOwned>(&self, key: Key) -> Result<V> {
        let key: T::Key = key.into();
        self.backend
            .get(key)
            .await
            .and_then(|v| bincode::deserialize(v.as_ref()).map_err(|e| DbError::DeserializationError(e.to_string())))
    }

    pub async fn set<V>(&mut self, key: Key, value: &V) -> Result<Option<V>>
        where
            V: Serialize + DeserializeOwned,
    {
        let key: T::Key = key.into();
        let value: T::Value = bincode::serialize(&value)
            .map_err(|e| DbError::SerializationError(e.to_string()))?
            .into_boxed_slice();

        match self.backend.set(key, value).await? {
            Some(v) => bincode::deserialize(v.as_ref())
                .map(|v| Some(v))
                .map_err(|e| DbError::DeserializationError(e.to_string())),
            None => Ok(None),
        }
    }

    pub async fn remove<V: DeserializeOwned>(&mut self, key: Key) -> Result<Option<V>> {
        let key: T::Key = key.into();
        match self.backend.remove(key).await? {
            Some(v) => bincode::deserialize(v.as_ref())
                .map(|v| Some(v))
                .map_err(|e| DbError::DeserializationError(e.to_string())),
            None => Ok(None),
        }
    }

    // async fn get_more<V: Serialize + DeserializeOwned>(&self, prefix: Box<[u8]>, suffix_size: u32, filter: Box<dyn Fn(&V) -> bool>) -> Result<Vec<V>> {
    //     let data = self.backend.get_more(prefix, suffix_size, Box::new(move |v| {
    //         let value = bincode::deserialize::<V>(v.as_ref())
    //             .map_err(|e| DbError::DeserializationError(e.to_string()));
    //         if value.is_ok() {
    //             (*filter)(&value.unwrap())
    //         } else {
    //             utils_log::error!("Error deserializing in iteration over keys: {}", value.err().unwrap().to_string());
    //             false
    //         }
    //     })).await?;
    //
    //     data.into_iter()
    //         .map(|v| bincode::deserialize::<V>(v.as_ref()))
    // }

    pub async fn batch(&mut self, batch: Batch, wait_for_write: bool) -> Result<()> {
        self.backend.batch(batch.ops, wait_for_write).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::DbError;
    use crate::traits::MockAsyncKVStorage;
    use mockall::predicate;
    use serde::Deserialize;

    impl BinaryAsyncKVStorage for MockAsyncKVStorage {}

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestKey {
        v: u16,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestValue {
        v: String,
    }

    #[async_std::test]
    async fn test_db_contains_serializes_correctly() {
        let key = TestKey { v: 1 };

        let expected = bincode::serialize(&key).unwrap().into_boxed_slice();

        let mut backend = MockAsyncKVStorage::new();
        backend
            .expect_contains()
            .with(predicate::eq(expected.clone()))
            .return_const(true);

        let db = DB::new(backend);

        assert!(db.contains(Key::new(&key).ok().unwrap()).await)
    }

    #[async_std::test]
    async fn test_db_get_serializes_correctly_and_succeeds_if_a_value_is_available() {
        let key = TestKey { v: 1 };
        let value = TestValue { v: "value".to_string() };

        let expected_key = bincode::serialize(&key).unwrap().into_boxed_slice();
        let ser_value: Result<Box<[u8]>> = Ok(bincode::serialize(&value).unwrap().into_boxed_slice());

        let mut backend = MockAsyncKVStorage::new();
        backend
            .expect_get()
            .with(predicate::eq(expected_key.clone()))
            .return_once(move |_| ser_value);

        let db = DB::new(backend);

        assert_eq!(db.get::<TestValue>(Key::new(&key).ok().unwrap()).await, Ok(value))
    }

    #[async_std::test]
    async fn test_db_get_serializes_correctly_and_fails_if_a_value_is_unavailable() {
        let key = TestKey { v: 1 };

        let expected_key = bincode::serialize(&key).unwrap().into_boxed_slice();

        let mut backend = MockAsyncKVStorage::new();
        backend
            .expect_get()
            .with(predicate::eq(expected_key.clone()))
            .return_once(|_| -> Result<Box<[u8]>> { Err(DbError::NotFound) });

        let db = DB::new(backend);

        assert_eq!(
            db.get::<TestValue>(Key::new(&key).ok().unwrap()).await,
            Err(DbError::NotFound)
        )
    }

    #[async_std::test]
    async fn test_db_set_serializes_correctly_and_sets_the_value() {
        let key = TestKey { v: 1 };
        let value = TestValue { v: "value".to_string() };

        let expected_key = bincode::serialize(&key).unwrap().into_boxed_slice();
        let expected_value = bincode::serialize(&value).unwrap().into_boxed_slice();

        let mut backend = MockAsyncKVStorage::new();
        backend
            .expect_set()
            .with(
                predicate::eq(expected_key.clone()),
                predicate::eq(expected_value.clone()),
            )
            .return_once(|_, _| Ok(None));

        let mut db = DB::new(backend);

        assert_eq!(db.set(Key::new(&key).ok().unwrap(), &value).await, Ok(None))
    }

    #[async_std::test]
    async fn test_db_set_serializes_correctly_and_fails_if_a_value_is_unavailable() {
        let key = TestKey { v: 1 };
        let value = TestValue { v: "value".to_string() };

        let expected_key = bincode::serialize(&key).unwrap().into_boxed_slice();
        let expected_value = bincode::serialize(&value).unwrap().into_boxed_slice();

        let mut backend = MockAsyncKVStorage::new();
        backend
            .expect_set()
            .with(
                predicate::eq(expected_key.clone()),
                predicate::eq(expected_value.clone()),
            )
            .return_once(|_, _| Err(DbError::NotFound));

        let mut db = DB::new(backend);

        assert_eq!(
            db.set(Key::new(&key).ok().unwrap(), &value).await,
            Err(DbError::NotFound)
        )
    }

    #[async_std::test]
    async fn test_db_set_serializes_correctly_and_returns_evicted_value_if_it_was_available() {
        let key = TestKey { v: 1 };
        let value = TestValue { v: "value".to_string() };
        let evicted = TestValue {
            v: "evicted".to_string(),
        };

        let expected_key = bincode::serialize(&key).unwrap().into_boxed_slice();
        let expected_value = bincode::serialize(&value).unwrap().into_boxed_slice();
        let evicted_value = bincode::serialize(&evicted).unwrap().into_boxed_slice();

        let mut backend = MockAsyncKVStorage::new();
        backend
            .expect_set()
            .with(
                predicate::eq(expected_key.clone()),
                predicate::eq(expected_value.clone()),
            )
            .return_once(move |_, _| Ok(Some(evicted_value)));

        let mut db = DB::new(backend);

        assert_eq!(db.set(Key::new(&key).ok().unwrap(), &value).await, Ok(Some(evicted)))
    }

    #[async_std::test]
    async fn test_db_remove_serializes_correctly_and_succeeds_without_evictions() {
        let key = TestKey { v: 1 };

        let expected_key = bincode::serialize(&key).unwrap().into_boxed_slice();

        let mut backend = MockAsyncKVStorage::new();
        backend
            .expect_remove()
            .with(predicate::eq(expected_key.clone()))
            .return_once(move |_| Ok(None));

        let mut db = DB::new(backend);

        assert_eq!(db.remove::<TestValue>(Key::new(&key).ok().unwrap()).await, Ok(None))
    }

    #[async_std::test]
    async fn test_db_remove_serializes_correctly_and_fails_if_the_underlying_layer_fails() {
        let key = TestKey { v: 1 };

        let expected_key = bincode::serialize(&key).unwrap().into_boxed_slice();

        let mut backend = MockAsyncKVStorage::new();
        backend
            .expect_remove()
            .with(predicate::eq(expected_key.clone()))
            .return_once(move |_| Err(DbError::NotFound));

        let mut db = DB::new(backend);

        assert_eq!(
            db.remove::<TestValue>(Key::new(&key).ok().unwrap()).await,
            Err(DbError::NotFound)
        )
    }

    #[async_std::test]
    async fn test_db_remove_serializes_correctly_and_returns_evicted_value_if_it_was_available() {
        let key = TestKey { v: 1 };
        let evicted = TestValue {
            v: "evicted".to_string(),
        };

        let expected_key = bincode::serialize(&key).unwrap().into_boxed_slice();
        let evicted_value = bincode::serialize(&evicted).unwrap().into_boxed_slice();

        let mut backend = MockAsyncKVStorage::new();
        backend
            .expect_remove()
            .with(predicate::eq(expected_key.clone()))
            .return_once(move |_| Ok(Some(evicted_value)));

        let mut db = DB::new(backend);

        assert_eq!(db.remove(Key::new(&key).ok().unwrap()).await, Ok(Some(evicted)))
    }
}
