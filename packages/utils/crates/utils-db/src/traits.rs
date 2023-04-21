use async_trait::async_trait;
use crate::errors::Result;


#[cfg_attr(test, mockall::automock(type Key=Box<[u8]>; type Value=Box<[u8]>;))]
#[async_trait(?Send)]           // not placing the `Send` trait limitations on the trait
pub trait AsyncKVStorage {
    type Key;
    type Value;

    #[must_use]
    async fn get(&self, key: Self::Key) -> Result<Self::Value>;

    async fn set(&mut self, key: Self::Key, value: Self::Value) -> Result<Option<Self::Value>>;

    #[must_use]
    async fn contains(&self, key: Self::Key) -> bool;

    async fn remove(&mut self, key: Self::Key) -> Result<Option<Self::Value>>;

    async fn dump(&self, destination: String) -> Result<()>;
}

pub trait BinaryAsyncKVStorage : AsyncKVStorage<Key=Box<[u8]>, Value=Box<[u8]>> {}

pub trait KVStorage {
    type Key;
    type Value;

    #[must_use]
    fn get(&self, key: &Self::Key) -> Option<Self::Value>;

    fn set(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value>;

    #[must_use]
    fn contains(&self, key: &Self::Key) -> bool;

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Value>;

    fn dump(&self) -> crate::errors::Result<()>;
}