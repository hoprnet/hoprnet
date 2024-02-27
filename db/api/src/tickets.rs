use async_trait::async_trait;

use crate::db::HoprDb;

#[async_trait]
pub trait HoprDbTicketOperations {}

#[async_trait]
impl HoprDbTicketOperations for HoprDb {}
