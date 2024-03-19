use async_trait::async_trait;
use futures::TryStreamExt;
use hopr_crypto_types::prelude::*;
use hopr_db_entity::channel;
use hopr_db_entity::prelude::Channel;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::db::HoprDb;
use crate::errors::{DbError, Result};
use crate::{HoprDbGeneralModelOperations, OptTx};

/// Defines DB API for accessing information about HOPR payment channels.
#[async_trait]
pub trait HoprDbChannelOperations {
    /// Retrieves channel by its channel ID hash.
    ///
    /// See [generate_channel_id] on how to generate a channel ID hash from source and destination [Addresses](Address).
    async fn get_channel_by_id<'a>(&'a self, tx: OptTx<'a>, id: Hash) -> Result<Option<ChannelEntry>>;

    /// Fetches all channels that are `Incoming` to the given `target`, or `Outgoing` from the given `target`
    async fn get_channels_via<'a>(
        &'a self,
        tx: OptTx<'a>,
        direction: ChannelDirection,
        target: Address,
    ) -> Result<Vec<ChannelEntry>>;

    /// Fetches all channels that are `Incoming` to this node.
    /// Shorthand for `get_channels_via(tx, ChannelDirection::Incoming, my_node)`
    async fn get_incoming_channels<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<ChannelEntry>>;

    /// Fetches all channels that are `Incoming` to this node.
    /// Shorthand for `get_channels_via(tx, ChannelDirection::Outgoing, my_node)`
    async fn get_outgoing_channels<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<ChannelEntry>>;

    /// Retrieves all channel information from the DB.
    async fn get_all_channels<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<ChannelEntry>>;

    /// Inserts or updates the given channel entry.
    async fn upsert_channel<'a>(&'a self, tx: OptTx<'a>, channel_entry: ChannelEntry) -> Result<()>;
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
                            .filter(channel::Column::ChannelId.eq(id.to_hex()))
                            .one(tx.as_ref())
                            .await?
                        {
                            Some(model.try_into()?)
                        } else {
                            None
                        },
                    )
                })
            })
            .await
    }

    async fn get_channels_via<'a>(
        &'a self,
        tx: OptTx<'a>,
        direction: ChannelDirection,
        target: Address,
    ) -> Result<Vec<ChannelEntry>> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Channel::find()
                        .filter(match direction {
                            ChannelDirection::Incoming => channel::Column::Destination.eq(target.to_string()),
                            ChannelDirection::Outgoing => channel::Column::Source.eq(target.to_string()),
                        })
                        .all(tx.as_ref())
                        .await?
                        .into_iter()
                        .map(|x| ChannelEntry::try_from(x).map_err(DbError::from))
                        .collect::<Result<Vec<_>>>()
                })
            })
            .await
    }

    async fn get_incoming_channels<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<ChannelEntry>> {
        self.get_channels_via(tx, ChannelDirection::Incoming, self.chain_key.public().to_address())
            .await
    }

    async fn get_outgoing_channels<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<ChannelEntry>> {
        self.get_channels_via(tx, ChannelDirection::Outgoing, self.chain_key.public().to_address())
            .await
    }

    async fn get_all_channels<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<ChannelEntry>> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Channel::find()
                        .stream(tx.as_ref())
                        .await?
                        .map_err(DbError::from)
                        .try_filter_map(|m| async move { Ok(Some(ChannelEntry::try_from(m)?)) })
                        .try_collect()
                        .await
                })
            })
            .await
    }

    async fn upsert_channel<'a>(&'a self, tx: OptTx<'a>, channel_entry: ChannelEntry) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let mut model = channel::ActiveModel::from(channel_entry);
                    if let Some(channel) = channel::Entity::find()
                        .filter(channel::Column::ChannelId.eq(channel_entry.get_id().to_hex()))
                        .one(tx.as_ref())
                        .await?
                    {
                        model.id = Set(channel.id);
                    }

                    Ok::<_, DbError>(model.save(tx.as_ref()).await?)
                })
            })
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::channels::HoprDbChannelOperations;
    use crate::db::HoprDb;
    use crate::HoprDbGeneralModelOperations;
    use hopr_crypto_types::keypairs::ChainKeypair;
    use hopr_crypto_types::prelude::Keypair;
    use hopr_internal_types::channels::ChannelStatus;
    use hopr_internal_types::prelude::{ChannelDirection, ChannelEntry};
    use hopr_primitive_types::prelude::{Address, BalanceType};

    #[async_std::test]
    async fn test_insert_get() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let ce = ChannelEntry::new(
            Address::default(),
            Address::default(),
            BalanceType::HOPR.zero(),
            0_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        db.upsert_channel(None, ce).await.expect("must insert channel");
        let from_db = db
            .get_channel_by_id(None, ce.get_id())
            .await
            .expect("must get channel")
            .expect("channel must be present");

        assert_eq!(ce, from_db, "channels must be equal");
    }

    #[async_std::test]
    async fn test_channel_get_for_destination_that_does_not_exist_returns_none() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let from_db = db
            .get_channels_via(None, ChannelDirection::Incoming, Address::default())
            .await
            .expect("db should not fail")
            .first()
            .cloned();

        assert_eq!(None, from_db, "should return None");
    }

    #[async_std::test]
    async fn test_channel_get_for_destination_that_exists_should_be_returned() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let expected_destination = Address::default();

        let ce = ChannelEntry::new(
            Address::default(),
            expected_destination.clone(),
            BalanceType::HOPR.zero(),
            0_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        db.upsert_channel(None, ce).await.expect("must insert channel");
        let from_db = db
            .get_channels_via(None, ChannelDirection::Incoming, Address::default())
            .await
            .expect("db should not fail")
            .first()
            .cloned();

        assert_eq!(Some(ce), from_db, "should return a valid channel");
    }

    #[async_std::test]
    async fn test_incoming_outgoing_channels() {
        let ckp = ChainKeypair::random();
        let addr_1 = ckp.public().to_address();
        let addr_2 = ChainKeypair::random().public().to_address();

        let db = HoprDb::new_in_memory(ckp).await;

        let ce_1 = ChannelEntry::new(
            addr_1,
            addr_2,
            BalanceType::HOPR.zero(),
            1_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        let ce_2 = ChannelEntry::new(
            addr_2,
            addr_1,
            BalanceType::HOPR.zero(),
            2_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    db_clone.upsert_channel(Some(tx), ce_1).await?;
                    db_clone.upsert_channel(Some(tx), ce_2).await
                })
            })
            .await
            .unwrap();

        let incoming = db
            .get_incoming_channels(None)
            .await
            .expect("should get incoming channels");

        let outgoing = db
            .get_outgoing_channels(None)
            .await
            .expect("should get outgoing channels");

        assert_eq!(vec![ce_2], incoming);
        assert_eq!(vec![ce_1], outgoing);
    }
}
