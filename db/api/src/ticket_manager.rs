// use async_trait::async_trait;
// use futures::stream::BoxStream;
// use hopr_crypto_types::prelude::Hash;
// use hopr_internal_types::acknowledgement::AcknowledgedTicket;
// use hopr_internal_types::prelude::{AcknowledgedTicketStatus, ChannelEntry};
// use std::sync::Arc;

// use crate::errors::Result;
// use crate::tickets::HoprDbTicketOperations;

// pub type WinningTicketSender = futures::channel::mpsc::Sender<AcknowledgedTicket>;

// /// Allows to select multiple tickets (if `index` is `None`)
// /// or a single ticket (with given `index`) in the given channel and epoch.
// /// The selection can be further restricted to select ticket only in the given `state`.
// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
// pub struct TicketSelector {
//     /// Channel ID
//     pub channel_id: Hash,
//     /// Channel epoch
//     pub epoch: u32,
//     /// If given, will select single ticket with the given index
//     /// in the given channel and epoch.
//     pub index: Option<u64>,
//     /// Further restriction to tickets with the given state.
//     pub state: Option<AcknowledgedTicketStatus>,
//     /// Further restrict to only aggregated tickets.
//     pub only_aggregated: bool,
// }

// impl From<&AcknowledgedTicket> for TicketSelector {
//     fn from(value: &AcknowledgedTicket) -> Self {
//         Self {
//             channel_id: value.ticket.channel_id,
//             epoch: value.ticket.channel_epoch,
//             index: Some(value.ticket.index),
//             state: Some(value.status),
//             only_aggregated: value.ticket.index_offset > 1,
//         }
//     }
// }

// impl From<&ChannelEntry> for TicketSelector {
//     fn from(value: &ChannelEntry) -> Self {
//         Self {
//             channel_id: value.get_id(),
//             epoch: value.channel_epoch.as_u32(),
//             index: None,
//             state: None,
//             only_aggregated: false,
//         }
//     }
// }

// /// Manages winning tickets FIFO queue and database.
// #[async_trait]
// pub trait TicketManager {
//     async fn update_ticket_states<'a>(
//         &'a self,
//         selector: TicketSelector,
//         state: AcknowledgedTicketStatus,
//     ) -> Result<BoxStream<'a, AcknowledgedTicket>>;

//     async fn count_tickets(&self, selector: TicketSelector) -> Result<usize>;

//     async fn get_winning_tickets_in_state<'a>(
//         &'a self,
//         selector: TicketSelector,
//         state: AcknowledgedTicketStatus,
//     ) -> Result<BoxStream<'a, AcknowledgedTicket>>;
// }

// #[derive(Copy, Clone, Debug, PartialEq, Eq, smart_default::SmartDefault)]
// pub struct TicketManagerConfig {
//     #[default = 10_000_000]
//     pub winning_ticket_queue_size: usize,
// }

// #[derive(Debug)]
// pub struct HoprTicketManager<Db: HoprDbTicketOperations> {
//     db: Db,
//     ticket_queue_rx: Arc<futures::channel::mpsc::Receiver<AcknowledgedTicket>>,
//     ticket_queue_tx: WinningTicketSender,
// }

// impl<Db: HoprDbTicketOperations + Clone> Clone for HoprTicketManager<Db> {
//     fn clone(&self) -> Self {
//         Self {
//             db: self.db.clone(),
//             ticket_queue_rx: self.ticket_queue_rx.clone(),
//             ticket_queue_tx: self.ticket_queue_tx.clone(),
//         }
//     }
// }

// impl<Db: HoprDbTicketOperations + Send + Sync> HoprTicketManager<Db> {
//     pub fn new(db: Db, cfg: TicketManagerConfig) -> Self {
//         let (ticket_queue_tx, rx) = futures::channel::mpsc::channel(cfg.winning_ticket_queue_size);
//         Self {
//             db,
//             ticket_queue_tx,
//             ticket_queue_rx: Arc::new(rx),
//         }
//     }

//     pub fn new_sender(&self) -> WinningTicketSender {
//         self.ticket_queue_tx.clone()
//     }

//     pub async fn ticket_loop(self) {
//         todo!()
//     }
// }

// #[async_trait]
// impl<Db: HoprDbTicketOperations + Send + Sync> TicketManager for HoprTicketManager<Db> {
//     async fn update_ticket_states<'a>(
//         &'a self,
//         selector: TicketSelector,
//         state: AcknowledgedTicketStatus,
//     ) -> Result<BoxStream<'a, AcknowledgedTicket>> {
//         todo!()
//     }

//     async fn count_tickets(&self, selector: TicketSelector) -> Result<usize> {
//         todo!()
//     }

//     async fn get_winning_tickets_in_state<'a>(
//         &'a self,
//         selector: TicketSelector,
//         state: AcknowledgedTicketStatus,
//     ) -> Result<BoxStream<'a, AcknowledgedTicket>> {
//         todo!()
//     }
// }

use futures::{future::BoxFuture, StreamExt};
use sea_orm::{ActiveModelTrait, TransactionTrait};
use std::sync::Arc;
use tracing::error;

use hopr_internal_types::acknowledgement::AcknowledgedTicket;

use crate::{errors::Result, OpenTransaction};

/// Functionlity related to locking and structural improvements to the underlying SQLite database
///
/// With SQLite, it is only possible to have a single write lock per database, meaning that
/// high frequency database access to tickets needed to be split from the rest of the database
/// operations.
///
/// High frequency of locking originating from the ticket processing pipeline could starve the DB,
/// and lock with other concurrent processes, therefore a single mutex for write operations exists,
/// which allows bottlenecking the database write access on the mutex, as well as allowing arbitrary
/// numbers of concurrent read operations.
///
/// The queue based mechanism also splits the storage of the ticket inside the database from the processing,
/// effectively allowing the processing pipelines to be independent from a database write access.
#[derive(Debug, Clone)]
pub(crate) struct TicketManager {
    pub(crate) tickets_db: sea_orm::DatabaseConnection,
    pub(crate) mutex: Arc<async_lock::Mutex<()>>,
    pub(crate) incoming_ack_tickets_tx: futures::channel::mpsc::Sender<AcknowledgedTicket>,
}

impl TicketManager {
    pub fn new(tickets_db: sea_orm::DatabaseConnection) -> Self {
        let (tx, mut rx) = futures::channel::mpsc::channel::<AcknowledgedTicket>(100_000);

        let mutex = Arc::new(async_lock::Mutex::new(()));

        // Creates a process to desynchronize storing of the ticket into the database
        // and the processing calls triggering such an operation.
        let db_clone = tickets_db.clone();
        let mutex_clone = mutex.clone();
        async_std::task::spawn(async move {
            // TODO: it would be beneficial to check the size hint and extract as much, as possible
            // in this step to avoid relocking for each individual ticket.
            while let Some(acknowledged_ticket) = rx.next().await {
                let _guard = mutex_clone.lock().await;
                if let Err(e) = hopr_db_entity::ticket::ActiveModel::from(acknowledged_ticket)
                    .insert(&db_clone)
                    .await
                {
                    error!("failed to insert a winning ticket into the DB: {e}")
                }
            }
        });

        Self {
            tickets_db,
            mutex,
            incoming_ack_tickets_tx: tx,
        }
    }

    pub fn insert_ticket(&self, ticket: AcknowledgedTicket) -> Result<()> {
        self.incoming_ack_tickets_tx.clone().try_send(ticket).map_err(|e| {
            crate::errors::DbError::LogicalError(format!(
                "failed to enqueue acknowledged ticket processing into the DB: {e}"
            ))
        })
    }

    pub async fn with_write_locked_db<'a, F, T, E>(&'a self, f: F) -> std::result::Result<T, E>
    where
        F: for<'c> FnOnce(&'c OpenTransaction) -> BoxFuture<'c, std::result::Result<T, E>> + Send,
        T: Send,
        E: std::error::Error + From<crate::errors::DbError>,
    {
        let mutex = self.mutex.clone();
        let _guard = mutex.lock().await;

        let transaction = OpenTransaction(
            self.tickets_db
                .begin_with_config(None, None)
                .await
                .map_err(crate::errors::DbError::BackendError)?,
            crate::TargetDb::Tickets,
        );

        transaction.perform(f).await
    }
}
