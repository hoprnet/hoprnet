use std::sync::Arc;
use async_trait::async_trait;
use futures::stream::BoxStream;
use hopr_crypto_types::prelude::Hash;
use hopr_internal_types::acknowledgement::AcknowledgedTicket;
use hopr_internal_types::prelude::{AcknowledgedTicketStatus, ChannelEntry};

use crate::errors::Result;
use crate::tickets::HoprDbTicketOperations;

pub type WinningTicketSender = futures::channel::mpsc::Sender<AcknowledgedTicket>;

/// Allows to select multiple tickets (if `index` is `None`)
/// or a single ticket (with given `index`) in the given channel and epoch.
/// The selection can be further restricted to select ticket only in the given `state`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TicketSelector {
    /// Channel ID
    pub channel_id: Hash,
    /// Channel epoch
    pub epoch: u32,
    /// If given, will select single ticket with the given index
    /// in the given channel and epoch.
    pub index: Option<u64>,
    /// Further restriction to tickets with the given state.
    pub state: Option<AcknowledgedTicketStatus>,
    /// Further restrict to only aggregated tickets.
    pub only_aggregated: bool,
}

impl From<&AcknowledgedTicket> for TicketSelector {
    fn from(value: &AcknowledgedTicket) -> Self {
        Self {
            channel_id: value.ticket.channel_id,
            epoch: value.ticket.channel_epoch,
            index: Some(value.ticket.index),
            state: Some(value.status),
            only_aggregated: value.ticket.index_offset > 1,
        }
    }
}

impl From<&ChannelEntry> for TicketSelector {
    fn from(value: &ChannelEntry) -> Self {
        Self {
            channel_id: value.get_id(),
            epoch: value.channel_epoch.as_u32(),
            index: None,
            state: None,
            only_aggregated: false,
        }
    }
}

/// Manages winning tickets FIFO queue and database.
#[async_trait]
pub trait TicketManager {
    async fn update_ticket_states<'a>(&'a self, selector: TicketSelector, state: AcknowledgedTicketStatus) -> Result<BoxStream<'a, AcknowledgedTicket>>;

    async fn count_tickets(&self, selector: TicketSelector) -> Result<usize>;

    async fn get_winning_tickets_in_state<'a>(&'a self, selector: TicketSelector, state: AcknowledgedTicketStatus) -> Result<BoxStream<'a, AcknowledgedTicket>>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, smart_default::SmartDefault)]
pub struct TicketManagerConfig {
    #[default = 10_000_000]
    pub winning_ticket_queue_size: usize
}

#[derive(Debug)]
pub struct HoprTicketManager<Db: HoprDbTicketOperations> {
    db: Db,
    ticket_queue_rx: Arc<futures::channel::mpsc::Receiver<AcknowledgedTicket>>,
    ticket_queue_tx: WinningTicketSender,
}

impl<Db: HoprDbTicketOperations + Clone> Clone for HoprTicketManager<Db> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            ticket_queue_rx: self.ticket_queue_rx.clone(),
            ticket_queue_tx: self.ticket_queue_tx.clone()
        }
    }
}

impl<Db: HoprDbTicketOperations + Send + Sync> HoprTicketManager<Db> {
    pub fn new(db: Db, cfg: TicketManagerConfig) -> Self {
        let (ticket_queue_tx, rx) = futures::channel::mpsc::channel(cfg.winning_ticket_queue_size);
        Self {
            db,
            ticket_queue_tx,
            ticket_queue_rx: Arc::new(rx)
        }
    }

    pub fn new_sender(&self) -> WinningTicketSender {
        self.ticket_queue_tx.clone()
    }

    pub async fn ticket_loop(self) {
        todo!()
    }
}

#[async_trait]
impl<Db: HoprDbTicketOperations + Send + Sync> TicketManager for HoprTicketManager<Db> {
    async fn update_ticket_states<'a>(&'a self, selector: TicketSelector, state: AcknowledgedTicketStatus) -> Result<BoxStream<'a, AcknowledgedTicket>> {
        todo!()
    }

    async fn count_tickets(&self, selector: TicketSelector) -> Result<usize> {
        todo!()
    }

    async fn get_winning_tickets_in_state<'a>(&'a self, selector: TicketSelector, state: AcknowledgedTicketStatus) -> Result<BoxStream<'a, AcknowledgedTicket>> {
        todo!()
    }
}
