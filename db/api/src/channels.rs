use async_trait::async_trait;
use hopr_crypto_types::prelude::*;
use hopr_db_entity::channel;
use hopr_db_entity::prelude::Channel;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::str::FromStr;

use crate::db::HoprDb;
use crate::errors::{DbError, Result};
use crate::{DbTimestamp, HoprDbGeneralModelOperations, OptTx};

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
        2 => {
            if let Some(ct) = &model.closure_time {
                let time = DbTimestamp::from_str(ct).map_err(|_| DbError::CorruptedData)?;
                Ok(ChannelStatus::PendingToClose(time.into()))
            } else {
                Err(DbError::CorruptedData)
            }
        }
        _ => Err(DbError::CorruptedData),
    }
}

/// Converts channel DB model to [ChannelEntry].
pub fn model_to_channel_entry(model: &channel::Model) -> Result<ChannelEntry> {
    Ok(ChannelEntry::new(
        model.source.parse()?,
        model.destination.parse()?,
        BalanceType::HOPR.balance_bytes(&model.balance),
        U256::from_be_bytes(&model.ticket_index),
        model_to_channel_status(model)?,
        U256::from_be_bytes(&model.epoch),
    ))
}

/// Converts [ChannelEntry] into the DB active model.
pub fn channel_entry_to_model(channel: ChannelEntry) -> channel::ActiveModel {
    let mut ret = channel::ActiveModel {
        channel_id: Set(channel.get_id().to_hex()),
        source: Set(channel.source.to_hex()),
        destination: Set(channel.destination.to_hex()),
        balance: Set(channel.balance.amount().to_be_bytes().into()),
        epoch: Set(channel.channel_epoch.to_be_bytes().into()),
        ticket_index: Set(channel.ticket_index.to_be_bytes().into()),
        ..Default::default()
    };
    channel_status_to_model(&mut ret, channel.status);
    ret
}

#[async_trait]
pub trait HoprDbChannelOperations {
    async fn get_channel_by_id<'a>(&'a self, tx: OptTx<'a>, id: Hash) -> Result<Option<ChannelEntry>>;

    async fn insert_channel<'a>(&'a self, tx: OptTx<'a>, channel_entry: ChannelEntry) -> Result<()>;
}

#[async_trait]
impl HoprDbChannelOperations for HoprDb {
    async fn get_channel_by_id<'a>(&'a self, tx: OptTx<'a>, id: Hash) -> Result<Option<ChannelEntry>> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Ok::<_, DbError>(
                        if let Some(model) = Channel::find()
                            .filter(channel::Column::ChannelId.eq(id.to_string()))
                            .one(tx.as_ref())
                            .await?
                        {
                            Some(model_to_channel_entry(&model)?)
                        } else {
                            None
                        },
                    )
                })
            })
            .await
    }

    async fn insert_channel<'a>(&'a self, tx: OptTx<'a>, channel_entry: ChannelEntry) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| Box::pin(async move { channel_entry_to_model(channel_entry).save(tx.as_ref()).await }))
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::channels::HoprDbChannelOperations;
    use crate::db::HoprDb;
    use hopr_internal_types::channels::ChannelStatus;
    use hopr_internal_types::prelude::ChannelEntry;
    use hopr_primitive_types::prelude::{Address, BalanceType};

    #[async_std::test]
    async fn test_insert_get() {
        let db = HoprDb::new_in_memory().await;

        let ce = ChannelEntry::new(
            Address::default(),
            Address::default(),
            BalanceType::HOPR.zero(),
            0_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        db.insert_channel(None, ce).await.expect("must insert channel");
        let from_db = db
            .get_channel_by_id(None, ce.get_id())
            .await
            .expect("must get channel")
            .expect("channel must be present");

        assert_eq!(ce, from_db, "channels must be equal");
    }
}
