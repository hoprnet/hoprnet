use crate::{
    errors::{DbError, Result},
    traits::AsyncKVStorage,
};
use futures_lite::stream::StreamExt;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt::{Display, Formatter},
    ops::Deref,
};
use utils_types::traits::BinarySerializable;

pub struct Batch {
    pub ops: Vec<crate::traits::BatchOperation<Box<[u8]>, Box<[u8]>>>,
}

// NOTE: The LevelDB implementation's iterator needs to know the precise size of the
pub fn serialize_to_bytes<S: Serialize + BinarySerializable>(s: &S) -> Result<Vec<u8>> {
    Ok(Vec::from(s.to_bytes()))
    // bincode::serialize(&s).map_err(|e| DbError::SerializationError(e.to_string()))
}

impl Batch {
    pub fn new() -> Self {
        Self {
            ops: Vec::with_capacity(10),
        }
    }

    pub fn put<U: Serialize>(&mut self, key: Key, value: U) {
        let key: Box<[u8]> = key.into();
        let value: Box<[u8]> = bincode::serialize(&value).unwrap().into_boxed_slice();

        self.ops
            .push(crate::traits::BatchOperation::put(crate::traits::Put { key, value }));
    }

    pub fn del(&mut self, key: Key) {
        let key: Box<[u8]> = key.into();

        self.ops
            .push(crate::traits::BatchOperation::del(crate::traits::Del { key }));
    }
}

#[derive(Debug, Clone)]
pub struct Key {
    key: Box<[u8]>,
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.key))
    }
}

impl Key {
    pub fn new<T: Serialize + BinarySerializable>(object: &T) -> Result<Self> {
        Ok(Self { key: object.to_bytes() })
    }

    pub fn new_from_str(object: &str) -> Result<Self> {
        Ok(Self {
            key: Box::from(object.as_bytes()),
        })
    }

    pub fn new_with_prefix<T: Serialize + BinarySerializable>(object: &T, prefix: &str) -> Result<Self> {
        let key = serialize_to_bytes(object)?;

        let mut result = Vec::with_capacity(prefix.len() + key.len());
        result.extend_from_slice(prefix.as_bytes().as_ref());
        result.extend_from_slice(key.as_ref());

        Ok(Self {
            key: result.into_boxed_slice(),
        })
    }

    pub fn new_bytes_with_prefix(object: &[u8], prefix: &str) -> Result<Self> {
        let mut result = Vec::with_capacity(prefix.len() + object.len());
        result.extend_from_slice(prefix.as_bytes().as_ref());
        result.extend_from_slice(object);

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

pub struct DB<T: AsyncKVStorage<Key = Box<[u8]>, Value = Box<[u8]>>> {
    backend: T,
}

impl<T: AsyncKVStorage<Key = Box<[u8]>, Value = Box<[u8]>>> DB<T> {
    pub fn new(backend: T) -> Self {
        Self { backend }
    }

    pub async fn contains(&self, key: Key) -> bool {
        self.backend.contains(key.into()).await.is_ok_and(|v| v)
    }

    pub async fn get_or_none<V: DeserializeOwned>(&self, key: Key) -> Result<Option<V>> {
        let key: T::Key = key.into();

        match self.backend.get(key.into()).await {
            Ok(Some(val)) => match bincode::deserialize(&val) {
                Ok(deserialized) => Ok(Some(deserialized)),
                Err(e) => Err(DbError::DeserializationError(format!(
                    "during get operation: {}",
                    e.to_string().as_str()
                ))),
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
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
            Some(v) => bincode::deserialize(v.as_ref()).map(|v| Some(v)).map_err(|e| {
                DbError::DeserializationError(format!("during set operation: {}", e.to_string().as_str()))
            }),
            None => Ok(None),
        }
    }

    pub async fn remove<V: DeserializeOwned>(&mut self, key: Key) -> Result<Option<V>> {
        let key: T::Key = key.into();

        match self.backend.remove(key).await? {
            Some(v) => bincode::deserialize(v.as_ref()).map(|v| Some(v)).map_err(|e| {
                DbError::DeserializationError(format!("during remove operation: {}", e.to_string().as_str()))
            }),
            None => Ok(None),
        }
    }

    pub async fn get_more<V: Serialize + DeserializeOwned>(
        &self,
        prefix: Box<[u8]>,
        suffix_size: u32,
        filter: &dyn Fn(&V) -> bool,
    ) -> Result<Vec<V>> {
        let mut output = Vec::new();

        let mut data_stream = Box::into_pin(self.backend.iterate(prefix, suffix_size)?);

        // fail fast for the first value that cannot be deserialized
        while let Some(value) = data_stream.next().await {
            let value =
                bincode::deserialize::<V>(value?.as_ref()).map_err(|e| DbError::DeserializationError(e.to_string()))?;

            if (*filter)(&value) {
                output.push(value);
            }
        }

        Ok(output)
    }

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
    use utils_types::traits::BinarySerializable;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestKey {
        v: u8,
    }

    impl BinarySerializable for TestKey {
        const SIZE: usize = 1;

        /// Deserializes the type from a binary blob.
        fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
            if data.len() != Self::SIZE {
                Err(utils_types::errors::GeneralError::InvalidInput)
            } else {
                Ok(Self { v: data[0] })
            }
        }

        /// Serializes the type into a fixed size binary blob.
        fn to_bytes(&self) -> Box<[u8]> {
            Box::new([self.v])
        }
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
            .returning(|_| Ok(true));

        let db = DB::new(backend);

        assert!(db.contains(Key::new(&key).ok().unwrap()).await)
    }

    #[async_std::test]
    async fn test_db_get_serializes_correctly_and_succeeds_if_a_value_is_available() {
        let key = TestKey { v: 1 };
        let value = TestValue { v: "value".to_string() };

        let expected_key = bincode::serialize(&key).unwrap().into_boxed_slice();
        let ser_value: Result<Option<Box<[u8]>>> = Ok(Some(bincode::serialize(&value).unwrap().into_boxed_slice()));

        let mut backend = MockAsyncKVStorage::new();
        backend
            .expect_get()
            .with(predicate::eq(expected_key.clone()))
            .return_once(move |_| ser_value);

        let db = DB::new(backend);

        assert_eq!(
            db.get_or_none::<TestValue>(Key::new(&key).ok().unwrap()).await,
            Ok(Some(value))
        )
    }

    #[async_std::test]
    async fn test_db_get_serializes_correctly_and_fails_if_a_value_is_unavailable() {
        let key = TestKey { v: 1 };

        let expected_key = bincode::serialize(&key).unwrap().into_boxed_slice();

        let mut backend = MockAsyncKVStorage::new();
        backend
            .expect_get()
            .with(predicate::eq(expected_key.clone()))
            .return_once(|_| -> Result<Option<Box<[u8]>>> { Err(DbError::NotFound) });

        let db = DB::new(backend);

        assert_eq!(
            db.get_or_none::<TestValue>(Key::new(&key).ok().unwrap()).await,
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
