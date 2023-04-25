use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::errors::Result;
use crate::traits::BinaryAsyncKVStorage;

pub struct Batch {
    pub ops: Vec<crate::traits::BatchOperation<Box<[u8]>, Box<[u8]>>>,
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

pub struct DB<T: BinaryAsyncKVStorage> {
    backend: T,
}

impl<T: BinaryAsyncKVStorage> DB<T> {
    pub fn new(backend: T) -> Self {
        DB::<T> { backend }
    }

    pub async fn contains<K: Serialize>(&self, key: &K) -> bool {
        let key: T::Key = bincode::serialize(&key).unwrap().into_boxed_slice();
        self.backend.contains(key).await
    }

    pub async fn get<K: Serialize, V: DeserializeOwned>(&self, key: &K) -> Result<V> {
        let key: T::Key = bincode::serialize(&key).unwrap().into_boxed_slice();
        self.backend.get(key).await.map(move |v| {
            let value: V = bincode::deserialize(&v).unwrap();
            value
        })
    }

    pub async fn set<K: Serialize, V: Serialize + DeserializeOwned>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<Option<V>> {
        let key: T::Key = bincode::serialize(&key).unwrap().into_boxed_slice();
        let value: T::Value = bincode::serialize(&value).unwrap().into_boxed_slice();
        self.backend.set(key, value).await.map(move |v| {
            v.map(move |x| {
                let value: V = bincode::deserialize(&x).unwrap();
                value
            })
        })
    }

    pub async fn remove<U: Serialize, V: DeserializeOwned>(&mut self, key: &U) -> Result<Option<V>> {
        let key: T::Key = bincode::serialize(&key).unwrap().into_boxed_slice();
        self.backend.remove(key).await.map(move |v| {
            v.map(move |x| {
                let value: V = bincode::deserialize(&x).unwrap();
                value
            })
        })
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
    use mockall::*;

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

        assert!(db.contains(&key).await)
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

        assert_eq!(db.get::<_, TestValue>(&key).await, Ok(value))
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

        assert_eq!(db.get::<_, TestValue>(&key).await, Err(DbError::NotFound))
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

        assert_eq!(db.set(&key, &value).await, Ok(None))
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

        assert_eq!(db.set(&key, &value).await, Err(DbError::NotFound))
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

        assert_eq!(db.set(&key, &value).await, Ok(Some(evicted)))
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

        assert_eq!(db.remove::<_, TestValue>(&key).await, Ok(None))
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

        assert_eq!(db.remove::<_, TestValue>(&key).await, Err(DbError::NotFound))
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

        assert_eq!(db.remove(&key).await, Ok(Some(evicted)))
    }
}
