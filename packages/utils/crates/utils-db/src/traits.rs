use async_trait::async_trait;
use crate::errors::Result;

// not placing the `Send` trait limitations on the trait
#[async_trait(?Send)]
pub trait AsyncKVStorage {
    type Key;
    type Value;

    #[must_use]
    async fn get(&self, key: Self::Key) -> Option<Self::Value>;

    async fn set(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value>;

    #[must_use]
    async fn contains(&self, key: Self::Key) -> bool;

    async fn remove(&mut self, key: Self::Key) -> Option<Self::Value>;

    async fn dump(&self, destination: String) -> Result<()>;
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

    fn dump(&self) -> crate::errors::Result<()>;
}