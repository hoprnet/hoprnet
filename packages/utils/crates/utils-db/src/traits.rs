use async_trait::async_trait;
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

#[cfg_attr(test, mockall::automock(type Key = Box < [u8] >; type Value = Box < [u8] >;))]
#[async_trait(? Send)] // not placing the `Send` trait limitations on the trait
pub trait AsyncKVStorage {
    type Key: Serialize;
    type Value: Serialize;

    async fn get(&self, key: Self::Key) -> Result<Self::Value>;

    async fn set(&mut self, key: Self::Key, value: Self::Value) -> Result<Option<Self::Value>>;

    async fn contains(&self, key: Self::Key) -> bool;

    async fn remove(&mut self, key: Self::Key) -> Result<Option<Self::Value>>;

    async fn dump(&self, destination: String) -> Result<()>;

    async fn get_more(&self, prefix: Self::Key, suffix_size: u32, filter: Box<dyn Fn(Self::Key) -> bool>) -> Result<Vec<Self::Value>>;

    async fn batch(
        &mut self,
        operations: Vec<BatchOperation<Self::Key, Self::Value>>,
        wait_for_write: bool,
    ) -> Result<()>;
}

pub trait BinaryAsyncKVStorage: AsyncKVStorage<Key = Box<[u8]>, Value = Box<[u8]>> {}

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
