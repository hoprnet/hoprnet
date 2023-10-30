use async_trait::async_trait;
use futures_lite::Stream;
use serde::{Deserialize, Serialize};

use crate::errors::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Del<K>
where
    K: Serialize,
{
    pub key: K,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Put<K, V>
where
    K: Serialize,
    V: Serialize,
{
    pub key: K,
    pub value: V,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(non_camel_case_types)]
pub enum BatchOperation<K, V>
where
    K: Serialize,
    V: Serialize,
{
    del(Del<K>),
    put(Put<K, V>),
}

pub type StorageValueIterator<T> = Box<dyn Stream<Item = crate::errors::Result<T>>>;

#[cfg_attr(test, mockall::automock(type Key = Box < [u8] >; type Value = Box < [u8] >;))]
#[async_trait(? Send)] // not placing the `Send` trait limitations on the trait
pub trait AsyncKVStorage {
    type Key: Serialize;
    type Value: Serialize;

    /// Gets a single database entry
    async fn get(&self, key: Self::Key) -> Result<Option<Self::Value>>;

    /// Sets a single database entry
    async fn set(&mut self, key: Self::Key, value: Self::Value) -> Result<Option<Self::Value>>;

    /// Returns true if database contains an entry that matches the given key
    async fn contains(&self, key: Self::Key) -> Result<bool>;

    /// Removes the database entry that matches the given key
    async fn remove(&mut self, key: Self::Key) -> Result<Option<Self::Value>>;

    /// Dumps the contents of the database into a file
    async fn dump(&self, destination: String) -> Result<()>;

    /// Returns an iterator that yields all database entries whose key matches
    /// the given prefix and the length of the suffix. Does not match shorter
    /// or longer suffixes, even though they have the right suffix.
    fn iterate(&self, prefix: Self::Key, suffix_size: u32) -> Result<StorageValueIterator<Self::Value>>;

    /// Returns an iterator that yields all database entries whose is in the
    /// interval from `start` (inclusive) and `end` (inclusive).
    fn iterate_range(&self, start: Self::Key, end: Self::Key) -> Result<StorageValueIterator<Self::Value>>;

    /// Constructs batch query
    async fn batch(
        &mut self,
        operations: Vec<BatchOperation<Self::Key, Self::Value>>,
        wait_for_write: bool,
    ) -> Result<()>;

    /// Flushes data to disk
    async fn flush(&mut self) -> Result<()>;
}

pub trait KVStorage {
    type Key;
    type Value;

    #[must_use]
    fn get(&self, key: &Self::Key) -> Option<Self::Value>;

    fn set(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value>;

    #[must_use]
    fn contains(&self, key: &Self::Key) -> bool;

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Value>;

    fn dump(&self) -> Result<()>;
}
