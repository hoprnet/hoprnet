use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};

trait KVStorable {
    type Key;
    type Value;

    #[must_use]
    fn get(&self, key: &Self::Key) -> Option<Self::Value>;

    fn set(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value>;

    #[must_use]
    fn contains(&self, key: &Self::Key) -> bool;

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Value>;

    fn dump(&self) -> Result<(), std::fmt::Error>;
}

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

impl<K, V> KVStorable for InMemoryHashMapStorage<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
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

    fn dump(&self) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

#[cfg(not(wasm))]
pub struct PickleStorage<V> {
    data: PickleDb,
    _phantom_value: std::marker::PhantomData<V>,
}

impl<V> PickleStorage<V> {
    pub fn new(path: &str) -> PickleStorage<V> {
        PickleStorage {
            data: PickleDb::new(
                path,
                PickleDbDumpPolicy::AutoDump,
                SerializationMethod::Json,
            ),
            _phantom_value: Default::default(),
        }
    }
}

impl<V> KVStorable for PickleStorage<V>
where
    V: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    type Key = String;
    type Value = V;

    fn get(&self, key: &Self::Key) -> Option<Self::Value> {
        self.data.get::<Self::Value>(key)
    }

    fn set(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value> {
        let _ = self.data.set(key.as_str(), &value);
        None
    }

    fn contains(&self, key: &Self::Key) -> bool {
        self.data.exists(key)
    }

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Value> {
        let _ = self.data.rem(key);
        None
    }

    fn dump(&self) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn generate_tmp_dir() -> std::path::PathBuf {
        let tmp_dir = TempDir::new().ok().unwrap();
        tmp_dir.into_path()
    }

    #[test]
    fn test_picklestorage_contains_on_no_value_should_fail() {
        let tmp_dir_path = generate_tmp_dir();
        let _cleanup = scopeguard::guard((), |_| {
            fs::remove_dir_all(tmp_dir_path.clone()).ok();
        });

        let file_path = format!("{}/test.db", tmp_dir_path.as_path().to_str().unwrap());
        let db: PickleStorage<i32> = PickleStorage::new(file_path.as_str());

        assert!(!db.contains(&"a".to_string()));
    }

    #[test]
    fn test_picklestorage_should_contains_the_value_if_set() {
        let tmp_dir_path = generate_tmp_dir();
        let _cleanup = scopeguard::guard((), |_| {
            fs::remove_dir_all(tmp_dir_path.clone()).ok();
        });

        let file_path = format!("{}/test.db", tmp_dir_path.as_path().to_str().unwrap());
        let mut db: PickleStorage<i32> = PickleStorage::new(file_path.as_str());

        let (expected_key, expected_value) = ("a".to_string(), 2);
        db.set(expected_key.clone(), expected_value);

        assert!(db.contains(&expected_key));
    }

    #[test]
    fn test_picklestorage_should_return_nothing_on_get_when_a_value_does_not_exist() {
        let tmp_dir_path = generate_tmp_dir();
        let _cleanup = scopeguard::guard((), |_| {
            fs::remove_dir_all(tmp_dir_path.clone()).ok();
        });

        let file_path = format!("{}/test.db", tmp_dir_path.as_path().to_str().unwrap());
        let db: PickleStorage<i32> = PickleStorage::new(file_path.as_str());

        assert!(db.get(&"a".to_string()).is_none());
    }

    #[test]
    fn test_picklestorage_should_return_a_value_when_it_exists() {
        let tmp_dir_path = generate_tmp_dir();
        let _cleanup = scopeguard::guard((), |_| {
            fs::remove_dir_all(tmp_dir_path.clone()).ok();
        });

        let file_path = format!("{}/test.db", tmp_dir_path.as_path().to_str().unwrap());
        let mut db: PickleStorage<i32> = PickleStorage::new(file_path.as_str());

        let (expected_key, expected_value) = ("a".to_string(), 2);
        db.set(expected_key.clone(), expected_value);

        assert_eq!(expected_value, db.get(&expected_key).unwrap());
    }

    #[test]
    fn test_picklestorage_should_be_able_to_remove_the_value() {
        let tmp_dir_path = generate_tmp_dir();
        let _cleanup = scopeguard::guard((), |_| {
            fs::remove_dir_all(tmp_dir_path.clone()).ok();
        });

        let file_path = format!("{}/test.db", tmp_dir_path.as_path().to_str().unwrap());
        let mut db: PickleStorage<i32> = PickleStorage::new(file_path.as_str());

        let (expected_key, expected_value) = ("a".to_string(), 2);
        db.set(expected_key.clone(), expected_value);
        db.remove(&expected_key);

        assert!(!db.contains(&expected_key));
    }

    #[test]
    fn test_hashmapstorage_contains_on_no_value_should_fail() {
        let db: InMemoryHashMapStorage<i32, i32> = InMemoryHashMapStorage::new();

        assert!(!db.contains(&1));
    }

    #[test]
    fn test_hashmapstorage_should_return_nothing_on_get_when_a_value_does_not_exist() {
        let db: InMemoryHashMapStorage<i32, i32> = InMemoryHashMapStorage::new();

        assert!(db.get(&1).is_none());
    }

    #[test]
    fn test_hashmapstorage_should_contains_the_value_if_set() {
        let mut db: InMemoryHashMapStorage<i32, i32> = InMemoryHashMapStorage::new();

        let (expected_key, expected_value) = (1, 2);
        db.set(expected_key, expected_value);

        assert!(db.contains(&expected_key));
    }

    #[test]
    fn test_hashmapstorage_should_return_a_value_when_it_exists() {
        let mut db: InMemoryHashMapStorage<i32, i32> = InMemoryHashMapStorage::new();

        let (expected_key, expected_value) = (1, 2);
        db.set(expected_key, expected_value);

        assert_eq!(expected_value, db.get(&expected_key).unwrap());
    }

    #[test]
    fn test_hashmapstorage_should_be_able_to_remove_the_value() {
        let mut db: InMemoryHashMapStorage<i32, i32> = InMemoryHashMapStorage::new();

        let (expected_key, expected_value) = (1, 2);
        db.set(expected_key, expected_value);
        db.remove(&expected_key);

        assert!(!db.contains(&expected_key));
    }
}
