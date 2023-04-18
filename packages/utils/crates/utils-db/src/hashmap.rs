use crate::errors::DbError;
use crate::traits::KVStorage;

pub struct InMemoryHashMapStorage<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
    V: Clone,
{
    data: std::collections::hash_map::HashMap<K, V>,
}

impl<K, V> InMemoryHashMapStorage<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
    V: Clone,
{
    pub fn new() -> InMemoryHashMapStorage<K, V> {
        InMemoryHashMapStorage {
            data: std::collections::hash_map::HashMap::new(),
        }
    }
}

impl<K, V> KVStorage for InMemoryHashMapStorage<K, V>
where
    K: Eq + std::hash::Hash,
    V: Clone,
{
    type Key = K;
    type Value = V;

    fn get(&self, key: &Self::Key) -> Option<Self::Value> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value> {
        self.data.insert(key, value)
    }

    fn contains(&self, key: &Self::Key) -> bool {
        self.data.contains_key(key)
    }

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Value> {
        self.data.remove(key)
    }

    fn dump(&self) -> Result<(), DbError> {
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

        assert!(!db.contains(&1));
    }

    #[test]
    fn test_hashmap_storage_should_return_nothing_on_get_when_a_value_does_not_exist() {
        let db: InMemoryHashMapStorage<i32, i32> = InMemoryHashMapStorage::new();

        assert!(db.get(&1).is_none());
    }

    #[test]
    fn test_hashmap_storage_should_contains_the_value_if_set() {
        let mut db: InMemoryHashMapStorage<i32, i32> = InMemoryHashMapStorage::new();

        let (expected_key, expected_value) = (1, 2);
        db.set(expected_key, expected_value);

        assert!(db.contains(&expected_key));
    }

    #[test]
    fn test_hashmap_storage_should_return_a_value_when_it_exists() {
        let mut db: InMemoryHashMapStorage<i32, i32> = InMemoryHashMapStorage::new();

        let (expected_key, expected_value) = (1, 2);
        db.set(expected_key, expected_value);

        assert_eq!(expected_value, db.get(&expected_key).unwrap());
    }

    #[test]
    fn test_hashmap_storage_should_be_able_to_remove_the_value() {
        let mut db: InMemoryHashMapStorage<i32, i32> = InMemoryHashMapStorage::new();

        let (expected_key, expected_value) = (1, 2);
        db.set(expected_key, expected_value);
        db.remove(&expected_key);

        assert!(!db.contains(&expected_key));
    }
}
