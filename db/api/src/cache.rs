use moka::future::Cache;
use crate::info::{IndexerData, SafeInfo};

#[derive(Debug, Clone, PartialEq, Eq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
pub enum CachedValue {
    IndexerDataCache(IndexerData),
    SafeInfoCache(SafeInfo)
}

#[derive(Debug, Clone, Default)]
pub struct DbCaches {
    pub(crate) values: Cache<CachedValueDiscriminants, CachedValue>
}
