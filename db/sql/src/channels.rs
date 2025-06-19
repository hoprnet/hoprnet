use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt, stream::BoxStream};
use hopr_crypto_types::prelude::*;
use hopr_db_entity::{channel, conversions::channels::ChannelStatusUpdate, prelude::Channel};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};

use crate::{
    HoprDbGeneralModelOperations, OptTx,
    cache::ChannelParties,
    db::HoprDb,
    errors::{DbSqlError, Result},
};

/// API for editing [ChannelEntry] in the DB.
pub struct ChannelEditor {
    orig: ChannelEntry,
    model: channel::ActiveModel,
    delete: bool,
}

impl ChannelEditor {
    /// Original channel entry **before** the edits.
    pub fn entry(&self) -> &ChannelEntry {
        &self.orig
    }

    /// Change the HOPR balance of the channel.
    pub fn change_balance(mut self, balance: HoprBalance) -> Self {
        self.model.balance = Set(balance.amount().to_be_bytes().to_vec());
        self
    }

    /// Change the channel status.
    pub fn change_status(mut self, status: ChannelStatus) -> Self {
        self.model.set_status(status);
        self
    }

    /// Change the ticket index.
    pub fn change_ticket_index(mut self, index: impl Into<U256>) -> Self {
        self.model.ticket_index = Set(index.into().to_be_bytes().to_vec());
        self
    }

    /// Change the channel epoch.
    pub fn change_epoch(mut self, epoch: impl Into<U256>) -> Self {
        self.model.epoch = Set(epoch.into().to_be_bytes().to_vec());
        self
    }

    /// If set, the channel will be deleted, no other edits will be done.
    pub fn delete(mut self) -> Self {
        self.delete = true;
        self
    }
}

/// Defines DB API for accessing information about HOPR payment channels.
#[async_trait]
pub trait HoprDbChannelOperations {
    /// Retrieves channel by its channel ID hash.
    ///
    /// See [generate_channel_id] on how to generate a channel ID hash from source and destination [Addresses](Address).
    async fn get_channel_by_id<'a>(&'a self, tx: OptTx<'a>, id: &Hash) -> Result<Option<ChannelEntry>>;

    /// Start changes to channel entry.
    /// If the channel with the given ID exists, the [ChannelEditor] is returned.
    /// Use [`HoprDbChannelOperations::finish_channel_update`] to commit edits to the DB when done.
    async fn begin_channel_update<'a>(&'a self, tx: OptTx<'a>, id: &Hash) -> Result<Option<ChannelEditor>>;

    /// Commits changes of the channel to the database.
    /// Returns the updated channel, or on deletion, the deleted channel entry.
    async fn finish_channel_update<'a>(&'a self, tx: OptTx<'a>, editor: ChannelEditor) -> Result<ChannelEntry>;

    /// Retrieves the channel by source and destination.
    /// This operation should be able to use cache since it can be also called from
    /// performance-sensitive locations.
    async fn get_channel_by_parties<'a>(
        &'a self,
        tx: OptTx<'a>,
        src: &Address,
        dst: &Address,
        use_cache: bool,
    ) -> Result<Option<ChannelEntry>>;

    /// Fetches all channels that are `Incoming` to the given `target`, or `Outgoing` from the given `target`
    async fn get_channels_via<'a>(
        &'a self,
        tx: OptTx<'a>,
        direction: ChannelDirection,
        target: &Address,
    ) -> Result<Vec<ChannelEntry>>;

    /// Fetches all channels that are `Incoming` to this node.
    /// Shorthand for `get_channels_via(tx, ChannelDirection::Incoming, my_node)`
    async fn get_incoming_channels<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<ChannelEntry>>;

    /// Fetches all channels that are `Outgoing` from this node.
    /// Shorthand for `get_channels_via(tx, ChannelDirection::Outgoing, my_node)`
    async fn get_outgoing_channels<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<ChannelEntry>>;

    /// Retrieves all channel information from the DB.
    async fn get_all_channels<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<ChannelEntry>>;

    /// Returns a stream of all channels that are `Open` or `PendingToClose` with an active grace period.s
    async fn stream_active_channels<'a>(&'a self) -> Result<BoxStream<'a, Result<ChannelEntry>>>;

    /// Inserts or updates the given channel entry.
    async fn upsert_channel<'a>(&'a self, tx: OptTx<'a>, channel_entry: ChannelEntry) -> Result<()>;
}

#[async_trait]
impl HoprDbChannelOperations for HoprDb {
    async fn get_channel_by_id<'a>(&'a self, tx: OptTx<'a>, id: &Hash) -> Result<Option<ChannelEntry>> {
        let id_hex = id.to_hex();
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Ok::<_, DbSqlError>(
                        if let Some(model) = Channel::find()
                            .filter(channel::Column::ChannelId.eq(id_hex))
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

    async fn begin_channel_update<'a>(&'a self, tx: OptTx<'a>, id: &Hash) -> Result<Option<ChannelEditor>> {
        let id_hex = id.to_hex();
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Ok::<_, DbSqlError>(
                        if let Some(model) = Channel::find()
                            .filter(channel::Column::ChannelId.eq(id_hex))
                            .one(tx.as_ref())
                            .await?
                        {
                            Some(ChannelEditor {
                                orig: model.clone().try_into()?,
                                model: model.into_active_model(),
                                delete: false,
                            })
                        } else {
                            None
                        },
                    )
                })
            })
            .await
    }

    async fn finish_channel_update<'a>(&'a self, tx: OptTx<'a>, editor: ChannelEditor) -> Result<ChannelEntry> {
        let epoch = editor.model.epoch.clone();
        let parties = ChannelParties(editor.orig.source, editor.orig.destination);
        let ret = self
            .nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    if !editor.delete {
                        let model = editor.model.update(tx.as_ref()).await?;
                        Ok::<_, DbSqlError>(model.try_into()?)
                    } else {
                        editor.model.delete(tx.as_ref()).await?;
                        Ok::<_, DbSqlError>(editor.orig)
                    }
                })
            })
            .await?;
        self.caches.src_dst_to_channel.invalidate(&parties).await;

        // Finally invalidate any unrealized values from the cache.
        // This might be a no-op if the channel was not in the cache
        // like for channels that are not ours.
        let channel_id = editor.orig.get_id();
        if let Some(channel_epoch) = epoch.try_as_ref() {
            self.caches
                .unrealized_value
                .invalidate(&(channel_id, U256::from_big_endian(channel_epoch.as_slice())))
                .await;
        }

        Ok(ret)
    }

    async fn get_channel_by_parties<'a>(
        &'a self,
        tx: OptTx<'a>,
        src: &Address,
        dst: &Address,
        use_cache: bool,
    ) -> Result<Option<ChannelEntry>> {
        let fetch_channel = async move {
            let src_hex = src.to_hex();
            let dst_hex = dst.to_hex();
            self.nest_transaction(tx)
                .await?
                .perform(|tx| {
                    Box::pin(async move {
                        Ok::<_, DbSqlError>(
                            if let Some(model) = Channel::find()
                                .filter(channel::Column::Source.eq(src_hex))
                                .filter(channel::Column::Destination.eq(dst_hex))
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
        };

        if use_cache {
            Ok(self
                .caches
                .src_dst_to_channel
                .try_get_with(ChannelParties(*src, *dst), fetch_channel)
                .await?)
        } else {
            fetch_channel.await
        }
    }

    async fn get_channels_via<'a>(
        &'a self,
        tx: OptTx<'a>,
        direction: ChannelDirection,
        target: &Address,
    ) -> Result<Vec<ChannelEntry>> {
        let target_hex = target.to_hex();
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Channel::find()
                        .filter(match direction {
                            ChannelDirection::Incoming => channel::Column::Destination.eq(target_hex),
                            ChannelDirection::Outgoing => channel::Column::Source.eq(target_hex),
                        })
                        .all(tx.as_ref())
                        .await?
                        .into_iter()
                        .map(|x| ChannelEntry::try_from(x).map_err(DbSqlError::from))
                        .collect::<Result<Vec<_>>>()
                })
            })
            .await
    }

    async fn get_incoming_channels<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<ChannelEntry>> {
        self.get_channels_via(tx, ChannelDirection::Incoming, &self.me_onchain)
            .await
    }

    async fn get_outgoing_channels<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<ChannelEntry>> {
        self.get_channels_via(tx, ChannelDirection::Outgoing, &self.me_onchain)
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
                        .map_err(DbSqlError::from)
                        .try_filter_map(|m| async move { Ok(Some(ChannelEntry::try_from(m)?)) })
                        .try_collect()
                        .await
                })
            })
            .await
    }

    async fn stream_active_channels<'a>(&'a self) -> Result<BoxStream<'a, Result<ChannelEntry>>> {
        Ok(Channel::find()
            .filter(
                channel::Column::Status
                    .eq(i8::from(ChannelStatus::Open))
                    .or(channel::Column::Status
                        .eq(i8::from(ChannelStatus::PendingToClose(
                            hopr_platform::time::native::current_time(), // irrelevant
                        )))
                        .and(channel::Column::ClosureTime.gt(Utc::now()))),
            )
            .stream(&self.index_db)
            .await?
            .map_err(DbSqlError::from)
            .and_then(|m| async move { Ok(ChannelEntry::try_from(m)?) })
            .boxed())
    }

    async fn upsert_channel<'a>(&'a self, tx: OptTx<'a>, channel_entry: ChannelEntry) -> Result<()> {
        let parties = ChannelParties(channel_entry.source, channel_entry.destination);
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

                    Ok::<_, DbSqlError>(model.save(tx.as_ref()).await?)
                })
            })
            .await?;

        self.caches.src_dst_to_channel.invalidate(&parties).await;

        // Finally, invalidate any unrealized values from the cache.
        // This might be a no-op if the channel was not in the cache
        // like for channels that are not ours.
        let channel_id = channel_entry.get_id();
        let channel_epoch = channel_entry.channel_epoch;
        self.caches
            .unrealized_value
            .invalidate(&(channel_id, channel_epoch))
            .await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::{keypairs::ChainKeypair, prelude::Keypair};
    use hopr_internal_types::{
        channels::ChannelStatus,
        prelude::{ChannelDirection, ChannelEntry},
    };
    use hopr_primitive_types::prelude::Address;

    use crate::{HoprDbGeneralModelOperations, channels::HoprDbChannelOperations, db::HoprDb};

    #[tokio::test]
    async fn test_insert_get_by_id() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let ce = ChannelEntry::new(
            Address::default(),
            Address::default(),
            0.into(),
            0_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        db.upsert_channel(None, ce).await?;
        let from_db = db
            .get_channel_by_id(None, &ce.get_id())
            .await?
            .expect("channel must be present");

        assert_eq!(ce, from_db, "channels must be equal");

        Ok(())
    }

    #[tokio::test]
    async fn test_insert_get_by_parties() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let a = Address::from(random_bytes());
        let b = Address::from(random_bytes());

        let ce = ChannelEntry::new(a, b, 0.into(), 0_u32.into(), ChannelStatus::Open, 0_u32.into());

        db.upsert_channel(None, ce).await?;
        let from_db = db
            .get_channel_by_parties(None, &a, &b, false)
            .await?
            .context("channel must be present")?;

        assert_eq!(ce, from_db, "channels must be equal");

        Ok(())
    }

    #[tokio::test]
    async fn test_channel_get_for_destination_that_does_not_exist_returns_none() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let from_db = db
            .get_channels_via(None, ChannelDirection::Incoming, &Address::default())
            .await?
            .first()
            .cloned();

        assert_eq!(None, from_db, "should return None");

        Ok(())
    }

    #[tokio::test]
    async fn test_channel_get_for_destination_that_exists_should_be_returned() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let expected_destination = Address::default();

        let ce = ChannelEntry::new(
            Address::default(),
            expected_destination,
            0.into(),
            0_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        db.upsert_channel(None, ce).await?;
        let from_db = db
            .get_channels_via(None, ChannelDirection::Incoming, &Address::default())
            .await?
            .first()
            .cloned();

        assert_eq!(Some(ce), from_db, "should return a valid channel");

        Ok(())
    }

    #[tokio::test]
    async fn test_incoming_outgoing_channels() -> anyhow::Result<()> {
        let ckp = ChainKeypair::random();
        let addr_1 = ckp.public().to_address();
        let addr_2 = ChainKeypair::random().public().to_address();

        let db = HoprDb::new_in_memory(ckp).await?;

        let ce_1 = ChannelEntry::new(
            addr_1,
            addr_2,
            0.into(),
            1_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        let ce_2 = ChannelEntry::new(
            addr_2,
            addr_1,
            0.into(),
            2_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone.upsert_channel(Some(tx), ce_1).await?;
                    db_clone.upsert_channel(Some(tx), ce_2).await
                })
            })
            .await?;

        assert_eq!(vec![ce_2], db.get_incoming_channels(None).await?);
        assert_eq!(vec![ce_1], db.get_outgoing_channels(None).await?);

        Ok(())
    }
}
