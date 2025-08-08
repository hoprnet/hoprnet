use async_trait::async_trait;
use futures::TryStreamExt;
use hopr_crypto_types::prelude::*;
use hopr_db_entity::{corrupted_channel, prelude::CorruptedChannel};
use hopr_internal_types::{corrupted_channels::CorruptedChannelEntry, prelude::*};
use hopr_primitive_types::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};

use crate::{
    HoprDbGeneralModelOperations, OptTx,
    db::HoprDb,
    errors::{DbSqlError, Result},
};

/// API for editing [CorruptedChannelEntry] in the DB.
pub struct CorruptedChannelEditor {
    orig: CorruptedChannelEntry,
    model: corrupted_channel::ActiveModel,
    delete: bool,
}

impl CorruptedChannelEditor {
    /// Original channel entry **before** the edits.
    pub fn entry(&self) -> &CorruptedChannelEntry {
        &self.orig
    }
}

/// Defines DB API for accessing information about HOPR payment channels.
#[async_trait]
pub trait HoprDbCorruptedChannelOperations {
    /// Retrieves corrupted channel by its channel ID hash.
    ///
    /// See [generate_channel_id] on how to generate a channel ID hash from source and destination [Addresses](Address).
    async fn get_corrupted_channel_by_id<'a>(
        &'a self,
        tx: OptTx<'a>,
        id: &Hash,
    ) -> Result<Option<CorruptedChannelEntry>>;

    /// Start changes to channel entry.
    /// If the channel with the given ID exists, the [CorruptedChannelEditor] is returned.
    /// Use [`HoprDbCorruptedChannelOperations::finish_corrupted_channel_update`] to commit edits to the DB when done.
    async fn begin_corrupted_channel_update<'a>(
        &'a self,
        tx: OptTx<'a>,
        id: &Hash,
    ) -> Result<Option<CorruptedChannelEditor>>;

    /// Commits changes of the corrupted channel to the database.
    /// Returns the updated corrupted channel, or on deletion, the deleted channel entry.
    ///
    /// It can also return `None` if the channel entry is being set as corrupted and a proper `CorruptedChannelEntry` cannot be
    /// created.
    async fn finish_corrupted_channel_update<'a>(
        &'a self,
        tx: OptTx<'a>,
        editor: CorruptedChannelEditor,
    ) -> Result<Option<CorruptedChannelEntry>>;

    /// Retrieves all corrupted channels information from the DB.
    async fn get_all_corrupted_channels<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<CorruptedChannelEntry>>;

    /// Inserts the given ChannelID as a corrupted channel entry.
    async fn upsert_corrupted_channel<'a>(&'a self, tx: OptTx<'a>, channel_id: ChannelId) -> Result<()>;
}

#[async_trait]
impl HoprDbCorruptedChannelOperations for HoprDb {
    async fn get_corrupted_channel_by_id<'a>(
        &'a self,
        tx: OptTx<'a>,
        id: &Hash,
    ) -> Result<Option<CorruptedChannelEntry>> {
        let id_hex = id.to_hex();
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Ok::<_, DbSqlError>(
                        if let Some(model) = CorruptedChannel::find()
                            .filter(corrupted_channel::Column::ChannelId.eq(id_hex))
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

    async fn begin_corrupted_channel_update<'a>(
        &'a self,
        tx: OptTx<'a>,
        id: &Hash,
    ) -> Result<Option<CorruptedChannelEditor>> {
        let id_hex = id.to_hex();
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    match CorruptedChannel::find()
                        .filter(corrupted_channel::Column::ChannelId.eq(id_hex.clone()))
                        .one(tx.as_ref())
                        .await?
                    {
                        Some(model) => Ok(Some(CorruptedChannelEditor {
                            orig: CorruptedChannelEntry::try_from(model.clone())?,
                            model: model.into_active_model(),
                            delete: false,
                        })),
                        None => Ok(None),
                    }
                })
            })
            .await
    }

    async fn finish_corrupted_channel_update<'a>(
        &'a self,
        tx: OptTx<'a>,
        editor: CorruptedChannelEditor,
    ) -> Result<Option<CorruptedChannelEntry>> {
        let ret = self
            .nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    if !editor.delete {
                        let model = editor.model.update(tx.as_ref()).await?;
                        match CorruptedChannelEntry::try_from(model) {
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

        Ok(ret)
    }

    async fn get_all_corrupted_channels<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<CorruptedChannelEntry>> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    CorruptedChannel::find()
                        .stream(tx.as_ref())
                        .await?
                        .map_err(DbSqlError::from)
                        .try_filter_map(|m| async move { Ok(Some(CorruptedChannelEntry::try_from(m)?)) })
                        .try_collect()
                        .await
                })
            })
            .await
    }

    async fn upsert_corrupted_channel<'a>(&'a self, tx: OptTx<'a>, channel_id: ChannelId) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let channel_entry = CorruptedChannelEntry::from(channel_id);
                    let mut model = corrupted_channel::ActiveModel::from(channel_entry);
                    if let Some(channel) = corrupted_channel::Entity::find()
                        .filter(corrupted_channel::Column::ChannelId.eq(channel_entry.channel_id().to_hex()))
                        .one(tx.as_ref())
                        .await?
                    {
                        model.id = Set(channel.id);
                    }

                    Ok::<_, DbSqlError>(model.save(tx.as_ref()).await?)
                })
            })
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use hopr_crypto_types::{keypairs::ChainKeypair, prelude::Keypair};
    use hopr_internal_types::channels::generate_channel_id;
    use hopr_primitive_types::prelude::Address;

    use crate::{corrupted_channels::HoprDbCorruptedChannelOperations, db::HoprDb};

    #[tokio::test]
    async fn test_insert_get_by_id() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let channel_id = generate_channel_id(&Address::default(), &Address::default());

        db.upsert_corrupted_channel(None, channel_id).await?;

        let from_db = db
            .get_corrupted_channel_by_id(None, &channel_id)
            .await?
            .expect("channel must be present");

        assert_eq!(channel_id, *from_db.channel_id(), "channels must be equal");

        Ok(())
    }

    #[tokio::test]
    async fn test_insert_duplicates_should_not_insert() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let channel_id = generate_channel_id(&Address::default(), &Address::default());

        db.upsert_corrupted_channel(None, channel_id)
            .await
            .context("Inserting a corrupted channel should not fail")?;

        // Attempt to insert the same channel again should fail
        db.upsert_corrupted_channel(None, channel_id)
            .await
            .context("Inserting a duplicate corrupted channel should not fail")?;

        let all_channels = db.get_all_corrupted_channels(None).await?;

        assert_eq!(all_channels.len(), 1, "There should be only one corrupted channel");
        assert_eq!(
            all_channels[0].channel_id(),
            &channel_id,
            "The channel ID should match the inserted one"
        );
        Ok(())
    }
}
