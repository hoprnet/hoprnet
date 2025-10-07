use std::ops::{Bound, RangeBounds};

use async_trait::async_trait;
use futures::{StreamExt, stream::BoxStream};
use hopr_crypto_types::prelude::*;
use hopr_db_entity::{channel, conversions::channels::ChannelStatusUpdate, prelude::Channel};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use sea_query::Condition;
use tracing::instrument;

use crate::{
    HoprDbGeneralModelOperations, HoprIndexerDb, OptTx,
    cache::ChannelParties,
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
    async fn finish_channel_update<'a>(&'a self, tx: OptTx<'a>, editor: ChannelEditor) -> Result<Option<ChannelEntry>>;

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

    /// Fetches all channels that are `Incoming` to this node.
    /// Shorthand for `get_channels_via(tx, ChannelDirection::Incoming, my_node)`
    async fn get_incoming_channels<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<ChannelEntry>>;

    /// Fetches all channels that are `Outgoing` from this node.
    /// Shorthand for `get_channels_via(tx, ChannelDirection::Outgoing, my_node)`
    async fn get_outgoing_channels<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<ChannelEntry>>;

    /// Retrieves all channels information from the DB.
    async fn get_all_channels<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<ChannelEntry>>;

    async fn stream_channels<'a, T: RangeBounds<DateTime<Utc>> + Send>(
        &'a self,
        source: Option<Address>,
        destination: Option<Address>,
        states: &[ChannelStatusDiscriminants],
        closure_range: T,
    ) -> Result<BoxStream<'a, ChannelEntry>>;

    /// Inserts or updates the given channel entry.
    async fn upsert_channel<'a>(&'a self, tx: OptTx<'a>, channel_entry: ChannelEntry) -> Result<()>;
}

#[async_trait]
impl HoprDbChannelOperations for HoprIndexerDb {
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
                    match Channel::find()
                        .filter(channel::Column::ChannelId.eq(id_hex.clone()))
                        .one(tx.as_ref())
                        .await?
                    {
                        Some(model) => Ok(Some(ChannelEditor {
                            orig: ChannelEntry::try_from(model.clone())?,
                            model: model.into_active_model(),
                            delete: false,
                        })),
                        None => Ok(None),
                    }
                })
            })
            .await
    }

    async fn finish_channel_update<'a>(&'a self, tx: OptTx<'a>, editor: ChannelEditor) -> Result<Option<ChannelEntry>> {
        let parties = ChannelParties(editor.orig.source, editor.orig.destination);
        let ret = self
            .nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    if !editor.delete {
                        let model = editor.model.update(tx.as_ref()).await?;
                        match ChannelEntry::try_from(model) {
                            Ok(channel) => Ok::<_, DbSqlError>(Some(channel)),
                            Err(e) => Err(DbSqlError::from(e)),
                        }
                    } else {
                        editor.model.delete(tx.as_ref()).await?;
                        Ok::<_, DbSqlError>(Some(editor.orig))
                    }
                })
            })
            .await?;
        self.caches.src_dst_to_channel.invalidate(&parties).await;

        Ok(ret)
    }

    #[instrument(level = "trace", skip(self, tx), err)]
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
            tracing::warn!(%src, %dst, "cache miss on get_channel_by_parties");
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

    async fn get_incoming_channels<'a>(&'a self, _tx: OptTx<'a>) -> Result<Vec<ChannelEntry>> {
        Ok(self
            .stream_channels(
                None,
                Some(self.me_onchain),
                &[
                    ChannelStatusDiscriminants::Open,
                    ChannelStatusDiscriminants::PendingToClose,
                    ChannelStatusDiscriminants::Closed,
                ],
                ..,
            )
            .await?
            .collect::<Vec<_>>()
            .await)
    }

    async fn get_outgoing_channels<'a>(&'a self, _tx: OptTx<'a>) -> Result<Vec<ChannelEntry>> {
        Ok(self
            .stream_channels(
                Some(self.me_onchain),
                None,
                &[
                    ChannelStatusDiscriminants::Open,
                    ChannelStatusDiscriminants::PendingToClose,
                    ChannelStatusDiscriminants::Closed,
                ],
                ..,
            )
            .await?
            .collect::<Vec<_>>()
            .await)
    }

    async fn get_all_channels<'a>(&'a self, _tx: OptTx<'a>) -> Result<Vec<ChannelEntry>> {
        let entries = self
            .stream_channels(
                None,
                None,
                &[
                    ChannelStatusDiscriminants::Open,
                    ChannelStatusDiscriminants::PendingToClose,
                    ChannelStatusDiscriminants::Closed,
                ],
                ..,
            )
            .await?
            .collect::<Vec<_>>()
            .await;
        Ok(entries)
    }

    async fn stream_channels<'a, T: RangeBounds<DateTime<Utc>> + Send>(
        &'a self,
        source: Option<Address>,
        destination: Option<Address>,
        states: &[ChannelStatusDiscriminants],
        closure_range: T,
    ) -> Result<BoxStream<'a, ChannelEntry>> {
        let mut incoming_cond = Condition::all();
        if let Some(source) = source {
            incoming_cond = incoming_cond.add(channel::Column::Source.eq(source.to_hex()));
        }
        if let Some(destination) = destination {
            incoming_cond = incoming_cond.add(channel::Column::Destination.eq(destination.to_hex()));
        }

        let mut states_condition = Condition::any();
        for state in states {
            // If we're including the pending to close channels in the query, make sure
            // we include range bounds on closure times
            if state == &ChannelStatusDiscriminants::PendingToClose {
                let mut closure_range_condition = Condition::all();
                closure_range_condition = closure_range_condition
                    .add(channel::Column::Status.eq(ChannelStatusDiscriminants::PendingToClose as i8));
                match closure_range.start_bound() {
                    Bound::Included(closure_start) => {
                        closure_range_condition =
                            closure_range_condition.add(channel::Column::ClosureTime.gte(*closure_start))
                    }
                    Bound::Excluded(closure_start) => {
                        closure_range_condition =
                            closure_range_condition.add(channel::Column::ClosureTime.gt(*closure_start))
                    }
                    _ => {}
                }
                match closure_range.end_bound() {
                    Bound::Included(closure_end) => {
                        closure_range_condition =
                            closure_range_condition.add(channel::Column::ClosureTime.lte(*closure_end))
                    }
                    Bound::Excluded(closure_end) => {
                        closure_range_condition =
                            closure_range_condition.add(channel::Column::ClosureTime.lt(*closure_end))
                    }
                    _ => {}
                }
                states_condition = states_condition.add(closure_range_condition);
            } else {
                states_condition = states_condition.add(channel::Column::Status.eq(*state as i8));
            }
        }

        Ok(Channel::find()
            .filter(sea_query::all![incoming_cond, states_condition])
            .stream(self.index_db.read_only())
            .await?
            .filter_map(|maybe_channel| {
                futures::future::ready(
                    maybe_channel
                        .and_then(|c| {
                            ChannelEntry::try_from(c)
                                .map_err(|e| sea_orm::error::DbErr::Custom(format!("cannot decode entity: {e}")))
                        })
                        .inspect_err(|error| tracing::error!(%error, "invalid channel entry"))
                        .ok(),
                )
            })
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

        // The invalidation of the unrealized value is done in the finish_channel_update function.
        // Has no effect if the channel has been inserted.

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::{keypairs::ChainKeypair, prelude::Keypair};
    use hopr_internal_types::{channels::ChannelStatus, prelude::ChannelEntry};
    use hopr_primitive_types::prelude::Address;

    use super::*;
    use crate::{HoprDbGeneralModelOperations, channels::HoprDbChannelOperations};

    #[tokio::test]
    async fn test_insert_get_by_id() -> anyhow::Result<()> {
        let db = HoprIndexerDb::new_in_memory(ChainKeypair::random()).await?;

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
        let db = HoprIndexerDb::new_in_memory(ChainKeypair::random()).await?;

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
        let ckp = ChainKeypair::random();
        let db = HoprIndexerDb::new_in_memory(ckp.clone()).await?;

        let from_db = db
            .get_channel_by_parties(None, &Address::default(), &ckp.public().to_address(), false)
            .await?;

        assert_eq!(None, from_db, "should return None");

        Ok(())
    }

    #[tokio::test]
    async fn test_channel_get_for_destination_that_exists_should_be_returned() -> anyhow::Result<()> {
        let ckp = ChainKeypair::random();
        let db = HoprIndexerDb::new_in_memory(ckp.clone()).await?;

        let expected_destination = Address::default();

        let ce = ChannelEntry::new(
            ckp.public().to_address(),
            expected_destination,
            0.into(),
            0_u32.into(),
            ChannelStatus::Open,
            0_u32.into(),
        );

        db.upsert_channel(None, ce).await?;
        let from_db = db
            .get_channel_by_parties(None, &ckp.public().to_address(), &Address::default(), false)
            .await?;

        assert_eq!(Some(ce), from_db, "should return a valid channel");

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_incoming_outgoing_channels() -> anyhow::Result<()> {
        let ckp = ChainKeypair::random();
        let addr_1 = ckp.public().to_address();
        let addr_2 = ChainKeypair::random().public().to_address();

        let db = HoprIndexerDb::new_in_memory(ckp).await?;

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
