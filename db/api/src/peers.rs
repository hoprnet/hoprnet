use async_trait::async_trait;

use crate::db::HoprDb;

#[async_trait]
pub trait HoprDbPeersOperations {}

#[async_trait]
impl HoprDbPeersOperations for HoprDb {}
