use futures::{future::BoxFuture, StreamExt};
use sea_orm::{ActiveModelTrait, TransactionTrait};
use std::sync::Arc;
use tracing::error;

use hopr_internal_types::acknowledgement::AcknowledgedTicket;

use crate::{errors::Result, OpenTransaction};

pub type WinningTicketSender = futures::channel::mpsc::Sender<AcknowledgedTicket>;

/// Functionality related to locking and structural improvements to the underlying SQLite database
///
/// With SQLite, it is only possible to have a single write lock per database, meaning that
/// high frequency database access to tickets needed to be split from the rest of the database
/// operations.
///
/// High frequency of locking originating from the ticket processing pipeline could starve the DB,
/// and lock with other concurrent processes, therefore a single mutex for write operations exists,
/// which allows bottle-necking the database write access on the mutex, as well as allowing arbitrary
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

    /// Sends a new acknowledged ticket into the FIFO queue.
    pub fn insert_ticket(&self, ticket: AcknowledgedTicket) -> Result<()> {
        self.incoming_ack_tickets_tx.clone().try_send(ticket).map_err(|e| {
            crate::errors::DbError::LogicalError(format!(
                "failed to enqueue acknowledged ticket processing into the DB: {e}"
            ))
        })
    }

    /// Acquires write lock to the Ticket DB and starts a new transaction.
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
