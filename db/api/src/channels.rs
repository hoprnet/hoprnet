use async_trait::async_trait;
use hopr_crypto_types::prelude::*;
use hopr_db_entity::channel;
use hopr_db_entity::prelude::Channel;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::str::FromStr;

use crate::db::HoprDb;
use crate::errors::{DbError, Result};

#[async_trait]
pub trait HoprDbChannelOperations {
    async fn get_channel_by_id(&self, id: &Hash) -> Result<ChannelEntry>;
}

#[async_trait]
impl HoprDbChannelOperations for HoprDb {
    async fn get_channel_by_id(&self, id: &Hash) -> Result<ChannelEntry> {
        let channel: channel::Model = Channel::find()
            .filter(channel::Column::ChannelId.eq(id.to_string()))
            .one(&self.db)
            .await?
            .ok_or(DbError::NotFound)?;

        let status = match channel.status {
            0 => ChannelStatus::Closed,
            1 => ChannelStatus::Open,
            2 => {
                if let Some(ct) = channel.closure_time {
                    let time = chrono::DateTime::<chrono::Utc>::from_str(&ct).map_err(|_| DbError::CorruptedData)?;
                    ChannelStatus::PendingToClose(time.into())
                } else {
                    return Err(DbError::CorruptedData);
                }
            }
            _ => return Err(DbError::CorruptedData),
        };

        Ok(ChannelEntry::new(
            channel.source.parse()?,
            channel.destination.parse()?,
            BalanceType::HOPR.balance(U256::from_big_endian(&channel.balance)),
            channel.ticket_index.into(),
            status,
            channel.epoch.into(),
        ))
    }
}
