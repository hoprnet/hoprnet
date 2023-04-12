use async_trait::async_trait;

#[async_trait]
pub trait AsyncKVStorage {
    type Key;
    type Value;

    #[must_use]
    async fn get(&self, key: &Self::Key) -> Option<Self::Value>;

    async fn set(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value>;

    #[must_use]
    async fn contains(&self, key: &Self::Key) -> bool;

    async fn remove(&mut self, key: &Self::Key) -> Option<Self::Value>;

    async fn dump(&self) -> Result<(), std::fmt::Error>;
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

    fn dump(&self) -> Result<(), std::fmt::Error>;
}