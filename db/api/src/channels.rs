use async_trait::async_trait;
use hopr_crypto_types::prelude::*;
use hopr_db_entity::channel;
use hopr_db_entity::prelude::Channel;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use std::str::FromStr;

use crate::db::HoprDb;
use crate::DbTimestamp;
use crate::errors::{DbError, Result};

/// Updates given active model of a channel with the given status.
pub fn channel_status_to_model(model: &mut channel::ActiveModel, new_status: ChannelStatus) {
    model.status = Set(u8::from(new_status) as i32);
    if let ChannelStatus::PendingToClose(t) = new_status {
        model.closure_time = Set(Some(DbTimestamp::from(t).to_rfc3339()))
    }
}

/// Converts status of the channel DB model to [ChannelStatus] enum.
pub fn model_to_channel_status(model: &channel::Model) -> Result<ChannelStatus> {
    match model.status {
        0 => Ok(ChannelStatus::Closed),
        1 => Ok(ChannelStatus::Open),
        2 => if let Some(ct) = &model.closure_time {
                let time = DbTimestamp::from_str(&ct).map_err(|_| DbError::CorruptedData)?;
                Ok(ChannelStatus::PendingToClose(time.into()))
            } else {
                Err(DbError::CorruptedData)
            },
        _ => Err(DbError::CorruptedData)
    }
}

/// Converts channel DB model to [ChannelEntry].
pub fn model_to_channel_entry(model: &channel::Model) -> Result<ChannelEntry> {
    Ok(ChannelEntry::new(
        model.source.parse()?,
        model.destination.parse()?,
        BalanceType::HOPR.balance_bytes(&model.balance)?,
        U256::from_big_endian(&model.ticket_index),
        model_to_channel_status(model)?,
        U256::from_big_endian(&model.epoch),
    ))
}

/// Converts [ChannelEntry] into the DB active model.
pub fn channel_entry_to_model(channel: ChannelEntry) -> channel::ActiveModel {
    let mut r = channel::ActiveModel {
        channel_id: Set(channel.get_id().to_hex()),
        source: Set(channel.source.to_hex()),
        destination: Set(channel.destination.to_hex()),
        balance: Set(channel.balance.amount().to_bytes().to_vec()),
        epoch: Set(channel.channel_epoch.to_bytes().to_vec()),
        ticket_index: Set(channel.ticket_index.to_bytes().to_vec()),
        ..Default::default()
    };
    channel_status_to_model(&mut r, channel.status);
    r
}

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

       model_to_channel_entry(&channel)
    }
}
