use hopr_api::chain::{ChannelId, RedeemableTicket};

use crate::{TicketQueue, TicketQueueStore};

pub struct RedbStore {
    db: std::sync::Arc<redb::Database>,
}

impl RedbStore {
    pub fn new(db: std::sync::Arc<redb::Database>) -> Self {
        Self { db }
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
    type Error = redb::Error;

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
