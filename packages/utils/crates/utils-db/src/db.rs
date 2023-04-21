use serde::{Serialize,de::DeserializeOwned};

use crate::errors::Result;
use crate::traits::{MockAsyncKVStorage,BinaryAsyncKVStorage};

pub struct DB<T: BinaryAsyncKVStorage> {
    backend: T,
}

impl<T: BinaryAsyncKVStorage> DB<T> {
    pub fn new(backend: T) -> Self {
        DB::<T> {
            backend
        }
    }

    pub async fn contains<K: Serialize>(&self, key: &K) -> bool {
        let key: T::Key = bincode::serialize(&key).unwrap().into_boxed_slice();
        self.backend.contains(key).await
    }

    pub async fn get<V: DeserializeOwned, K: Serialize>(&self, key: &K) -> Result<V> {
        let key: T::Key = bincode::serialize(&key).unwrap().into_boxed_slice();
        self.backend
            .get(key).await
            .map(move |v| {
                let value: V = bincode::deserialize(&v).unwrap();
                value
            })
    }

    pub async fn set<V: Serialize + DeserializeOwned, K: Serialize>(&mut self, key: &K, value: &V) -> Result<Option<V>> {
        let key: T::Key = bincode::serialize(&key).unwrap().into_boxed_slice();
        let value: T::Value = bincode::serialize(&value).unwrap().into_boxed_slice();
        self.backend
            .set(key, value).await
            .map(move |v| {
                v.map(move |x| {
                    let value: V = bincode::deserialize(&x).unwrap();
                    value
                })
            })
    }

    pub async fn remove<V: DeserializeOwned, U: Serialize>(&mut self, key: &U) -> Result<Option<V>> {
        let key: T::Key = bincode::serialize(&key).unwrap().into_boxed_slice();
        self.backend
            .remove(key).await
            .map(move |v| {
                v.map(move |x| {
                    let value: V = bincode::deserialize(&x).unwrap();
                    value
                })
            })
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leveldb() {
        let backend = MockAsyncKVStorage::new();
    }
}