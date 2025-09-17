use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, OnceLock},
};

use futures::{Sink, SinkExt, StreamExt, TryStreamExt, channel::mpsc::UnboundedSender, pin_mut};
use hopr_api::db::TicketSelector;
use hopr_async_runtime::prelude::spawn;
use hopr_db_entity::ticket;
use hopr_internal_types::tickets::AcknowledgedTicket;
use hopr_primitive_types::prelude::{HoprBalance, IntoEndian, ToHex};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter, TransactionTrait,
};
use tracing::{debug, error};

use crate::{cache::NodeDbCaches, errors::NodeDbError, db::HoprNodeDb, tickets::WrappedTicketSelector};

/// Functionality related to locking and structural improvements to the underlying SQLite database
///
/// With SQLite, it is only possible to have a single write lock per database, meaning that
/// high-frequency database access to tickets needed to be split from the rest of the database
/// operations.
///
/// High frequency of locking originating from the ticket processing pipeline could starve the DB
/// and lock with other concurrent processes. Therefore, a single mutex for write operations exists,
/// which allows bottle-necking the database write access on the mutex, as well as allowing arbitrary
/// numbers of concurrent read operations.
///
/// The queue-based mechanism also splits the storage of the ticket inside the database from the processing,
/// effectively allowing the processing pipelines to be independent of a database write access.
#[derive(Debug, Clone)]
pub(crate) struct TicketManager {
    pub(crate) tickets_db: sea_orm::DatabaseConnection,
    pub(crate) mutex: Arc<async_lock::Mutex<()>>,
    incoming_ack_tickets_tx: Arc<OnceLock<futures::channel::mpsc::Sender<AcknowledgedTicket>>>,
    caches: Arc<NodeDbCaches>,
}

impl TicketManager {
    pub fn new(tickets_db: sea_orm::DatabaseConnection, caches: Arc<NodeDbCaches>) -> Self {
        Self {
            tickets_db,
            mutex: Arc::new(async_lock::Mutex::new(())),
            incoming_ack_tickets_tx: Arc::new(OnceLock::new()),
            caches,
        }
    }

    /// Must be called to start processing tickets into the DB.
    pub fn start_ticket_processing<S, E>(&self, ticket_notifier: S) -> Result<(), NodeDbError>
    where
        S: Sink<AcknowledgedTicket, Error = E> + Send + 'static,
        E: std::error::Error,
    {
        let (tx, mut rx) = futures::channel::mpsc::channel::<AcknowledgedTicket>(100_000);

        self.incoming_ack_tickets_tx
            .set(tx)
            .map_err(|_| NodeDbError::LogicalError("ticket processing already started".into()))?;

        // Creates a process to desynchronize storing of the ticket into the database
        // and the processing calls triggering such an operation.
        let db_clone = self.tickets_db.clone();
        let mutex_clone = self.mutex.clone();

        // NOTE: This spawned task does not need to be explicitly canceled, since it will
        // be automatically dropped when the event sender object is dropped.
        spawn(async move {
            pin_mut!(ticket_notifier);
            while let Some(ticket_to_insert) = rx.next().await {
                let ticket_inserted = match db_clone.begin_with_config(None, None).await {
                    Ok(transaction) => {
                        let _quard = mutex_clone.lock().await;
                        let ticket_to_insert_clone = ticket_to_insert.clone();
                        if let Err(error) = transaction
                            .transaction(|tx| {
                                Box::pin(async move {
                                    // Insertion of a new acknowledged ticket
                                    let channel_id = ticket_to_insert_clone.verified_ticket().channel_id.to_hex();

                                    hopr_db_entity::ticket::ActiveModel::from(ticket_to_insert_clone)
                                        .insert(tx)
                                        .await?;

                                    // Update the ticket winning count in the statistics
                                    if let Some(model) = hopr_db_entity::ticket_statistics::Entity::find()
                                        .filter(
                                            hopr_db_entity::ticket_statistics::Column::ChannelId.eq(channel_id.clone()),
                                        )
                                        .one(tx)
                                        .await?
                                    {
                                        let winning_tickets = model.winning_tickets + 1;
                                        let mut active_model = model.into_active_model();
                                        active_model.winning_tickets = sea_orm::Set(winning_tickets);
                                        active_model
                                    } else {
                                        hopr_db_entity::ticket_statistics::ActiveModel {
                                            channel_id: sea_orm::Set(channel_id),
                                            winning_tickets: sea_orm::Set(1),
                                            ..Default::default()
                                        }
                                    }
                                    .save(tx)
                                    .await?;
                                    Ok::<_, sea_orm::DbErr>(())
                                })
                            })
                            .await
                        {
                            error!(%error, "failed to insert the winning ticket and update the ticket stats");
                            false
                        } else {
                            debug!(acknowledged_ticket = %ticket_to_insert, "ticket persisted into the ticket db");
                            true
                        }
                    }
                    Err(error) => {
                        error!(%error, "failed to create a transaction for ticket insertion");
                        false
                    }
                };

                // Notify about the ticket once successfully inserted into the Tickets DB
                if ticket_inserted {
                    if let Err(error) = ticket_notifier.send(ticket_to_insert).await {
                        error!(%error, "failed to notify the ticket notifier about the winning ticket");
                    }
                }
            }

            tracing::info!(task = "ticket processing", "long-running background task finished")
        });

        Ok(())
    }

    /// Sends a new acknowledged ticket into the FIFO queue.
    ///
    /// The [`start_ticket_processing`](TicketManager::start_ticket_processing) method
    /// must be called before calling this method, or it will fail.
    pub async fn insert_ticket(&self, ticket: AcknowledgedTicket) -> Result<(), NodeDbError> {
        let channel = ticket.verified_ticket().channel_id;
        let value = ticket.verified_ticket().amount;
        let epoch = ticket.verified_ticket().channel_epoch;

        self.incoming_ack_tickets_tx
            .get()
            .ok_or(NodeDbError::LogicalError("ticket processing not started".into()))?
            .clone()
            .try_send(ticket)
            .map_err(|e| {
                NodeDbError::LogicalError(format!(
                    "failed to enqueue acknowledged ticket processing into the DB: {e}"
                ))
            })?;

        let unrealized_value = self.unrealized_value(TicketSelector::new(channel, epoch)).await?;

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            crate::tickets::METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                &[&channel.to_string(), "unredeemed"],
                (unrealized_value + value).amount().as_u128() as f64,
            );
        }

        self.caches
            .unrealized_value
            .insert((channel, epoch.into()), unrealized_value + value)
            .await;

        Ok(())
    }

    /// Get unrealized value for a channel
    pub async fn unrealized_value(&self, selector: TicketSelector) -> Result<HoprBalance, NodeDbError> {
        if !selector.is_single_channel() {
            return Err(NodeDbError::LogicalError(
                "selector must represent a single channel".into(),
            ));
        }

        let channel_id = selector.channel_identifiers[0].0;
        let channel_epoch = selector.channel_identifiers[0].1;
        let selector: WrappedTicketSelector = selector.into();

        let tickets_db = self.tickets_db.clone();
        let selector_clone = selector.clone();
        Ok(self
            .caches
            .unrealized_value
            .try_get_with_by_ref(&(channel_id, channel_epoch), async move {
                tracing::warn!(%channel_id, %channel_epoch, "cache miss on unrealized value");
                tickets_db
                    .transaction(|tx| {
                        Box::pin(async move {
                            ticket::Entity::find()
                                .filter(selector_clone)
                                .stream(tx)
                                .await?
                                .try_fold(HoprBalance::zero(), |value, t| async move {
                                    Ok(value + HoprBalance::from_be_bytes(t.amount))
                                })
                                .await
                        })
                    })
                    .await
            })
            .await?)
    }

    pub async fn write_transaction<'a, F, T, E>(&self, action: F) -> Result<T, sea_orm::TransactionError<E>>
    where
        F: for<'c> FnOnce(&'c DatabaseTransaction) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>> + Send,
        T: Send,
        E: std::error::Error + Send,
    {
        let _guard = self.mutex.lock().await;
        self.tickets_db.transaction(action).await
    }
}

impl HoprNodeDb {
    /// Starts ticket processing by the `TicketManager` with an optional new ticket notifier.
    /// Without calling this method, tickets will not be persisted into the DB.
    ///
    /// If the notifier is given, it will receive notifications once a new ticket has been
    /// persisted into the Tickets DB.
    pub fn start_ticket_processing(
        &self,
        ticket_notifier: Option<UnboundedSender<AcknowledgedTicket>>,
    ) -> Result<(), NodeDbError> {
        if let Some(notifier) = ticket_notifier {
            self.ticket_manager.start_ticket_processing(notifier)
        } else {
            self.ticket_manager.start_ticket_processing(futures::sink::drain())
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use hex_literal::hex;
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;

    use super::*;
    use crate::db::HoprNodeDb;

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).expect("lazy static keypair should be valid");
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).expect("lazy static keypair should be valid");
        static ref ALICE_OFFCHAIN: OffchainKeypair = OffchainKeypair::random();
        static ref BOB_OFFCHAIN: OffchainKeypair = OffchainKeypair::random();
    }

    const TICKET_VALUE: u64 = 100_000;

    fn generate_random_ack_ticket(index: u32) -> anyhow::Result<AcknowledgedTicket> {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let challenge = Response::from_half_keys(&hk1, &hk2)?.to_challenge()?;

        let ticket = TicketBuilder::default()
            .direction(BOB.public().as_ref(), ALICE.public().as_ref())
            .amount(TICKET_VALUE)
            .index(index as u64)
            .channel_epoch(4)
            .challenge(challenge)
            .build_signed(&BOB, &Hash::default())?;

        Ok(ticket.into_acknowledged(Response::from_half_keys(&hk1, &hk2)?))
    }

    #[tokio::test]
    async fn test_insert_ticket_properly_resolves_the_cached_value() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory(BOB.clone()).await?;

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            u32::MAX.into(),
            1.into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        assert_eq!(
            HoprBalance::zero(),
            db.ticket_manager.unrealized_value((&channel).into()).await?
        );

        let ticket = generate_random_ack_ticket(1)?;
        let ticket_value = ticket.verified_ticket().amount;

        let (tx, mut rx) = futures::channel::mpsc::unbounded();

        db.ticket_manager.start_ticket_processing(tx)?;

        db.ticket_manager.insert_ticket(ticket.clone()).await?;

        assert_eq!(
            ticket_value,
            db.ticket_manager.unrealized_value((&channel).into()).await?
        );

        let recv_ticket = rx.next().await.ok_or(anyhow::anyhow!("no ticket received"))?;
        assert_eq!(recv_ticket, ticket);

        Ok(())
    }
}
