use async_trait::async_trait;

use crate::db::HoprDb;

#[async_trait]
pub trait HoprDbRegistryOperations {}

#[async_trait]
impl HoprDbRegistryOperations for HoprDb {}
