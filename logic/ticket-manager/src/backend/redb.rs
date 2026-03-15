use std::str::FromStr;

use hopr_api::{
    chain::{ChannelId, RedeemableTicket},
    types::{internal::prelude::Ticket, primitive::prelude::BytesRepresentable},
};
use redb::{ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition, TableHandle};

use crate::{OutgoingIndexStore, TicketQueue, TicketQueueStore};

const OUT_IDX_TABLE: TableDefinition<([u8; ChannelId::SIZE], u32), u64> = TableDefinition::new("channel_out_index");

/// Implementation of [`OutgoingIndexStore`] and [`TicketQueueStore`] using `redb` database and `postcard` serializer.
pub struct RedbStore {
    db: std::sync::Arc<redb::Database>,
}

impl RedbStore {
    /// Creates a new instance on the given path.
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, RedbStoreError> {
        let db = std::sync::Arc::new(redb::Database::create(path)?);
        let tx = db.begin_write()?;
        tx.open_table(OUT_IDX_TABLE)?;
        tx.commit()?;
        Ok(Self { db })
    }
}

impl OutgoingIndexStore for RedbStore {
    type Error = RedbStoreError;

    fn load_outgoing_index(&self, channel_id: &ChannelId, epoch: u32) -> Result<Option<u64>, Self::Error> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(OUT_IDX_TABLE)?;
        Ok(table.get(((*channel_id).into(), epoch))?.map(|v| v.value()))
    }

    fn save_outgoing_index(&mut self, channel_id: &ChannelId, epoch: u32, index: u64) -> Result<(), Self::Error> {
        let tx = self.db.begin_write()?;
        {
            let mut table = tx.open_table(OUT_IDX_TABLE)?;
            table.insert(((*channel_id).into(), epoch), index)?;
        }
        tx.commit()?;
        Ok(())
    }

    fn delete_outgoing_index(&mut self, channel_id: &ChannelId, epoch: u32) -> Result<(), Self::Error> {
        let tx = self.db.begin_write()?;
        {
            let mut table = tx.open_table(OUT_IDX_TABLE)?;
            table.remove(((*channel_id).into(), epoch))?;
        }
        tx.commit()?;
        Ok(())
    }

    fn iter_outgoing_indices(&self) -> Result<impl Iterator<Item = (ChannelId, u32)>, Self::Error> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(OUT_IDX_TABLE)?;
        Ok(table
            .iter()?
            .filter_map(|v| v.ok())
            .map(|(k, _)| (k.value().0.into(), k.value().1))
            .collect::<Vec<_>>()
            .into_iter())
    }
}

const TABLE_QUEUE_NAME_PREFIX: &str = "ctq_";

type TicketTableDef<'a> = TableDefinition<'a, u128, Vec<u8>>;

#[inline]
fn make_index(ticket: &RedeemableTicket) -> u128 {
    ((ticket.verified_ticket().channel_epoch as u128) << 64) | ticket.verified_ticket().index as u128
}

impl TicketQueueStore for RedbStore {
    type Queue = RedbTicketQueue;

    fn open_or_create_queue(
        &mut self,
        channel_id: &ChannelId,
    ) -> Result<Self::Queue, <Self::Queue as TicketQueue>::Error> {
        {
            let tx = self.db.begin_write()?;
            tx.open_table(TicketTableDef::new(&format!("{TABLE_QUEUE_NAME_PREFIX}{channel_id}")))?;
            tx.commit()?;
        }

        Ok(RedbTicketQueue {
            db: std::sync::Arc::downgrade(&self.db),
            channel_id: *channel_id,
        })
    }

    fn delete_queue(&mut self, channel_id: &ChannelId) -> Result<Vec<Ticket>, <Self::Queue as TicketQueue>::Error> {
        let tx = self.db.begin_write()?;
        let mut ret = Vec::new();
        {
            // Drain all the tickets from the queue first
            let mut table = tx.open_table(TicketTableDef::new(&format!("{TABLE_QUEUE_NAME_PREFIX}{channel_id}")))?;
            while let Some((_, ticket)) = table.pop_first()? {
                let ticket: RedeemableTicket = postcard::from_bytes(&ticket.value())?;
                ret.push(*ticket.verified_ticket()); // Make it unredeemable
            }
        }
        tx.delete_table(TicketTableDef::new(&format!("{TABLE_QUEUE_NAME_PREFIX}{channel_id}")))?;
        tx.commit()?;

        Ok(ret)
    }

    fn iter_queues(&self) -> Result<impl Iterator<Item = ChannelId>, <Self::Queue as TicketQueue>::Error> {
        let tx = self.db.begin_read()?;
        Ok(tx
            .list_tables()?
            .filter_map(|t| {
                t.name()
                    .strip_prefix(TABLE_QUEUE_NAME_PREFIX)
                    .and_then(|c| ChannelId::from_str(c).ok())
            })
            .collect::<Vec<_>>()
            .into_iter())
    }
}

/// Implementation of [`TicketQueue`] using `redb` database and `postcard` serializer,
/// associated with the [`RedbStore`].
pub struct RedbTicketQueue {
    db: std::sync::Weak<redb::Database>,
    channel_id: ChannelId,
}

impl TicketQueue for RedbTicketQueue {
    type Error = RedbStoreError;

    fn len(&self) -> Result<usize, Self::Error> {
        if let Some(db) = self.db.upgrade() {
            let tx = db.begin_read()?;
            let table = tx.open_table(TicketTableDef::new(&format!(
                "{TABLE_QUEUE_NAME_PREFIX}{}",
                self.channel_id
            )))?;
            Ok(table.len()? as usize)
        } else {
            Err(RedbStoreError::Database(redb::Error::DatabaseClosed))
        }
    }

    fn is_empty(&self) -> Result<bool, Self::Error> {
        if let Some(db) = self.db.upgrade() {
            let tx = db.begin_read()?;
            let table = tx.open_table(TicketTableDef::new(&format!(
                "{TABLE_QUEUE_NAME_PREFIX}{}",
                self.channel_id
            )))?;
            Ok(table.is_empty()?)
        } else {
            Err(RedbStoreError::Database(redb::Error::DatabaseClosed))
        }
    }

    fn push(&mut self, ticket: RedeemableTicket) -> Result<(), Self::Error> {
        if let Some(db) = self.db.upgrade() {
            let tx = db.begin_write()?;
            {
                let mut table = tx.open_table(TicketTableDef::new(&format!(
                    "{TABLE_QUEUE_NAME_PREFIX}{}",
                    self.channel_id
                )))?;
                table.insert(make_index(&ticket), postcard::to_stdvec(&ticket)?)?;
            }
            tx.commit()?;
            Ok(())
        } else {
            Err(RedbStoreError::Database(redb::Error::DatabaseClosed))
        }
    }

    fn pop(&mut self) -> Result<Option<RedeemableTicket>, Self::Error> {
        if let Some(db) = self.db.upgrade() {
            let tx = db.begin_write()?;
            let maybe_ticket = {
                let mut table = tx.open_table(TicketTableDef::new(&format!(
                    "{TABLE_QUEUE_NAME_PREFIX}{}",
                    self.channel_id
                )))?;
                table.pop_first()?.map(|(_, v)| v.value())
            };
            tx.commit()?;
            if let Some(ticket_bytes) = maybe_ticket {
                Ok(Some(postcard::from_bytes(&ticket_bytes)?))
            } else {
                Ok(None)
            }
        } else {
            Err(RedbStoreError::Database(redb::Error::DatabaseClosed))
        }
    }

    fn peek(&self) -> Result<Option<RedeemableTicket>, Self::Error> {
        if let Some(db) = self.db.upgrade() {
            let tx = db.begin_read()?;
            let table = tx.open_table(TicketTableDef::new(&format!(
                "{TABLE_QUEUE_NAME_PREFIX}{}",
                self.channel_id
            )))?;
            let ticket_bytes = table.first()?.map(|(_, v)| v.value());
            if let Some(ticket_bytes) = ticket_bytes {
                Ok(postcard::from_bytes(&ticket_bytes)?)
            } else {
                Ok(None)
            }
        } else {
            Err(RedbStoreError::Database(redb::Error::DatabaseClosed))
        }
    }

    fn iter_unordered(&self) -> Result<impl Iterator<Item = Result<RedeemableTicket, Self::Error>>, Self::Error> {
        if let Some(db) = self.db.upgrade() {
            let tx = db.begin_read()?;
            let table = tx.open_table(TicketTableDef::new(&format!(
                "{TABLE_QUEUE_NAME_PREFIX}{}",
                self.channel_id
            )))?;
            Ok(table
                .iter()?
                .map(|result| {
                    result.map_err(RedbStoreError::from).and_then(|(_, v)| {
                        postcard::from_bytes::<RedeemableTicket>(&v.value()).map_err(RedbStoreError::from)
                    })
                })
                .collect::<Vec<Result<RedeemableTicket, RedbStoreError>>>()
                .into_iter())
        } else {
            Err(RedbStoreError::Database(redb::Error::DatabaseClosed))
        }
    }
}

/// Errors returned by the [`RedbStore`].
#[derive(Debug, thiserror::Error)]
pub enum RedbStoreError {
    #[error("database error: {0}")]
    Database(#[from] redb::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] postcard::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<redb::DatabaseError> for RedbStoreError {
    fn from(error: redb::DatabaseError) -> Self {
        Self::Database(error.into())
    }
}

impl From<redb::TransactionError> for RedbStoreError {
    fn from(error: redb::TransactionError) -> Self {
        Self::Database(error.into())
    }
}

impl From<redb::TableError> for RedbStoreError {
    fn from(error: redb::TableError) -> Self {
        Self::Database(error.into())
    }
}

impl From<redb::StorageError> for RedbStoreError {
    fn from(error: redb::StorageError) -> Self {
        Self::Database(error.into())
    }
}

impl From<redb::CommitError> for RedbStoreError {
    fn from(error: redb::CommitError) -> Self {
        Self::Database(error.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::tests::*;

    #[test]
    fn redb_queue_maintains_natural_ticket_order() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        queue_maintains_natural_ticket_order(RedbStore::new(file)?.open_or_create_queue(&Default::default())?)
    }

    #[test]
    fn redb_queue_returns_all_tickets() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        queue_returns_all_tickets(RedbStore::new(file)?.open_or_create_queue(&Default::default())?)
    }
    #[test]
    fn redb_queue_is_empty_when_drained() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        queue_is_empty_when_drained(RedbStore::new(file)?.open_or_create_queue(&Default::default())?)
    }

    #[test]
    fn redb_queue_returns_empty_iterator_when_drained() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        queue_returns_empty_iterator_when_drained(RedbStore::new(file)?.open_or_create_queue(&Default::default())?)
    }
    #[test]
    fn redb_queue_returns_correct_total_ticket_value() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        queue_returns_correct_total_ticket_value(RedbStore::new(file)?.open_or_create_queue(&Default::default())?)
    }

    #[test]
    fn redb_queue_returns_correct_total_ticket_value_with_min_index() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        queue_returns_correct_total_ticket_value_with_min_index(
            RedbStore::new(file)?.open_or_create_queue(&Default::default())?,
        )
    }

    #[test]
    fn redb_out_index_store_should_load_existing_index_for_channel_epoch() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        out_index_store_should_load_existing_index_for_channel_epoch(RedbStore::new(file)?)
    }

    #[test]
    fn redb_out_index_store_should_not_load_non_existing_index_for_channel_epoch() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        out_index_store_should_not_load_non_existing_index_for_channel_epoch(RedbStore::new(file)?)
    }

    #[test]
    fn redb_out_index_store_should_store_new_index_for_channel_epoch() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        out_index_store_should_store_new_index_for_channel_epoch(RedbStore::new(file)?)
    }

    #[test]
    fn redb_out_index_store_should_delete_existing_index_for_channel_epoch() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        out_index_store_should_delete_existing_index_for_channel_epoch(RedbStore::new(file)?)
    }

    #[test]
    fn redb_out_index_should_update_existing_index_for_channel_epoch() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        out_index_should_update_existing_index_for_channel_epoch(RedbStore::new(file)?)
    }

    #[test]
    fn redb_out_index_store_should_iterate_existing_indices_for_channel_epoch() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        out_index_store_should_iterate_existing_indices_for_channel_epoch(RedbStore::new(file)?)
    }

    #[test]
    fn redb_ticket_store_should_create_new_queue_for_channel() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        ticket_store_should_create_new_queue_for_channel(RedbStore::new(file)?)
    }

    #[test]
    fn redb_ticket_store_should_open_existing_queue_for_channel() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        ticket_store_should_open_existing_queue_for_channel(RedbStore::new(file)?)
    }

    #[test]
    fn redb_ticket_store_should_delete_existing_queue_for_channel() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        ticket_store_should_delete_existing_queue_for_channel(RedbStore::new(file)?)
    }

    #[test]
    fn redb_ticket_store_should_delete_existing_queue_for_channel_and_return_neglected_tickets() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        ticket_store_should_delete_existing_queue_for_channel_and_return_neglected_tickets(RedbStore::new(file)?)
    }

    #[test]
    fn redb_ticket_store_should_iterate_existing_queues_for_channel() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        ticket_store_should_iterate_existing_queues_for_channel(RedbStore::new(file)?)
    }

    #[test]
    fn redb_ticket_store_should_not_fail_to_delete_non_existing_queue_for_channel() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        ticket_store_should_not_fail_to_delete_non_existing_queue_for_channel(RedbStore::new(file)?)
    }
}
