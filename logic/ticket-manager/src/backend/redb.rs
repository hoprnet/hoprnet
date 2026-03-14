use hopr_api::chain::{ChannelId, RedeemableTicket};
use redb::ReadableDatabase;

use crate::{OutgoingIndexStore, TicketQueue, TicketQueueStore};

pub struct RedbStore {
    db: std::sync::Arc<redb::Database>,
}

impl RedbStore {
    pub fn new(path: &str) -> Result<Self, RedbStoreError> {
        let db = std::sync::Arc::new(redb::Database::create(path)?);
        Ok(Self { db })
    }
}

impl OutgoingIndexStore for RedbStore {
    type Error = RedbStoreError;

    fn load_outgoing_index(&self, channel_id: &ChannelId, epoch: u32) -> Result<Option<u64>, Self::Error> {
        //let tx = self.db.begin_read()?;
        todo!()
    }

    fn save_outgoing_index(&mut self, channel_id: &ChannelId, epoch: u32, index: u64) -> Result<(), Self::Error> {
        todo!()
    }

    fn delete_outgoing_index(&mut self, channel_id: &ChannelId, epoch: u32) -> Result<(), Self::Error> {
        todo!()
    }

    fn iter_outgoing_indices(&self) -> impl Iterator<Item = (ChannelId, u32)> {
        std::iter::empty()
    }
}

impl TicketQueueStore for RedbStore {
    type Queue = RedbTicketQueue;

    fn open_or_create_queue(
        &mut self,
        channel_id: &ChannelId,
    ) -> Result<Self::Queue, <Self::Queue as TicketQueue>::Error> {
        todo!()
    }

    fn delete_queue(&mut self, channel_id: &ChannelId) -> Result<(), <Self::Queue as TicketQueue>::Error> {
        todo!()
    }

    fn iter_queues(&self) -> impl Iterator<Item = ChannelId> {
        std::iter::empty()
    }
}

pub struct RedbTicketQueue {
    db: std::sync::Weak<redb::Database>,
}

impl TicketQueue for RedbTicketQueue {
    type Error = RedbStoreError;

    fn len(&self) -> usize {
        todo!()
    }

    fn push(&mut self, ticket: RedeemableTicket) -> Result<(), Self::Error> {
        todo!()
    }

    fn pop(&mut self) -> Result<Option<RedeemableTicket>, Self::Error> {
        todo!()
    }

    fn peek(&self) -> Result<Option<RedeemableTicket>, Self::Error> {
        todo!()
    }

    fn iter_unordered(&self) -> impl Iterator<Item = Result<RedeemableTicket, Self::Error>> {
        std::iter::empty()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RedbStoreError {
    #[error("database error: {0}")]
    Database(#[from] redb::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] postcard::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
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
    use super::{super::tests::*, *};

    #[test]
    fn redb_queue_maintains_natural_ticket_order() -> anyhow::Result<()> {
        queue_maintains_natural_ticket_order(RedbTicketQueue {
            db: std::sync::Weak::new(),
        })
    }

    #[test]
    fn redb_queue_returns_all_tickets() -> anyhow::Result<()> {
        queue_returns_all_tickets(RedbTicketQueue {
            db: std::sync::Weak::new(),
        })
    }

    #[test]
    fn redb_queue_is_empty_when_drained() -> anyhow::Result<()> {
        queue_is_empty_when_drained(RedbTicketQueue {
            db: std::sync::Weak::new(),
        })
    }

    #[test]
    fn redb_queue_returns_empty_iterator_when_drained() -> anyhow::Result<()> {
        queue_returns_empty_iterator_when_drained(RedbTicketQueue {
            db: std::sync::Weak::new(),
        })
    }

    #[test]
    fn redb_queue_returns_correct_total_ticket_value() -> anyhow::Result<()> {
        queue_returns_correct_total_ticket_value(RedbTicketQueue {
            db: std::sync::Weak::new(),
        })
    }

    #[test]
    fn redb_queue_returns_correct_total_ticket_value_with_min_index() -> anyhow::Result<()> {
        queue_returns_correct_total_ticket_value_with_min_index(RedbTicketQueue {
            db: std::sync::Weak::new(),
        })
    }

    #[test]
    fn redb_out_index_store_should_load_existing_index_for_channel_epoch() -> anyhow::Result<()> {
        out_index_store_should_load_existing_index_for_channel_epoch(RedbStore {
            db: std::sync::Arc::new(redb::Database::create("test.db")?),
        })
    }

}
