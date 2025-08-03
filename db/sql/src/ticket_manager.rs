use std::sync::{Arc, OnceLock};

use futures::{Sink, SinkExt, StreamExt, TryStreamExt, future::BoxFuture, pin_mut};
use hopr_async_runtime::prelude::spawn;
use hopr_db_api::tickets::TicketSelector;
use hopr_db_entity::ticket;
use hopr_internal_types::{prelude::AcknowledgedTicketStatus, tickets::AcknowledgedTicket};
use hopr_primitive_types::prelude::{HoprBalance, IntoEndian, ToHex};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, TransactionTrait};
use tracing::{debug, error};

use crate::{
    OpenTransaction, cache::HoprDbCaches, errors::Result, prelude::DbSqlError, tickets::WrappedTicketSelector,
};

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
    incoming_ack_tickets_tx: Arc<OnceLock<futures::channel::mpsc::Sender<TicketOperation>>>,
    caches: Arc<HoprDbCaches>,
}

enum TicketOperation {
    /// Inserts a new ticket
    Insert(AcknowledgedTicket),
    /// Replaces multiple tickets (in BeingAggregated state) with the given aggregated ticket.
    Replace(AcknowledgedTicket),
}

impl TicketOperation {
    fn ticket(&self) -> &AcknowledgedTicket {
        match self {
            TicketOperation::Insert(ticket) => ticket,
            TicketOperation::Replace(ticket) => ticket,
        }
    }
}

impl TicketManager {
    pub fn new(tickets_db: sea_orm::DatabaseConnection, caches: Arc<HoprDbCaches>) -> Self {
        Self {
            tickets_db,
            mutex: Arc::new(async_lock::Mutex::new(())),
            incoming_ack_tickets_tx: Arc::new(OnceLock::new()),
            caches,
        }
    }

    /// Must be called to start processing tickets into the DB.
    pub fn start_ticket_processing<S, E>(&self, ticket_notifier: S) -> Result<()>
    where
        S: Sink<AcknowledgedTicket, Error = E> + Send + 'static,
        E: std::error::Error,
    {
        let (tx, mut rx) = futures::channel::mpsc::channel::<TicketOperation>(100_000);

        self.incoming_ack_tickets_tx
            .set(tx)
            .map_err(|_| DbSqlError::LogicalError("ticket processing already started".into()))?;

        // Creates a process to desynchronize storing of the ticket into the database
        // and the processing calls triggering such an operation.
        let db_clone = self.tickets_db.clone();
        let mutex_clone = self.mutex.clone();

        // NOTE: This spawned task does not need to be explicitly canceled, since it will
        // be automatically dropped when the event sender object is dropped.
        spawn(async move {
            pin_mut!(ticket_notifier);
            while let Some(ticket_op) = rx.next().await {
                let ticket_to_insert = ticket_op.ticket().clone();
                let ticket_inserted = match db_clone
                    .begin_with_config(None, None)
                    .await
                    .map_err(DbSqlError::BackendError)
                {
                    Ok(transaction) => {
                        let transaction = OpenTransaction(transaction, crate::TargetDb::Tickets);

                        let _quard = mutex_clone.lock().await;

                        if let Err(error) = transaction
                            .perform(|tx| {
                                Box::pin(async move {
                                    match ticket_op {
                                        // Insertion of a new acknowledged ticket
                                        TicketOperation::Insert(ack_ticket) => {
                                            let channel_id = ack_ticket.verified_ticket().channel_id.to_hex();

                                            hopr_db_entity::ticket::ActiveModel::from(ack_ticket)
                                                .insert(tx.as_ref())
                                                .await?;

                                            // Update the ticket winning count in the statistics
                                            if let Some(model) = hopr_db_entity::ticket_statistics::Entity::find()
                                                .filter(
                                                    hopr_db_entity::ticket_statistics::Column::ChannelId
                                                        .eq(channel_id.clone()),
                                                )
                                                .one(tx.as_ref())
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
                                            .save(tx.as_ref())
                                            .await?;
                                        }
                                        TicketOperation::Replace(ack_ticket) => {
                                            // Replacement range on the aggregated ticket
                                            let start_idx = ack_ticket.verified_ticket().index;
                                            let offset = ack_ticket.verified_ticket().index_offset as u64;

                                            // Replace all BeingAggregated tickets with aggregated index range in this
                                            // channel
                                            let selector = TicketSelector::new(
                                                ack_ticket.verified_ticket().channel_id,
                                                ack_ticket.verified_ticket().channel_epoch,
                                            )
                                            .with_index_range(start_idx..start_idx + offset)
                                            .with_state(AcknowledgedTicketStatus::BeingAggregated);

                                            let deleted = ticket::Entity::delete_many()
                                                .filter(WrappedTicketSelector::from(selector))
                                                .exec(tx.as_ref())
                                                .await?;

                                            if deleted.rows_affected > offset {
                                                return Err(DbSqlError::LogicalError(format!(
                                                    "deleted ticket count ({}) must not be more than the ticket index \
                                                     offset {offset}",
                                                    deleted.rows_affected,
                                                )));
                                            }

                                            ticket::Entity::insert::<ticket::ActiveModel>(ack_ticket.into())
                                                .exec(tx.as_ref())
                                                .await?;
                                        }
                                    }
                                    Ok::<_, DbSqlError>(())
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
        });

        Ok(())
    }

    /// Sends a new acknowledged ticket into the FIFO queue.
    ///
    /// The [`start_ticket_processing`](TicketManager::start_ticket_processing) method
    /// must be called before calling this method, or it will fail.
    pub async fn insert_ticket(&self, ticket: AcknowledgedTicket) -> Result<()> {
        let channel = ticket.verified_ticket().channel_id;
        let value = ticket.verified_ticket().amount;
        let epoch = ticket.verified_ticket().channel_epoch;

        self.incoming_ack_tickets_tx
            .get()
            .ok_or(DbSqlError::LogicalError("ticket processing not started".into()))?
            .clone()
            .try_send(TicketOperation::Insert(ticket))
            .map_err(|e| {
                DbSqlError::LogicalError(format!(
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

    /// Sends aggregated replacement ticket into the FIFO queue.
    ///
    /// The [`start_ticket_processing`](TicketManager::start_ticket_processing) method
    /// must be called before calling this method, or it will fail.
    pub async fn replace_tickets(&self, ticket: AcknowledgedTicket) -> Result<()> {
        self.incoming_ack_tickets_tx
            .get()
            .ok_or(DbSqlError::LogicalError("ticket processing not started".into()))?
            .clone()
            .try_send(TicketOperation::Replace(ticket))
            .map_err(|e| {
                DbSqlError::LogicalError(format!(
                    "failed to enqueue acknowledged ticket processing into the DB: {e}"
                ))
            })
    }

    /// Get unrealized value for a channel
    pub async fn unrealized_value(&self, selector: TicketSelector) -> Result<HoprBalance> {
        if !selector.is_single_channel() {
            return Err(crate::DbSqlError::LogicalError(
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
                OpenTransaction(
                    tickets_db
                        .begin_with_config(None, None)
                        .await
                        .map_err(DbSqlError::BackendError)?,
                    crate::TargetDb::Tickets,
                )
                .perform(|tx| {
                    Box::pin(async move {
                        ticket::Entity::find()
                            .filter(selector_clone)
                            .stream(tx.as_ref())
                            .await
                            .map_err(crate::errors::DbSqlError::from)?
                            .map_err(crate::errors::DbSqlError::from)
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

    /// Acquires write lock to the Ticket DB and starts a new transaction.
    pub async fn with_write_locked_db<'a, F, T, E>(&'a self, f: F) -> std::result::Result<T, E>
    where
        F: for<'c> FnOnce(&'c OpenTransaction) -> BoxFuture<'c, std::result::Result<T, E>> + Send,
        T: Send,
        E: std::error::Error + From<crate::errors::DbSqlError>,
    {
        let mutex = self.mutex.clone();
        let _guard = mutex.lock().await;

        let transaction = OpenTransaction(
            self.tickets_db
                .begin_with_config(None, None)
                .await
                .map_err(crate::errors::DbSqlError::BackendError)?,
            crate::TargetDb::Tickets,
        );

        transaction.perform(f).await
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use hex_literal::hex;
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::prelude::*;
    use hopr_db_api::info::DomainSeparator;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;

    use crate::{
        accounts::HoprDbAccountOperations, channels::HoprDbChannelOperations, db::HoprDb, info::HoprDbInfoOperations,
    };

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).expect("lazy static keypair should be valid");
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).expect("lazy static keypair should be valid");
    }

    lazy_static::lazy_static! {
        static ref ALICE_OFFCHAIN: OffchainKeypair = OffchainKeypair::random();
        static ref BOB_OFFCHAIN: OffchainKeypair = OffchainKeypair::random();
    }

    const TICKET_VALUE: u64 = 100_000;

    async fn add_peer_mappings(db: &HoprDb, peers: Vec<(OffchainKeypair, ChainKeypair)>) -> crate::errors::Result<()> {
        for (peer_offchain, peer_onchain) in peers.into_iter() {
            db.insert_account(
                None,
                AccountEntry {
                    public_key: *peer_offchain.public(),
                    chain_addr: peer_onchain.public().to_address(),
                    entry_type: AccountType::NotAnnounced,
                    published_at: 0,
                },
            )
            .await?
        }

        Ok(())
    }

    fn generate_random_ack_ticket(index: u32) -> anyhow::Result<AcknowledgedTicket> {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let cp1: CurvePoint = hk1.to_challenge().try_into()?;
        let cp2: CurvePoint = hk2.to_challenge().try_into()?;
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let ticket = TicketBuilder::default()
            .direction(&BOB.public().to_address(), &ALICE.public().to_address())
            .amount(TICKET_VALUE)
            .index(index as u64)
            .channel_epoch(4)
            .challenge(Challenge::from(cp_sum).to_ethereum_challenge())
            .build_signed(&BOB, &Hash::default())?;

        Ok(ticket.into_acknowledged(Response::from_half_keys(&hk1, &hk2)?))
    }

    #[tokio::test]
    async fn test_insert_ticket_properly_resolves_the_cached_value() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Hash::default())
            .await?;
        add_peer_mappings(
            &db,
            vec![
                (ALICE_OFFCHAIN.clone(), ALICE.clone()),
                (BOB_OFFCHAIN.clone(), BOB.clone()),
            ],
        )
        .await?;

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            u32::MAX.into(),
            1.into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        db.upsert_channel(None, channel).await?;

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
