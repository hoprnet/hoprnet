use crate::errors::DbError;
use crate::traits::{AsyncKVStorage, BatchOperation, KVStorage, StorageValueIterator};
use async_trait::async_trait;
use futures_lite::stream::iter;
use futures_lite::StreamExt;
use std::cmp::Ordering;

#[derive(Default)]
pub struct InMemoryHashMapStorage<K, V>
where
    K: std::cmp::Eq + std::hash::Hash + serde::Serialize,
    V: Clone + serde::Serialize,
{
    data: std::collections::hash_map::HashMap<K, V>,
}

impl<K, V> InMemoryHashMapStorage<K, V>
where
    K: std::cmp::Eq + std::hash::Hash + serde::Serialize,
    V: Clone + serde::Serialize,
{
    pub fn new() -> InMemoryHashMapStorage<K, V> {
        InMemoryHashMapStorage {
            data: std::collections::hash_map::HashMap::new(),
        }
    }
}

impl<K, V> KVStorage for InMemoryHashMapStorage<K, V>
where
    K: Eq + std::hash::Hash + serde::Serialize,
    V: Clone + serde::Serialize,
{
    type Key = K;
    type Value = V;

    fn get(&self, key: Self::Key) -> Result<Option<Self::Value>, DbError> {
        Ok(self.data.get(&key).cloned())
    }

    fn set(&mut self, key: Self::Key, value: Self::Value) -> Result<Option<Self::Value>, DbError> {
        Ok(self.data.insert(key, value))
    }

    fn contains(&self, key: Self::Key) -> Result<bool, DbError> {
        Ok(self.data.contains_key(&key))
    }

    fn remove(&mut self, key: Self::Key) -> Result<Option<Self::Value>, DbError> {
        Ok(self.data.remove(&key))
    }

    fn iterate(&self, prefix: Self::Key, suffix_size: u32) -> Result<Vec<Self::Value>, DbError> {
        todo!("iterate not implement on InMemoryHashMapStorage")
    }

    fn batch(&mut self, operations: Vec<BatchOperation<Self::Key, Self::Value>>) -> Result<(), DbError> {
        todo!("batch not implement on InMemoryHashMapStorage")
    }
}

pub type BinaryHashMapStorage = InMemoryHashMapStorage<Box<[u8]>, Box<[u8]>>;

#[async_trait(?Send)]
impl AsyncKVStorage for BinaryHashMapStorage {
    type Key = Box<[u8]>;
    type Value = Box<[u8]>;

    async fn get(&self, key: Self::Key) -> crate::errors::Result<Option<Self::Value>> {
        Ok(self.data.get(&key).cloned())
    }

    async fn set(&mut self, key: Self::Key, value: Self::Value) -> crate::errors::Result<Option<Self::Value>> {
        Ok(self.data.insert(key, value))
    }

    async fn contains(&self, key: Self::Key) -> crate::errors::Result<bool> {
        Ok(self.data.contains_key(&key))
    }

    async fn remove(&mut self, key: Self::Key) -> crate::errors::Result<Option<Self::Value>> {
        Ok(self.data.remove(&key))
    }

    fn iterate(&self, prefix: Self::Key, suffix_size: u32) -> crate::errors::Result<StorageValueIterator<Self::Value>> {
        let mut first_key: Vec<u8> = prefix.clone().into();
        first_key.extend((0..suffix_size).map(|_| 0u8));

        let mut last_key: Vec<u8> = prefix.clone().into();
        last_key.extend((0..suffix_size).map(|_| 0xffu8));

        let d = iter(self.data.clone().into_iter())
            .filter(move |(key, _)| {
                let upper_bound = key.as_ref().cmp(&last_key);
                let lower_bound = key.as_ref().cmp(&first_key);
                upper_bound != Ordering::Greater && lower_bound != Ordering::Less
            })
            .map(|(_, v)| Ok(v));

        Ok(Box::new(d))
    }

    async fn batch(&mut self, operations: Vec<BatchOperation<Self::Key, Self::Value>>) -> crate::errors::Result<()> {
        for op in operations {
            match op {
                BatchOperation::del(v) => self.data.remove(&v.key),
                BatchOperation::put(v) => self.data.insert(v.key, v.value),
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[allow(dead_code)]
    fn generate_tmp_dir() -> std::path::PathBuf {
        let tmp_dir = TempDir::new().ok().unwrap();
        tmp_dir.into_path()
    }

    #[test]
    fn test_hashmap_storage_contains_on_no_value_should_fail() {
        let db: InMemoryHashMapStorage<i32, i32> = InMemoryHashMapStorage::new();

        assert!(!db.contains(1).unwrap());
    }

    #[test]
    fn test_hashmap_storage_should_return_nothing_on_get_when_a_value_does_not_exist() {
        let db: InMemoryHashMapStorage<i32, i32> = InMemoryHashMapStorage::new();

        assert!(db.get(1).unwrap().is_none());
    }

    #[test]
    fn test_hashmap_storage_should_contains_the_value_if_set() {
        let mut db: InMemoryHashMapStorage<i32, i32> = InMemoryHashMapStorage::new();

        let (expected_key, expected_value) = (1, 2);
        db.set(expected_key, expected_value);

        assert!(db.contains(expected_key).unwrap());
    }

    #[test]
    fn test_hashmap_storage_should_return_a_value_when_it_exists() {
        let mut db: InMemoryHashMapStorage<i32, i32> = InMemoryHashMapStorage::new();

        let (expected_key, expected_value) = (1, 2);
        db.set(expected_key, expected_value);

        assert_eq!(expected_value, db.get(expected_key).unwrap().unwrap());
    }

    #[test]
    fn test_hashmap_storage_should_be_able_to_remove_the_value() {
        let mut db: InMemoryHashMapStorage<i32, i32> = InMemoryHashMapStorage::new();

        let (expected_key, expected_value) = (1, 2);
        db.set(expected_key, expected_value);
        db.remove(expected_key);

        assert!(!db.contains(expected_key).unwrap());
    }
}
