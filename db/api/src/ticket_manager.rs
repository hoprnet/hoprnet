use futures::{future::BoxFuture, StreamExt, TryStreamExt};
use hopr_db_entity::ticket;
use hopr_primitive_types::{
    primitives::{Balance, BalanceType},
    traits::ToHex,
};
use moka::future::Cache;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use sea_query::SimpleExpr;
use std::sync::Arc;
use tracing::error;

use hopr_crypto_types::types::Hash;
use hopr_internal_types::acknowledgement::AcknowledgedTicket;

use crate::{db::ExpiryNever, errors::Result, tickets::TicketSelector, OpenTransaction};

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
    pub(crate) unrealized_value: Cache<Hash, Balance>,
}

impl TicketManager {
    pub fn new(tickets_db: sea_orm::DatabaseConnection) -> Self {
        let unrealized_value = Cache::builder()
            .expire_after(ExpiryNever {})
            .max_capacity(10_000)
            .build();

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
            unrealized_value,
        }
    }

    /// Sends a new acknowledged ticket into the FIFO queue.
    pub async fn insert_ticket(&self, ticket: AcknowledgedTicket) -> Result<()> {
        let channel = ticket.ticket.channel_id;
        let value = ticket.ticket.amount;

        let balance = ticket::Entity::find()
            .filter(hopr_db_entity::ticket::Column::ChannelId.eq(channel.to_hex()))
            .stream(&self.tickets_db)
            .await
            .map_err(crate::errors::DbError::from)?
            .map_err(crate::errors::DbError::from)
            .try_fold(BalanceType::HOPR.zero(), |value, t| async move {
                Ok(value + BalanceType::HOPR.balance_bytes(t.amount))
            })
            .await?;

        self.incoming_ack_tickets_tx.clone().try_send(ticket).map_err(|e| {
            crate::errors::DbError::LogicalError(format!(
                "failed to enqueue acknowledged ticket processing into the DB: {e}"
            ))
        })?;

        Ok(self.unrealized_value.insert(channel, balance + value).await)
    }

    /// Get unrealized value for a channel
    pub async fn unrealized_value(&self, selector: TicketSelector) -> Result<Balance> {
        let transaction = OpenTransaction(
            self.tickets_db
                .begin_with_config(None, None)
                .await
                .map_err(crate::errors::DbError::BackendError)?,
            crate::TargetDb::Tickets,
        );

        Ok(self
            .unrealized_value
            .get_with(selector.channel_id, async move {
                transaction
                    .perform(|tx| {
                        Box::pin(async move {
                            ticket::Entity::find()
                                .filter(SimpleExpr::from(selector))
                                .stream(tx.as_ref())
                                .await
                                .map_err(crate::errors::DbError::from)?
                                .map_err(crate::errors::DbError::from)
                                .try_fold(BalanceType::HOPR.zero(), |value, t| async move {
                                    Ok(value + BalanceType::HOPR.balance_bytes(t.amount))
                                })
                                .await
                        })
                    })
                    .await
                    .unwrap_or_else(|e| {
                        error!("Encountered an error fetching a cached unrealized ticket value: {e}");
                        Balance::zero(BalanceType::HOPR)
                    })
            })
            .await)
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

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;

    use crate::accounts::HoprDbAccountOperations;
    use crate::channels::HoprDbChannelOperations;
    use crate::db::HoprDb;
    use crate::info::{DomainSeparator, HoprDbInfoOperations};

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();
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
                },
            )
            .await?
        }

        Ok(())
    }

    fn generate_random_ack_ticket(index: u32) -> AcknowledgedTicket {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let cp1: CurvePoint = hk1.to_challenge().into();
        let cp2: CurvePoint = hk2.to_challenge().into();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let ticket = Ticket::new(
            &ALICE.public().to_address(),
            &BalanceType::HOPR.balance(TICKET_VALUE),
            index.into(),
            1_u32.into(),
            1.0f64,
            4u64.into(),
            Challenge::from(cp_sum).to_ethereum_challenge(),
            &BOB,
            &Hash::default(),
        )
        .unwrap();

        let unacked_ticket = UnacknowledgedTicket::new(ticket, hk1, BOB.public().to_address());
        unacked_ticket.acknowledge(&hk2, &ALICE, &Hash::default()).unwrap()
    }

    #[async_std::test]
    async fn test_insert_ticket_properly_resolves_the_cached_value(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await;
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
            BalanceType::HOPR.balance(u32::MAX),
            1.into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        db.upsert_channel(None, channel.clone()).await.unwrap();

        assert_eq!(
            Balance::zero(BalanceType::HOPR),
            db.ticket_manager.unrealized_value((&channel).into()).await?
        );

        let ticket = generate_random_ack_ticket(1);
        let ticket_value = ticket.ticket.amount;

        db.ticket_manager.insert_ticket(ticket).await?;

        assert_eq!(
            ticket_value,
            db.ticket_manager.unrealized_value((&channel).into()).await?
        );

        Ok(())
    }
}
