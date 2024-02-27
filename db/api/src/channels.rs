use crate::db::HoprDb;
use async_trait::async_trait;

#[async_trait]
pub trait HoprDbChannelOperations {}

#[async_trait]
impl HoprDbChannelOperations for HoprDb {}
