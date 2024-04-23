use async_stream::stream;
use hopr_db_entity::ticket_statistics;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::TryStreamExt;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Set};
use sea_query::{Condition, Expr, IntoCondition};
use tracing::{debug, error, info, trace};

use hopr_crypto_types::prelude::*;
use hopr_db_entity::{outgoing_ticket_index, ticket};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::channels::HoprDbChannelOperations;
use crate::db::HoprDb;
use crate::errors::DbError::LogicalError;
use crate::errors::{DbError, Result};
use crate::info::{DomainSeparator, HoprDbInfoOperations};
use crate::resolver::HoprDbResolverOperations;
use crate::{HoprDbGeneralModelOperations, OpenTransaction, OptTx, TargetDb};

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::MultiGauge;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    pub static ref METRIC_HOPR_TICKETS_INCOMING_STATISTICS: MultiGauge = MultiGauge::new(
        "hopr_tickets_incoming_statistics",
        "Ticket statistics for channels with incoming tickets.",
        &["channel", "statistic"]
    ).unwrap();
}

/// Allows to select multiple tickets (if `index` is `None`)
/// or a single ticket (with given `index`) in the given channel and epoch.
/// The selection can be further restricted to select ticket only in the given `state`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TicketSelector {
    /// Channel ID
    pub channel_id: Hash,
    /// Channel epoch
    pub epoch: U256,
    /// If given, will select single ticket with the given index
    /// in the given channel and epoch.
    pub index: Option<u64>,
    /// Further restriction to tickets with the given state.
    pub state: Option<AcknowledgedTicketStatus>,
    /// Further restrict to only aggregated tickets.
    pub only_aggregated: bool,
}

impl Display for TicketSelector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ticket selector in {} epoch {}{}{}{}",
            self.channel_id,
            self.epoch,
            self.index.map(|idx| format!(" with index {idx}")).unwrap_or("".into()),
            self.state
                .map(|state| format!(" in state {state}"))
                .unwrap_or("".into()),
            if self.only_aggregated { " only aggregated" } else { "" }
        )
    }
}

impl TicketSelector {
    /// Create a new ticket selector given the `channel_id` and `epoch`.
    pub fn new<T: Into<U256>>(channel_id: Hash, epoch: T) -> Self {
        Self {
            channel_id,
            epoch: epoch.into(),
            index: None,
            state: None,
            only_aggregated: false,
        }
    }

    /// If `false` is returned, the selector can fetch more than a single ticket.
    pub fn is_unique(&self) -> bool {
        self.index.is_some()
    }

    /// Returns this instance with ticket index set.
    pub fn with_index(mut self, index: u64) -> Self {
        self.index = Some(index);
        self
    }

    /// Returns this instance with ticket state set.
    pub fn with_state(mut self, state: AcknowledgedTicketStatus) -> Self {
        self.state = Some(state);
        self
    }

    /// Returns this instance without ticket state set.
    pub fn with_no_state(mut self) -> Self {
        self.state = None;
        self
    }

    /// Returns this instance with `only_aggregated` flag value.
    pub fn with_aggregated_only(mut self, only_aggregated: bool) -> Self {
        self.only_aggregated = only_aggregated;
        self
    }
}

impl From<&AcknowledgedTicket> for TicketSelector {
    fn from(value: &AcknowledgedTicket) -> Self {
        Self {
            channel_id: value.verified_ticket().channel_id,
            epoch: value.verified_ticket().channel_epoch.into(),
            index: Some(value.verified_ticket().index),
            state: Some(value.status),
            only_aggregated: value.verified_ticket().index_offset > 1,
        }
    }
}

impl From<&RedeemableTicket> for TicketSelector {
    fn from(value: &RedeemableTicket) -> Self {
        Self {
            channel_id: value.verified_ticket().channel_id,
            epoch: value.verified_ticket().channel_epoch.into(),
            index: Some(value.verified_ticket().index),
            state: None,
            only_aggregated: value.verified_ticket().index_offset > 1,
        }
    }
}

impl From<&ChannelEntry> for TicketSelector {
    fn from(value: &ChannelEntry) -> Self {
        Self {
            channel_id: value.get_id(),
            epoch: value.channel_epoch,
            index: None,
            state: None,
            only_aggregated: false,
        }
    }
}

impl From<ChannelEntry> for TicketSelector {
    fn from(value: ChannelEntry) -> Self {
        TicketSelector::from(&value)
    }
}

impl IntoCondition for TicketSelector {
    fn into_condition(self) -> Condition {
        let mut expr = ticket::Column::ChannelId
            .eq(self.channel_id.to_hex())
            .and(ticket::Column::ChannelEpoch.eq(self.epoch.to_be_bytes().to_vec()));

        if let Some(index) = self.index {
            expr = expr.and(ticket::Column::Index.eq(index.to_be_bytes().to_vec()));
        }

        if let Some(state) = self.state {
            expr = expr.and(ticket::Column::State.eq(state as u8))
        }

        if self.only_aggregated {
            expr = expr.and(ticket::Column::IndexOffset.gt(1));
        }

        expr.into_condition()
    }
}

/// Prerequisites for the ticket aggregator.
/// The prerequisites are **independent** of each other.
/// If none of the prerequisites are given, they are considered satisfied.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AggregationPrerequisites {
    /// Minimum number of tickets in the channel.
    pub min_ticket_count: Option<usize>,
    /// Minimum ratio of balance of unaggregated messages and channel stake.
    /// I.e. the condition is met if sum of unaggregated ticket amounts divided by
    /// the total channel stake is greater than `min_unaggregated_ratio`.
    pub min_unaggregated_ratio: Option<f64>,
}

impl AggregationPrerequisites {
    /// Filters the list of ticket models according to the prerequisites.
    /// **NOTE:** the input list is assumed to be sorted by ticket index in ascending order.
    ///
    /// The following is applied:
    /// - the list of tickets is reduced so that the total amount on the tickets does not exceed the channel balance
    /// - it is checked whether the list size is greater than `min_unaggregated_ratio`
    /// - it is checked whether the ratio of total amount on the unaggregated tickets on the list and the channel balance ratio is greater than `min_unaggregated_ratio`
    fn filter_satisfying_ticket_models(
        self,
        models: Vec<ticket::Model>,
        channel_entry: &ChannelEntry,
    ) -> Result<Vec<ticket::Model>> {
        let channel_id = channel_entry.get_id();

        let mut to_be_aggregated = Vec::with_capacity(models.len());
        let mut total_balance = BalanceType::HOPR.zero();
        let mut unaggregated_balance = BalanceType::HOPR.zero();

        for m in models {
            let to_add = BalanceType::HOPR.balance_bytes(&m.amount);
            // Count only balances of unaggregated tickets
            if m.index_offset == 1 {
                unaggregated_balance = unaggregated_balance.add(to_add);
            }

            // Do a balance check to be sure not to aggregate more than current channel stake
            total_balance = total_balance + to_add;
            if total_balance.gt(&channel_entry.balance) {
                break;
            }

            to_be_aggregated.push(m);
        }

        // If there are no criteria, just send everything for aggregation
        if self.min_ticket_count.is_none() && self.min_unaggregated_ratio.is_none() {
            info!("Aggregation check OK {channel_id}: no aggregation prerequisites were given");
            return Ok(to_be_aggregated);
        }

        let to_be_agg_count = to_be_aggregated.len();

        // Check the aggregation threshold
        if let Some(agg_threshold) = self.min_ticket_count {
            if to_be_agg_count >= agg_threshold {
                info!("Aggregation check OK {channel_id}: {to_be_agg_count} >= {agg_threshold} ack tickets");
                return Ok(to_be_aggregated);
            } else {
                debug!("Aggregation check FAIL {channel_id}: {to_be_agg_count} < {agg_threshold} ack tickets");
            }
        }

        if let Some(unrealized_threshold) = self.min_unaggregated_ratio {
            let diminished_balance = channel_entry.balance.mul_f64(unrealized_threshold)?;

            // Trigger aggregation if unrealized balance greater or equal to X percent of the current balance
            // and there are at least two tickets
            if unaggregated_balance.ge(&diminished_balance) {
                if to_be_agg_count > 1 {
                    info!("Aggregation check OK {channel_id}: unrealized balance {unaggregated_balance} >= {diminished_balance} in {to_be_agg_count} tickets");
                    return Ok(to_be_aggregated);
                } else {
                    debug!("Aggregation check FAIL {channel_id}: unrealized balance {unaggregated_balance} >= {diminished_balance} but in only {to_be_agg_count} tickets");
                }
            } else {
                debug!("Aggregation check FAIL {channel_id}: unrealized balance {unaggregated_balance} < {diminished_balance} in {to_be_agg_count} tickets");
            }
        }

        debug!("Aggregation check FAIL {channel_id}: no prerequisites were met");
        Ok(vec![])
    }
}

#[async_trait]
pub trait HoprDbTicketOperations {
    /// Retrieve acknowledged winning tickets according to the given `selector`.
    /// The optional transaction `tx` must be in the [tickets database](TargetDb::Tickets).
    async fn get_all_tickets(&self) -> Result<Vec<AcknowledgedTicket>>;

    /// Retrieve acknowledged winning tickets according to the given `selector`.
    /// The optional transaction `tx` must be in the [tickets database](TargetDb::Tickets).
    async fn get_tickets<'a>(&'a self, tx: OptTx<'a>, selector: TicketSelector) -> Result<Vec<AcknowledgedTicket>>;

    /// Marks tickets as redeemed (removing them from the DB) and updating the statistics.
    /// Returns the number of tickets that were redeemed.
    async fn mark_tickets_redeemed(&self, selector: TicketSelector) -> Result<usize>;

    /// Marks tickets as redeemed (removing them from the DB) and updating the statistics.
    /// Returns the number of tickets that were neglected.
    async fn mark_tickets_neglected(&self, selector: TicketSelector) -> Result<usize>;

    /// Updates the ticket statistics according to the fact that the given ticket has
    /// been rejected by the packet processing pipeline.
    async fn mark_ticket_rejected(&self, ticket: &Ticket) -> Result<()>;

    /// Updates [state](AcknowledgedTicketStatus) of the tickets matching the given `selector`.
    /// Returns the updated tickets in the new state.
    async fn update_ticket_states_and_fetch<'a>(
        &'a self,
        selector: TicketSelector,
        new_state: AcknowledgedTicketStatus,
    ) -> Result<BoxStream<'a, AcknowledgedTicket>>;

    /// Updates [state](AcknowledgedTicketStatus) of the tickets matching the given `selector`.
    async fn update_ticket_states(
        &self,
        selector: TicketSelector,
        new_state: AcknowledgedTicketStatus,
    ) -> Result<usize>;

    /// Retrieves the ticket statistics for the given channel.
    /// If no channel is given, it retrieves aggregate ticket statistics for all channels.
    ///
    /// The optional transaction `tx` must be in the [tickets database](TargetDb::Tickets).
    async fn get_ticket_statistics<'a>(
        &'a self,
        tx: OptTx<'a>,
        channel_id: Option<Hash>,
    ) -> Result<ChannelTicketStatistics>;

    /// Counts the tickets matching the given `selector` and their total value.
    ///
    /// The optional transaction `tx` must be in the [tickets database](TargetDb::Tickets).
    async fn get_tickets_value<'a>(&'a self, tx: OptTx<'a>, selector: TicketSelector) -> Result<(usize, Balance)>;

    /// Sets the stored outgoing ticket index to `index`, only if the currently stored value
    /// is less than `index`. This ensures the stored value can only be growing.
    /// Returns the old value.
    ///
    /// If the entry is not yet present for the given ID, it is initialized to 0.
    async fn compare_and_set_outgoing_ticket_index(&self, channel_id: Hash, index: u64) -> Result<u64>;

    /// Resets the outgoing ticket index to 0 for the given channel id.
    /// Returns the old value before reset.
    ///
    /// If the entry is not yet present for the given ID, it is initialized to 0.
    async fn reset_outgoing_ticket_index(&self, channel_id: Hash) -> Result<u64>;

    /// Increments the outgoing ticket index in the given channel ID and returns the value before incrementing.
    ///
    /// If the entry is not yet present for the given ID, it is initialized to 0 and incremented.
    async fn increment_outgoing_ticket_index(&self, channel_id: Hash) -> Result<u64>;

    /// Gets the current outgoing ticket index for the given channel id.
    ///
    /// If the entry is not yet present for the given ID, it is initialized to 0.
    async fn get_outgoing_ticket_index(&self, channel_id: Hash) -> Result<Arc<AtomicU64>>;

    /// Compares outgoing ticket indices in the cache with the stored values
    /// and updates the stored value where changed.
    /// Returns the number of updated ticket indices.
    async fn persist_outgoing_ticket_indices(&self) -> Result<usize>;

    /// Prepare a viable collection of tickets to be aggregated.
    ///
    /// Some preconditions for tickets apply. This callback will collect the aggregatable
    /// tickets and marks them as being aggregated.
    async fn prepare_aggregation_in_channel(
        &self,
        channel: &Hash,
        prerequisites: AggregationPrerequisites,
    ) -> Result<Option<(OffchainPublicKey, Vec<TransferableWinningTicket>)>>;

    /// Perform a ticket aggregation rollback in the channel.
    ///
    /// If a ticket aggregation fails, this callback can be invoked to make sure that
    /// resources are properly restored and cleaned up in the database, allowing further
    /// aggregations.
    async fn rollback_aggregation_in_channel(&self, channel: Hash) -> Result<()>;

    /// Replace the aggregated tickets locally with an aggregated ticket from the counterparty.
    async fn process_received_aggregated_ticket(
        &self,
        aggregated_ticket: Ticket,
        chain_keypair: &ChainKeypair,
    ) -> Result<AcknowledgedTicket>;

    /// Performs ticket aggregation as an issuing party of the given tickets.
    async fn aggregate_tickets(
        &self,
        destination: OffchainPublicKey,
        acked_tickets: Vec<TransferableWinningTicket>,
        me: &ChainKeypair,
    ) -> Result<VerifiedTicket>;
}

/// Can contains ticket statistics for a channel or aggregate ticket statistics for all channels.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChannelTicketStatistics {
    pub winning_tickets: u128,
    pub neglected_value: Balance,
    pub redeemed_value: Balance,
    pub unredeemed_value: Balance,
    pub rejected_value: Balance,
}

impl Default for ChannelTicketStatistics {
    fn default() -> Self {
        Self {
            winning_tickets: 0,
            neglected_value: BalanceType::HOPR.zero(),
            redeemed_value: BalanceType::HOPR.zero(),
            unredeemed_value: BalanceType::HOPR.zero(),
            rejected_value: BalanceType::HOPR.zero(),
        }
    }
}

pub(crate) async fn find_stats_for_channel(
    tx: &OpenTransaction,
    channel_id: &Hash,
) -> Result<ticket_statistics::Model> {
    if let Some(model) = ticket_statistics::Entity::find()
        .filter(ticket_statistics::Column::ChannelId.eq(channel_id.to_hex()))
        .one(tx.as_ref())
        .await?
    {
        Ok(model)
    } else {
        let new_stats = ticket_statistics::ActiveModel {
            channel_id: Set(channel_id.to_hex()),
            ..Default::default()
        }
        .insert(tx.as_ref())
        .await?;

        Ok(new_stats)
    }
}

#[async_trait]
impl HoprDbTicketOperations for HoprDb {
    async fn get_all_tickets(&self) -> Result<Vec<AcknowledgedTicket>> {
        self.nest_transaction_in_db(None, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    ticket::Entity::find()
                        .all(tx.as_ref())
                        .await?
                        .into_iter()
                        .map(AcknowledgedTicket::try_from)
                        .collect::<hopr_db_entity::errors::Result<Vec<_>>>()
                        .map_err(DbError::from)
                })
            })
            .await
    }

    async fn get_tickets<'a>(&'a self, tx: OptTx<'a>, selector: TicketSelector) -> Result<Vec<AcknowledgedTicket>> {
        debug!("fetching tickets via {selector}");

        self.nest_transaction_in_db(tx, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    ticket::Entity::find()
                        .filter(selector)
                        .all(tx.as_ref())
                        .await?
                        .into_iter()
                        .map(AcknowledgedTicket::try_from)
                        .collect::<hopr_db_entity::errors::Result<Vec<_>>>()
                        .map_err(DbError::from)
                })
            })
            .await
    }

    async fn mark_tickets_redeemed(&self, selector: TicketSelector) -> Result<usize> {
        let myself = self.clone();
        self.ticket_manager
            .with_write_locked_db(|tx| {
                Box::pin(async move {
                    // Obtain the amount of redeemed tickets and their value
                    let (redeemed_count, redeemed_value) = myself.get_tickets_value(Some(tx), selector).await?;

                    if redeemed_count > 0 {
                        // Delete the redeemed tickets first
                        let deleted = ticket::Entity::delete_many().filter(selector).exec(tx.as_ref()).await?;

                        // Update the stats if successful
                        if deleted.rows_affected == redeemed_count as u64 {
                            let current_stats = find_stats_for_channel(tx, &selector.channel_id).await?;

                            let current_redeemed_value = U256::from_be_bytes(current_stats.redeemed_value.clone());

                            let mut new_stats = current_stats.into_active_model();
                            new_stats.redeemed_value =
                                Set((current_redeemed_value + redeemed_value.amount()).to_be_bytes().into());
                            new_stats.save(tx.as_ref()).await?;

                            #[cfg(all(feature = "prometheus", not(test)))]
                            {
                                METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                                    &[&selector.channel_id.to_string(), "redeemed"],
                                    (current_redeemed_value + redeemed_value.amount()).as_u128() as f64,
                                );
                            }

                            myself.caches.unrealized_value.invalidate(&selector.channel_id).await;
                        } else {
                            return Err(DbError::LogicalError(format!(
                                "could not mark {redeemed_count} ticket as redeemed"
                            )));
                        }
                    }

                    info!(
                        "removed {redeemed_count} of redeemed tickets from channel {}",
                        selector.channel_id
                    );
                    Ok(redeemed_count)
                })
            })
            .await
    }

    async fn mark_tickets_neglected(&self, selector: TicketSelector) -> Result<usize> {
        let myself = self.clone();

        self.ticket_manager
            .with_write_locked_db(|tx| {
                Box::pin(async move {
                    // Obtain the amount of neglected tickets and their value
                    let (neglectable_count, neglectable_value) = myself.get_tickets_value(Some(tx), selector).await?;

                    if neglectable_count > 0 {
                        // Delete the neglectable tickets first
                        let deleted = ticket::Entity::delete_many().filter(selector).exec(tx.as_ref()).await?;

                        // Update the stats if successful
                        if deleted.rows_affected == neglectable_count as u64 {
                            let current_status = find_stats_for_channel(tx, &selector.channel_id).await?;

                            let current_neglected_value = U256::from_be_bytes(current_status.neglected_value.clone());

                            let mut new_stats = current_status.into_active_model();
                            new_stats.neglected_value = Set((current_neglected_value + neglectable_value.amount())
                                .to_be_bytes()
                                .into());
                            new_stats.save(tx.as_ref()).await?;

                            #[cfg(all(feature = "prometheus", not(test)))]
                            {
                                METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                                    &[&selector.channel_id.to_string(), "neglected"],
                                    (current_neglected_value + neglectable_value.amount()).as_u128() as f64,
                                );
                            }

                            // invalidating unrealized balance for the channel
                            myself.caches.unrealized_value.invalidate(&selector.channel_id).await;
                        } else {
                            return Err(DbError::LogicalError(format!(
                                "could not mark {neglectable_count} ticket as neglected"
                            )));
                        }
                    }

                    info!(
                        "removed {neglectable_count} of neglected tickets from channel {}",
                        selector.channel_id
                    );
                    Ok(neglectable_count)
                })
            })
            .await
    }

    async fn mark_ticket_rejected(&self, ticket: &Ticket) -> Result<()> {
        let channel_id = ticket.channel_id;
        let amount = ticket.amount;
        self.ticket_manager
            .with_write_locked_db(|tx| {
                Box::pin(async move {
                    let stats = find_stats_for_channel(tx, &channel_id).await?;

                    let current_rejected_value = U256::from_be_bytes(stats.rejected_value.clone());

                    let mut active_stats = stats.into_active_model();
                    active_stats.rejected_value = Set((current_rejected_value + amount.amount()).to_be_bytes().into());
                    active_stats.save(tx.as_ref()).await?;

                    #[cfg(all(feature = "prometheus", not(test)))]
                    {
                        METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                            &[&channel_id.to_string(), "rejected"],
                            (current_rejected_value + amount.amount()).as_u128() as f64,
                        );
                    }

                    Ok::<(), DbError>(())
                })
            })
            .await
    }

    async fn update_ticket_states_and_fetch<'a>(
        &'a self,
        selector: TicketSelector,
        new_state: AcknowledgedTicketStatus,
    ) -> Result<BoxStream<'a, AcknowledgedTicket>> {
        Ok(Box::pin(stream! {
            match ticket::Entity::find()
                .filter(selector)
                .stream(self.conn(TargetDb::Tickets))
                .await {
                Ok(mut stream) => {
                    while let Ok(Some(ticket)) = stream.try_next().await {
                        let active_ticket = ticket::ActiveModel {
                            id: Set(ticket.id),
                            state: Set(new_state as u8 as i32),
                            ..Default::default()
                        };

                        {
                            let _g = self.ticket_manager.mutex.lock();
                            if let Err(e) = active_ticket.update(self.conn(TargetDb::Tickets)).await {
                                error!("failed to update ticket in the db: {e}");
                            }
                        }

                        match AcknowledgedTicket::try_from(ticket) {
                            Ok(mut ticket) => {
                                // Update the state manually, since we do not want to re-fetch the model after the update
                                ticket.status = new_state;
                                yield ticket
                            },
                            Err(e) => {
                                tracing::error!("failed to decode ticket from the db: {e}");
                            }
                        }
                    }
                },
                Err(e) => tracing::error!("failed open ticket db stream: {e}")
            }
        }))
    }

    async fn update_ticket_states(
        &self,
        selector: TicketSelector,
        new_state: AcknowledgedTicketStatus,
    ) -> Result<usize> {
        self.ticket_manager
            .with_write_locked_db(|tx| {
                Box::pin(async move {
                    let update = ticket::Entity::update_many()
                        .filter(selector)
                        .col_expr(ticket::Column::State, Expr::value(new_state as u8))
                        .exec(tx.as_ref())
                        .await?;
                    Ok::<_, DbError>(update.rows_affected as usize)
                })
            })
            .await
    }

    async fn get_ticket_statistics<'a>(
        &'a self,
        tx: OptTx<'a>,
        channel_id: Option<Hash>,
    ) -> Result<ChannelTicketStatistics> {
        match channel_id {
            None => {
                self.nest_transaction_in_db(tx, TargetDb::Tickets)
                    .await?
                    .perform(|tx| {
                        Box::pin(async move {
                            let unredeemed_value = ticket::Entity::find()
                                .stream(tx.as_ref())
                                .await?
                                .try_fold(U256::zero(), |amount, x| async move {
                                    Ok(amount + U256::from_be_bytes(x.amount))
                                })
                                .await?;

                            let mut all_stats = ticket_statistics::Entity::find()
                                .all(tx.as_ref())
                                .await?
                                .into_iter()
                                .fold(ChannelTicketStatistics::default(), |mut acc, stats| {
                                    acc.neglected_value =
                                        acc.neglected_value + BalanceType::HOPR.balance_bytes(stats.neglected_value);
                                    acc.redeemed_value =
                                        acc.redeemed_value + BalanceType::HOPR.balance_bytes(stats.redeemed_value);
                                    acc.rejected_value =
                                        acc.rejected_value + BalanceType::HOPR.balance_bytes(stats.rejected_value);
                                    acc.winning_tickets += stats.winning_tickets as u128;
                                    acc
                                });

                            all_stats.unredeemed_value = BalanceType::HOPR.balance(unredeemed_value);

                            Ok::<_, DbError>(all_stats)
                        })
                    })
                    .await
            }
            Some(channel) => {
                // We need to make sure the channel exists, to avoid creating
                // stats entry for a non-existing channel
                if self.get_channel_by_id(None, &channel).await?.is_none() {
                    return Err(DbError::ChannelNotFound(channel));
                }

                self.nest_transaction_in_db(tx, TargetDb::Tickets)
                    .await?
                    .perform(|tx| {
                        Box::pin(async move {
                            let stats = find_stats_for_channel(tx, &channel).await?;
                            let unredeemed_value = ticket::Entity::find()
                                .filter(ticket::Column::ChannelId.eq(channel.to_hex()))
                                .stream(tx.as_ref())
                                .await?
                                .try_fold(U256::zero(), |amount, x| async move {
                                    Ok(amount + U256::from_be_bytes(x.amount))
                                })
                                .await?;

                            Ok::<_, DbError>(ChannelTicketStatistics {
                                winning_tickets: stats.winning_tickets as u128,
                                neglected_value: BalanceType::HOPR.balance_bytes(stats.neglected_value),
                                redeemed_value: BalanceType::HOPR.balance_bytes(stats.redeemed_value),
                                unredeemed_value: BalanceType::HOPR.balance(unredeemed_value),
                                rejected_value: BalanceType::HOPR.balance_bytes(stats.rejected_value),
                            })
                        })
                    })
                    .await
            }
        }
    }

    async fn get_tickets_value<'a>(&'a self, tx: OptTx<'a>, selector: TicketSelector) -> Result<(usize, Balance)> {
        Ok(self
            .nest_transaction_in_db(tx, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    ticket::Entity::find()
                        .filter(selector)
                        .stream(tx.as_ref())
                        .await
                        .map_err(DbError::from)?
                        .map_err(DbError::from)
                        .try_fold((0_usize, BalanceType::HOPR.zero()), |(count, value), t| async move {
                            Ok((count + 1, value + BalanceType::HOPR.balance_bytes(t.amount)))
                        })
                        .await
                })
            })
            .await?)
    }

    async fn compare_and_set_outgoing_ticket_index(&self, channel_id: Hash, index: u64) -> Result<u64> {
        let old_value = self
            .get_outgoing_ticket_index(channel_id)
            .await?
            .fetch_max(index, Ordering::SeqCst);

        // TODO: should we hint the persisting mechanism to trigger the flush?
        // if old_value < index {}

        Ok(old_value)
    }

    async fn reset_outgoing_ticket_index(&self, channel_id: Hash) -> Result<u64> {
        let old_value = self
            .get_outgoing_ticket_index(channel_id)
            .await?
            .swap(0, Ordering::SeqCst);

        // TODO: should we hint the persisting mechanism to trigger the flush?
        // if old_value > 0 { }

        Ok(old_value)
    }

    async fn increment_outgoing_ticket_index(&self, channel_id: Hash) -> Result<u64> {
        let old_value = self
            .get_outgoing_ticket_index(channel_id)
            .await?
            .fetch_add(1, Ordering::SeqCst);

        // TODO: should we hint the persisting mechanism to trigger the flush?

        Ok(old_value)
    }

    async fn get_outgoing_ticket_index(&self, channel_id: Hash) -> Result<Arc<AtomicU64>> {
        let tkt_manager = self.ticket_manager.clone();

        self.caches
            .ticket_index
            .try_get_with(channel_id, async move {
                let maybe_index = outgoing_ticket_index::Entity::find()
                    .filter(outgoing_ticket_index::Column::ChannelId.eq(channel_id.to_hex()))
                    .one(&tkt_manager.tickets_db)
                    .await?;

                Ok(Arc::new(AtomicU64::new(match maybe_index {
                    Some(model) => U256::from_be_bytes(model.index).as_u64(),
                    None => {
                        tkt_manager
                            .with_write_locked_db(|tx| {
                                Box::pin(async move {
                                    outgoing_ticket_index::ActiveModel {
                                        channel_id: Set(channel_id.to_hex()),
                                        ..Default::default()
                                    }
                                    .insert(tx.as_ref())
                                    .await?;
                                    Ok::<_, DbError>(())
                                })
                            })
                            .await?;
                        0_u64
                    }
                })))
            })
            .await
            .map_err(|e: Arc<DbError>| LogicalError(format!("failed to retrieve ticket index: {e}")))
    }

    async fn persist_outgoing_ticket_indices(&self) -> Result<usize> {
        let outgoing_indices = outgoing_ticket_index::Entity::find().all(&self.tickets_db).await?;

        let mut updated = 0;
        for index_model in outgoing_indices {
            let channel_id = Hash::from_hex(&index_model.channel_id)?;
            let db_index = U256::from_be_bytes(&index_model.index).as_u64();
            if let Some(cached_index) = self.caches.ticket_index.get(&channel_id).await {
                // Note that the persisted value is always lagging behind the cache,
                // so the fact that the cached index can change between this load
                // storing it in the DB is allowed.
                let cached_index = cached_index.load(Ordering::SeqCst);

                // Store the ticket index in a separate write transaction
                if cached_index > db_index {
                    let mut index_active_model = index_model.into_active_model();
                    index_active_model.index = Set(cached_index.to_be_bytes().to_vec());
                    self.ticket_manager
                        .with_write_locked_db(|wtx| {
                            Box::pin(async move {
                                index_active_model.save(wtx.as_ref()).await?;
                                Ok::<_, DbError>(())
                            })
                        })
                        .await?;

                    debug!("updated ticket index in channel {channel_id} from {db_index} to {cached_index}");
                    updated += 1;
                }
            } else {
                // The value is not yet in the cache, meaning there's low traffic on this
                // channel, so the value has not been yet fetched.
                trace!("channel {channel_id} is in the DB but not yet in the cache.");
            }
        }

        Ok::<_, DbError>(updated)
    }

    async fn prepare_aggregation_in_channel(
        &self,
        channel: &Hash,
        prerequisites: AggregationPrerequisites,
    ) -> Result<Option<(OffchainPublicKey, Vec<TransferableWinningTicket>)>> {
        let myself = self.clone();

        let channel_id = *channel;

        let (channel_entry, peer, domain_separator) =
            self.nest_transaction_in_db(None, TargetDb::Index)
                .await?
                .perform(|tx| {
                    Box::pin(async move {
                        let entry = myself
                            .get_channel_by_id(Some(tx), &channel_id)
                            .await?
                            .ok_or(DbError::ChannelNotFound(channel_id))?;

                        if entry.status == ChannelStatus::Closed {
                            return Err(DbError::LogicalError(format!("channel '{channel_id}' is closed")));
                        } else if entry.direction(&myself.me_onchain) != Some(ChannelDirection::Incoming) {
                            return Err(DbError::LogicalError(format!("channel '{channel_id}' is not incoming")));
                        }

                        let pk = myself
                            .resolve_packet_key(&entry.source)
                            .await?
                            .ok_or(DbError::LogicalError(format!(
                                "peer '{}' has no offchain key record",
                                entry.source
                            )))?;

                        let domain_separator =
                            myself.get_indexer_data(Some(tx)).await?.channels_dst.ok_or_else(|| {
                                crate::errors::DbError::LogicalError("domain separator missing".into())
                            })?;

                        Ok((entry, pk, domain_separator))
                    })
                })
                .await?;

        let myself = self.clone();
        let tickets = self
            .ticket_manager
            .with_write_locked_db(|tx| {
                Box::pin(async move {
                    // verify that no aggregation is in progress in the channel
                    if ticket::Entity::find()
                        .filter(
                            TicketSelector::from(&channel_entry).with_state(AcknowledgedTicketStatus::BeingAggregated),
                        )
                        .one(tx.as_ref())
                        .await?
                        .is_some()
                    {
                        return Err(DbError::LogicalError(format!(
                            "'{channel_entry}' is already being aggregated",
                        )));
                    }

                    // find the index of the last ticket being redeemed
                    let first_idx_to_take = ticket::Entity::find()
                        .filter(
                            TicketSelector::from(&channel_entry).with_state(AcknowledgedTicketStatus::BeingRedeemed),
                        )
                        .order_by_desc(ticket::Column::Index)
                        .one(tx.as_ref())
                        .await?
                        .map(|m| U256::from_be_bytes(m.index).as_u64() + 1)
                        .unwrap_or(0_u64); // go from the lowest possible index of none is found

                    // get the list of all tickets to be aggregated
                    let to_be_aggregated = ticket::Entity::find()
                        .filter(TicketSelector::from(&channel_entry))
                        .filter(ticket::Column::Index.gte(first_idx_to_take.to_be_bytes().to_vec()))
                        .filter(ticket::Column::State.ne(AcknowledgedTicketStatus::BeingAggregated as u8))
                        .order_by_asc(ticket::Column::Index)
                        .all(tx.as_ref())
                        .await?;

                    // Filter the list of tickets according to the prerequisites
                    let to_be_aggregated = prerequisites
                        .filter_satisfying_ticket_models(to_be_aggregated, &channel_entry)?
                        .into_iter()
                        .map(|model| {
                            AcknowledgedTicket::try_from(model)
                                .map_err(DbError::from)
                                .and_then(|ack| {
                                    ack.into_transferable(&myself.chain_key, &domain_separator)
                                        .map_err(DbError::from)
                                })
                        })
                        .collect::<Result<Vec<_>>>()?;

                    // mark all tickets with appropriate characteristics as being aggregated
                    if !to_be_aggregated.is_empty() {
                        let last_idx_to_take = to_be_aggregated.last().unwrap().ticket.index;
                        let marked: sea_orm::UpdateResult = ticket::Entity::update_many()
                            .filter(TicketSelector::from(&channel_entry))
                            .filter(ticket::Column::Index.gte(first_idx_to_take.to_be_bytes().to_vec()))
                            .filter(ticket::Column::Index.lte(last_idx_to_take.to_be_bytes().to_vec()))
                            .filter(ticket::Column::State.ne(AcknowledgedTicketStatus::BeingAggregated as u8))
                            .col_expr(
                                ticket::Column::State,
                                Expr::value(AcknowledgedTicketStatus::BeingAggregated as u8 as i32),
                            )
                            .exec(tx.as_ref())
                            .await?;

                        if marked.rows_affected as usize != to_be_aggregated.len() {
                            return Err(DbError::LogicalError(format!(
                                "expected to mark {}, but was able to mark {}",
                                to_be_aggregated.len(),
                                marked.rows_affected,
                            )));
                        }
                    }

                    debug!(
                        "prepared {} tickets to aggregate in {} ({})",
                        to_be_aggregated.len(),
                        channel_entry.get_id(),
                        channel_entry.channel_epoch,
                    );

                    Ok(to_be_aggregated)
                })
            })
            .await?;

        Ok((!tickets.is_empty()).then_some((peer, tickets)))
    }

    async fn rollback_aggregation_in_channel(&self, channel: Hash) -> Result<()> {
        let channel_entry = self
            .get_channel_by_id(None, &channel)
            .await?
            .ok_or(DbError::ChannelNotFound(channel))?;

        let selector = TicketSelector::from(channel_entry).with_state(AcknowledgedTicketStatus::BeingAggregated);

        let reverted = self
            .update_ticket_states(selector, AcknowledgedTicketStatus::Untouched)
            .await?;

        debug!(
            "rollback happened for ticket aggregation in '{channel}' with {reverted} tickets rolled back as a result",
        );
        Ok(())
    }

    async fn process_received_aggregated_ticket(
        &self,
        aggregated_ticket: Ticket,
        chain_keypair: &ChainKeypair,
    ) -> Result<AcknowledgedTicket> {
        if chain_keypair.public().to_address() != self.me_onchain {
            return Err(DbError::LogicalError(
                "chain key for ticket aggregation does not match the DB public address".into(),
            ));
        }

        let myself = self.clone();
        let channel_id = aggregated_ticket.channel_id;

        let (channel_entry, domain_separator) =
            self.nest_transaction_in_db(None, TargetDb::Index)
                .await?
                .perform(|tx| {
                    Box::pin(async move {
                        let entry = myself
                            .get_channel_by_id(Some(tx), &channel_id)
                            .await?
                            .ok_or(DbError::ChannelNotFound(channel_id))?;

                        if entry.status == ChannelStatus::Closed {
                            return Err(DbError::LogicalError(format!("channel '{channel_id}' is closed")));
                        } else if entry.direction(&myself.me_onchain) != Some(ChannelDirection::Incoming) {
                            return Err(DbError::LogicalError(format!("channel '{channel_id}' is not incoming")));
                        }

                        let domain_separator =
                            myself.get_indexer_data(Some(tx)).await?.channels_dst.ok_or_else(|| {
                                crate::errors::DbError::LogicalError("domain separator missing".into())
                            })?;

                        Ok((entry, domain_separator))
                    })
                })
                .await?;

        // Verify the ticket first
        let aggregated_ticket = aggregated_ticket
            .verify(&channel_entry.source, &domain_separator)
            .map_err(|e| {
                DbError::LogicalError(format!(
                    "failed to verify received aggregated ticket in {channel_id}: {e}"
                ))
            })?;

        // Aggregated tickets always have 100% winning probability
        if aggregated_ticket.win_prob() != 1.0f64 {
            return Err(DbError::LogicalError(
                "Aggregated tickets must have 100% win probability".into(),
            ));
        }

        let acknowledged_tickets = self
            .nest_transaction_in_db(None, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    ticket::Entity::find()
                        .filter(
                            TicketSelector::from(&channel_entry).with_state(AcknowledgedTicketStatus::BeingAggregated),
                        )
                        .all(tx.as_ref())
                        .await
                        .map_err(DbError::BackendError)
                })
            })
            .await?;

        if acknowledged_tickets.is_empty() {
            debug!("Received unexpected aggregated ticket in channel {channel_id}");
            return Err(DbError::LogicalError(format!(
                "failed insert aggregated ticket, because no tickets seem to be aggregated for '{channel_id}'",
            )));
        }

        let stored_value = acknowledged_tickets
            .iter()
            .map(|m| BalanceType::HOPR.balance_bytes(&m.amount))
            .fold(Balance::zero(BalanceType::HOPR), |acc, amount| acc.add(amount));

        // Value of received ticket can be higher (profit for us) but not lower
        if aggregated_ticket.verified_ticket().amount.lt(&stored_value) {
            error!("Aggregated ticket value in '{channel_id}' is lower than sum of stored tickets",);
            return Err(DbError::LogicalError(
                "Value of received aggregated ticket is too low".into(),
            ));
        }

        let acknowledged_tickets = acknowledged_tickets
            .into_iter()
            .map(AcknowledgedTicket::try_from)
            .collect::<hopr_db_entity::errors::Result<Vec<AcknowledgedTicket>>>()?;

        // can be done, because the tickets collection is tested for emptiness before
        let first_stored_ticket = acknowledged_tickets.first().unwrap();

        // calculate the new current ticket index
        #[allow(unused_variables)]
        let current_ticket_index_from_aggregated_ticket =
            U256::from(aggregated_ticket.verified_ticket().index).add(aggregated_ticket.verified_ticket().index_offset);

        let acked_aggregated_ticket = aggregated_ticket.into_acknowledged(first_stored_ticket.response.clone());

        let ticket = acked_aggregated_ticket.clone();
        self.ticket_manager
            .with_write_locked_db(|tx| {
                Box::pin(async move {
                    let deleted = ticket::Entity::delete_many()
                        .filter(
                            TicketSelector::from(channel_entry).with_state(AcknowledgedTicketStatus::BeingAggregated),
                        )
                        .exec(tx.as_ref())
                        .await?;

                    if deleted.rows_affected as usize != acknowledged_tickets.len() {
                        return Err(DbError::LogicalError(format!(
                            "The deleted aggregated ticket count ({}) does not correspond to the expected count: {}",
                            deleted.rows_affected,
                            acknowledged_tickets.len(),
                        )));
                    }

                    ticket::Entity::insert::<ticket::ActiveModel>(ticket.into())
                        .exec(tx.as_ref())
                        .await?;

                    Ok::<(), DbError>(())
                })
            })
            .await?;

        info!("successfully processed received aggregated {acked_aggregated_ticket}");
        Ok(acked_aggregated_ticket)
    }

    async fn aggregate_tickets(
        &self,
        destination: OffchainPublicKey,
        mut acked_tickets: Vec<TransferableWinningTicket>,
        me: &ChainKeypair,
    ) -> Result<VerifiedTicket> {
        if me.public().to_address() != self.me_onchain {
            return Err(DbError::LogicalError(
                "chain key for ticket aggregation does not match the DB public address".into(),
            ));
        }

        let domain_separator = self
            .get_indexer_data(None)
            .await?
            .domain_separator(DomainSeparator::Channel)
            .ok_or_else(|| DbError::LogicalError("domain separator missing".into()))?;

        if acked_tickets.is_empty() {
            return Err(DbError::LogicalError(
                "at least one ticket required for aggregation".to_owned(),
            ));
        }

        if acked_tickets.len() == 1 {
            let single = acked_tickets
                .pop()
                .unwrap()
                .into_redeemable(&self.me_onchain, &domain_separator)?;

            self.compare_and_set_outgoing_ticket_index(
                single.verified_ticket().channel_id,
                single.verified_ticket().index + 1,
            )
            .await?;

            return Ok(single.ticket);
        }

        acked_tickets.sort();
        acked_tickets.dedup();

        let myself = self.clone();

        let (channel_entry, destination) = self
            .nest_transaction_in_db(None, TargetDb::Index)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let address = myself
                        .resolve_chain_key(&destination)
                        .await?
                        .ok_or(DbError::LogicalError(format!(
                            "peer '{}' has no chain key record",
                            destination.to_peerid_str()
                        )))?;

                    let entry = myself
                        .get_channel_by_parties(Some(tx), &myself.me_onchain, &address)
                        .await?
                        .ok_or_else(|| DbError::ChannelNotFound(generate_channel_id(&myself.me_onchain, &address)))?;

                    if entry.status == ChannelStatus::Closed {
                        return Err(DbError::LogicalError(format!("{entry} is closed")));
                    } else if entry.direction(&myself.me_onchain) != Some(ChannelDirection::Outgoing) {
                        return Err(DbError::LogicalError(format!("{entry} is not outgoing")));
                    }

                    Ok((entry, address))
                })
            })
            .await?;

        let channel_balance = channel_entry.balance;
        let channel_epoch = channel_entry.channel_epoch.as_u32();
        let channel_id = channel_entry.get_id();

        let mut final_value = Balance::zero(BalanceType::HOPR);

        // Validate all received tickets and turn them into RedeemableTickets
        let verified_tickets = acked_tickets
            .into_iter()
            .map(|t| t.into_redeemable(&self.me_onchain, &domain_separator))
            .collect::<hopr_internal_types::errors::Result<Vec<_>>>()
            .map_err(|e| {
                DbError::LogicalError(format!("trying to aggregate an invalid or a non-winning ticket: {e}"))
            })?;

        // Perform additional consistency check on the verified tickets
        for (i, acked_ticket) in verified_tickets.iter().enumerate() {
            if channel_id != acked_ticket.verified_ticket().channel_id {
                return Err(DbError::LogicalError(format!(
                    "ticket for aggregation has an invalid channel id {}",
                    acked_ticket.verified_ticket().channel_id
                )));
            }

            if acked_ticket.verified_ticket().channel_epoch != channel_epoch {
                return Err(DbError::LogicalError("channel epochs do not match".into()));
            }

            if i + 1 < verified_tickets.len()
                && acked_ticket.verified_ticket().index + acked_ticket.verified_ticket().index_offset as u64
                    > verified_tickets[i + 1].verified_ticket().index
            {
                return Err(DbError::LogicalError("tickets with overlapping index intervals".into()));
            }

            final_value = final_value.add(&acked_ticket.verified_ticket().amount);
            if final_value.gt(&channel_balance) {
                return Err(DbError::LogicalError(format!("ticket amount to aggregate {final_value} is greater than the balance {channel_balance} of channel {channel_id}")));
            }
        }

        info!(
            "aggregated {} tickets in channel {channel_id} with total value {final_value}",
            verified_tickets.len()
        );

        let first_acked_ticket = verified_tickets.first().unwrap();
        let last_acked_ticket = verified_tickets.last().unwrap();

        // calculate the minimum current ticket index as the larger value from the acked ticket index and on-chain ticket_index from channel_entry
        let current_ticket_index_from_acked_tickets = last_acked_ticket.verified_ticket().index + 1;
        self.compare_and_set_outgoing_ticket_index(channel_id, current_ticket_index_from_acked_tickets)
            .await?;

        Ok(TicketBuilder::default()
            .direction(&self.me_onchain, &destination)
            .balance(final_value)
            .index(first_acked_ticket.verified_ticket().index)
            .index_offset(
                (last_acked_ticket.verified_ticket().index - first_acked_ticket.verified_ticket().index + 1) as u32,
            )
            .win_prob(1.0) // Aggregated tickets have always 100% winning probability
            .channel_epoch(channel_epoch)
            .challenge(first_acked_ticket.verified_ticket().challenge)
            .build_signed(me, &domain_separator)?)
    }
}

impl HoprDb {
    /// Used only by non-SQLite code and tests.
    pub async fn upsert_ticket<'a>(&'a self, tx: OptTx<'a>, acknowledged_ticket: AcknowledgedTicket) -> Result<()> {
        self.nest_transaction_in_db(tx, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    // For purpose of upserting, we must select only by the triplet (channel id, epoch, index)
                    let selector = TicketSelector::new(
                        acknowledged_ticket.verified_ticket().channel_id,
                        acknowledged_ticket.verified_ticket().channel_epoch,
                    )
                    .with_index(acknowledged_ticket.verified_ticket().index);

                    debug!("upserting ticket {acknowledged_ticket}");
                    let mut model = ticket::ActiveModel::from(acknowledged_ticket);

                    if let Some(ticket) = ticket::Entity::find().filter(selector).one(tx.as_ref()).await? {
                        model.id = Set(ticket.id);
                    }

                    Ok::<_, DbError>(model.save(tx.as_ref()).await?)
                })
            })
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use hopr_db_entity::ticket;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, Set};
    use std::ops::Add;
    use std::sync::atomic::Ordering;
    use std::time::{Duration, SystemTime};

    use crate::accounts::HoprDbAccountOperations;
    use crate::channels::HoprDbChannelOperations;
    use crate::db::HoprDb;
    use crate::errors::DbError;
    use crate::info::{DomainSeparator, HoprDbInfoOperations};
    use crate::prelude::ChannelTicketStatistics;
    use crate::tickets::{AggregationPrerequisites, HoprDbTicketOperations, TicketSelector};
    use crate::{HoprDbGeneralModelOperations, TargetDb};

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();
        static ref CHANNEL_ID: Hash = generate_channel_id(&BOB.public().to_address(), &ALICE.public().to_address());
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

    fn generate_random_ack_ticket(src: &ChainKeypair, dst: &ChainKeypair, index: u32) -> AcknowledgedTicket {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let cp1: CurvePoint = hk1.to_challenge().try_into().unwrap();
        let cp2: CurvePoint = hk2.to_challenge().try_into().unwrap();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let ticket = Ticket::new(
            &dst.public().to_address(),
            &BalanceType::HOPR.balance(TICKET_VALUE),
            index.into(),
            1_u32.into(),
            1.0f64,
            4u64.into(),
            Challenge::from(cp_sum).to_ethereum_challenge(),
            src,
            &Hash::default(),
        )
        .unwrap();

        let unacked_ticket = UnacknowledgedTicket::new(ticket, hk1, src.public().to_address());
        unacked_ticket.acknowledge(&hk2, &dst, &Hash::default()).unwrap()
    }

    async fn init_db_with_tickets(db: &HoprDb, count_tickets: u64) -> (ChannelEntry, Vec<AcknowledgedTicket>) {
        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(u32::MAX),
            (count_tickets + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        db.upsert_channel(None, channel).await.unwrap();

        let tickets = (0..count_tickets)
            .into_iter()
            .map(|i| generate_random_ack_ticket(&BOB, &ALICE, i as u32))
            .collect::<Vec<_>>();

        let db_clone = db.clone();
        let tickets_clone = tickets.clone();
        db.begin_transaction_in_db(TargetDb::Tickets)
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    for t in tickets_clone {
                        db_clone.upsert_ticket(Some(tx), t).await?;
                    }
                    Ok::<(), DbError>(())
                })
            })
            .await
            .expect("tx should succeed");

        (channel, tickets)
    }

    #[async_std::test]
    async fn test_insert_get_ticket() {
        let db = HoprDb::new_in_memory(ALICE.clone()).await;
        db.set_domain_separator(None, DomainSeparator::Channel, Hash::default())
            .await
            .unwrap();

        let (channel, mut tickets) = init_db_with_tickets(&db, 1).await;
        let ack_ticket = tickets.pop().unwrap();

        assert_eq!(channel.get_id(), ack_ticket.ticket.channel_id, "channel ids must match");
        assert_eq!(
            channel.channel_epoch.as_u32(),
            ack_ticket.ticket.channel_epoch,
            "epochs must match"
        );

        let db_ticket = db
            .get_tickets(None, (&ack_ticket).into())
            .await
            .expect("should get ticket")
            .first()
            .cloned()
            .expect("ticket should exist");

        assert_eq!(ack_ticket, db_ticket, "tickets must be equal");
    }

    #[async_std::test]
    async fn test_mark_redeemed() {
        let db = HoprDb::new_in_memory(ALICE.clone()).await;
        const COUNT_TICKETS: u64 = 10;

        let (_, tickets) = init_db_with_tickets(&db, COUNT_TICKETS).await;

        let stats = db.get_ticket_statistics(None, None).await.unwrap();
        assert_eq!(
            BalanceType::HOPR.balance(TICKET_VALUE * COUNT_TICKETS),
            stats.unredeemed_value,
            "unredeemed balance must match"
        );
        assert_eq!(
            BalanceType::HOPR.zero(),
            stats.redeemed_value,
            "there must be 0 redeemed value"
        );

        assert_eq!(
            stats,
            db.get_ticket_statistics(None, Some(*CHANNEL_ID)).await.unwrap(),
            "per channel stats must be same"
        );

        const TO_REDEEM: u64 = 2;
        let db_clone = db.clone();
        db.begin_transaction_in_db(TargetDb::Tickets)
            .await
            .unwrap()
            .perform(|_tx| {
                Box::pin(async move {
                    for i in 0..TO_REDEEM as usize {
                        let r = db_clone.mark_tickets_redeemed((&tickets[i]).into()).await?;
                        assert_eq!(1, r, "must redeem only a single ticket");
                    }
                    Ok::<(), DbError>(())
                })
            })
            .await
            .expect("tx must not fail");

        let stats = db.get_ticket_statistics(None, None).await.unwrap();
        assert_eq!(
            BalanceType::HOPR.balance(TICKET_VALUE * (COUNT_TICKETS - TO_REDEEM)),
            stats.unredeemed_value,
            "unredeemed balance must match"
        );
        assert_eq!(
            BalanceType::HOPR.balance(TICKET_VALUE * TO_REDEEM),
            stats.redeemed_value,
            "there must be a redeemed value"
        );

        assert_eq!(
            stats,
            db.get_ticket_statistics(None, Some(*CHANNEL_ID)).await.unwrap(),
            "per channel stats must be same"
        );
    }

    #[async_std::test]
    async fn test_mark_redeem_should_not_mark_redeem_twice() {
        let db = HoprDb::new_in_memory(ALICE.clone()).await;

        let ticket = init_db_with_tickets(&db, 1).await.1.pop().unwrap();

        db.mark_tickets_redeemed((&ticket).into()).await.expect("must not fail");
        assert_eq!(
            0,
            db.mark_tickets_redeemed((&ticket).into()).await.expect("must not fail")
        );
    }

    #[async_std::test]
    async fn test_mark_redeem_should_redeem_all_tickets() {
        let db = HoprDb::new_in_memory(ALICE.clone()).await;

        let count_tickets = 10;
        let channel = init_db_with_tickets(&db, count_tickets).await.0;

        let count_marked = db
            .mark_tickets_redeemed((&channel).into())
            .await
            .expect("must not fail");
        assert_eq!(count_tickets, count_marked as u64, "must mark all tickets in channel");
    }

    #[async_std::test]
    async fn test_mark_tickets_neglected() {
        let db = HoprDb::new_in_memory(ALICE.clone()).await;
        const COUNT_TICKETS: u64 = 10;

        let (channel, _) = init_db_with_tickets(&db, COUNT_TICKETS).await;

        let stats = db.get_ticket_statistics(None, None).await.unwrap();
        assert_eq!(
            BalanceType::HOPR.balance(TICKET_VALUE * COUNT_TICKETS),
            stats.unredeemed_value,
            "unredeemed balance must match"
        );
        assert_eq!(
            BalanceType::HOPR.zero(),
            stats.neglected_value,
            "there must be 0 redeemed value"
        );

        assert_eq!(
            stats,
            db.get_ticket_statistics(None, Some(*CHANNEL_ID)).await.unwrap(),
            "per channel stats must be same"
        );

        db.mark_tickets_neglected((&channel).into())
            .await
            .expect("should mark as neglected");

        let stats = db.get_ticket_statistics(None, None).await.unwrap();
        assert_eq!(
            BalanceType::HOPR.zero(),
            stats.unredeemed_value,
            "unredeemed balance must be zero"
        );
        assert_eq!(
            BalanceType::HOPR.balance(TICKET_VALUE * COUNT_TICKETS),
            stats.neglected_value,
            "there must be a neglected value"
        );

        assert_eq!(
            stats,
            db.get_ticket_statistics(None, Some(*CHANNEL_ID)).await.unwrap(),
            "per channel stats must be same"
        );
    }

    #[async_std::test]
    async fn test_mark_ticket_rejected() {
        let db = HoprDb::new_in_memory(ALICE.clone()).await;

        let (_, mut ticket) = init_db_with_tickets(&db, 1).await;
        let ticket = ticket.pop().unwrap().ticket;

        let stats = db.get_ticket_statistics(None, None).await.unwrap();
        assert_eq!(BalanceType::HOPR.zero(), stats.rejected_value);
        assert_eq!(
            stats,
            db.get_ticket_statistics(None, Some(*CHANNEL_ID)).await.unwrap(),
            "per channel stats must be same"
        );

        db.mark_ticket_rejected(&ticket).await.unwrap();

        let stats = db.get_ticket_statistics(None, None).await.unwrap();
        assert_eq!(ticket.amount, stats.rejected_value);
        assert_eq!(
            stats,
            db.get_ticket_statistics(None, Some(*CHANNEL_ID)).await.unwrap(),
            "per channel stats must be same"
        );
    }

    #[async_std::test]
    async fn test_update_tickets_states_and_fetch() {
        let db = HoprDb::new_in_memory(ALICE.clone()).await;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await
            .unwrap();

        let channel = init_db_with_tickets(&db, 10).await.0;

        let selector = TicketSelector::from(&channel).with_index(5);

        let v: Vec<AcknowledgedTicket> = db
            .update_ticket_states_and_fetch(selector, AcknowledgedTicketStatus::BeingRedeemed)
            .await
            .expect("must create stream")
            .collect()
            .await;

        assert_eq!(1, v.len(), "single ticket must be updated");
        assert_eq!(
            AcknowledgedTicketStatus::BeingRedeemed,
            v.first().unwrap().status,
            "status must be set"
        );

        let selector = TicketSelector::from(&channel).with_state(AcknowledgedTicketStatus::Untouched);

        let v: Vec<AcknowledgedTicket> = db
            .update_ticket_states_and_fetch(selector, AcknowledgedTicketStatus::BeingRedeemed)
            .await
            .expect("must create stream")
            .collect()
            .await;

        assert_eq!(9, v.len(), "only specific tickets must have state set");
        assert!(
            v.iter().all(|t| t.ticket.index != 5),
            "only tickets with different state must update"
        );
        assert!(
            v.iter().all(|t| t.status == AcknowledgedTicketStatus::BeingRedeemed),
            "tickets must have updated state"
        );
    }

    #[async_std::test]
    async fn test_update_tickets_states() {
        let db = HoprDb::new_in_memory(ALICE.clone()).await;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await
            .unwrap();

        let channel = init_db_with_tickets(&db, 10).await.0;
        let selector = TicketSelector::from(&channel).with_state(AcknowledgedTicketStatus::Untouched);

        db.update_ticket_states(selector, AcknowledgedTicketStatus::BeingRedeemed)
            .await
            .unwrap();

        let v: Vec<AcknowledgedTicket> = db
            .update_ticket_states_and_fetch(selector, AcknowledgedTicketStatus::BeingRedeemed)
            .await
            .expect("must create stream")
            .collect()
            .await;

        assert!(v.is_empty(), "must not update if already updated");
    }

    #[async_std::test]
    async fn test_ticket_index_should_be_zero_if_not_yet_present() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let hash = Hash::default();

        let idx = db.get_outgoing_ticket_index(hash).await.unwrap();
        assert_eq!(0, idx.load(Ordering::SeqCst), "initial index must be zero");

        let r = hopr_db_entity::outgoing_ticket_index::Entity::find()
            .filter(hopr_db_entity::outgoing_ticket_index::Column::ChannelId.eq(hash.to_hex()))
            .one(&db.tickets_db)
            .await
            .unwrap()
            .expect("index must exist");

        assert_eq!(0, U256::from_be_bytes(r.index).as_u64(), "index must be zero");
    }

    #[async_std::test]
    async fn test_ticket_stats_must_fail_for_non_existing_channel() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        db.get_ticket_statistics(None, Some(*CHANNEL_ID))
            .await
            .expect_err("must fail for non-existing channel");
    }

    #[async_std::test]
    async fn test_ticket_stats_must_be_zero_when_no_tickets() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(u32::MAX),
            0.into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        db.upsert_channel(None, channel).await.unwrap();

        let stats = db.get_ticket_statistics(None, Some(*CHANNEL_ID)).await.unwrap();

        assert_eq!(
            ChannelTicketStatistics::default(),
            stats,
            "must be equal to default which is all zeros"
        );

        assert_eq!(
            stats,
            db.get_ticket_statistics(None, None).await.unwrap(),
            "per-channel stats must be the same as global stats"
        )
    }

    #[async_std::test]
    async fn test_ticket_stats_must_be_different_per_channel() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let channel_1 = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(u32::MAX),
            0.into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        db.upsert_channel(None, channel_1).await.unwrap();

        let channel_2 = ChannelEntry::new(
            ALICE.public().to_address(),
            BOB.public().to_address(),
            BalanceType::HOPR.balance(u32::MAX),
            0.into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        db.upsert_channel(None, channel_2).await.unwrap();

        let t1 = generate_random_ack_ticket(&BOB, &ALICE, 1);
        let t2 = generate_random_ack_ticket(&ALICE, &BOB, 1);

        let value = t1.ticket.amount;

        db.upsert_ticket(None, t1).await.unwrap();
        db.upsert_ticket(None, t2).await.unwrap();

        let stats_1 = db
            .get_ticket_statistics(
                None,
                Some(generate_channel_id(
                    &BOB.public().to_address(),
                    &ALICE.public().to_address(),
                )),
            )
            .await
            .unwrap();

        let stats_2 = db
            .get_ticket_statistics(
                None,
                Some(generate_channel_id(
                    &ALICE.public().to_address(),
                    &BOB.public().to_address(),
                )),
            )
            .await
            .unwrap();

        assert_eq!(value, stats_1.unredeemed_value);
        assert_eq!(value, stats_2.unredeemed_value);

        assert_eq!(BalanceType::HOPR.zero(), stats_1.neglected_value);
        assert_eq!(BalanceType::HOPR.zero(), stats_2.neglected_value);

        assert_eq!(stats_1, stats_2);

        db.mark_tickets_neglected(channel_1.into()).await.unwrap();

        let stats_1 = db
            .get_ticket_statistics(
                None,
                Some(generate_channel_id(
                    &BOB.public().to_address(),
                    &ALICE.public().to_address(),
                )),
            )
            .await
            .unwrap();

        let stats_2 = db
            .get_ticket_statistics(
                None,
                Some(generate_channel_id(
                    &ALICE.public().to_address(),
                    &BOB.public().to_address(),
                )),
            )
            .await
            .unwrap();

        assert_eq!(BalanceType::HOPR.zero(), stats_1.unredeemed_value);
        assert_eq!(value, stats_1.neglected_value);

        assert_eq!(BalanceType::HOPR.zero(), stats_2.neglected_value);
    }

    #[async_std::test]
    async fn test_ticket_index_compare_and_set_and_increment() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let hash = Hash::default();

        let old_idx = db.compare_and_set_outgoing_ticket_index(hash, 1).await.unwrap();
        assert_eq!(0, old_idx, "old value must be 0");

        let new_idx = db.get_outgoing_ticket_index(hash).await.unwrap().load(Ordering::SeqCst);
        assert_eq!(1, new_idx, "new value must be 1");

        let old_idx = db.increment_outgoing_ticket_index(hash).await.unwrap();
        assert_eq!(1, old_idx, "old value must be 1");

        let new_idx = db.get_outgoing_ticket_index(hash).await.unwrap().load(Ordering::SeqCst);
        assert_eq!(2, new_idx, "new value must be 2");
    }

    #[async_std::test]
    async fn test_ticket_index_compare_and_set_must_not_decrease() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let hash = Hash::default();

        let new_idx = db.get_outgoing_ticket_index(hash).await.unwrap().load(Ordering::SeqCst);
        assert_eq!(0, new_idx, "value must be 0");

        db.compare_and_set_outgoing_ticket_index(hash, 1).await.unwrap();

        let new_idx = db.get_outgoing_ticket_index(hash).await.unwrap().load(Ordering::SeqCst);
        assert_eq!(1, new_idx, "new value must be 1");

        let old_idx = db.compare_and_set_outgoing_ticket_index(hash, 0).await.unwrap();
        assert_eq!(1, old_idx, "old value must be 1");
        assert_eq!(1, new_idx, "new value must be 1");

        let old_idx = db.compare_and_set_outgoing_ticket_index(hash, 1).await.unwrap();
        assert_eq!(1, old_idx, "old value must be 1");
        assert_eq!(1, new_idx, "new value must be 1");
    }

    #[async_std::test]
    async fn test_ticket_index_reset() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let hash = Hash::default();

        let new_idx = db.get_outgoing_ticket_index(hash).await.unwrap().load(Ordering::SeqCst);
        assert_eq!(0, new_idx, "value must be 0");

        db.compare_and_set_outgoing_ticket_index(hash, 1).await.unwrap();

        let new_idx = db.get_outgoing_ticket_index(hash).await.unwrap().load(Ordering::SeqCst);
        assert_eq!(1, new_idx, "new value must be 1");

        let old_idx = db.reset_outgoing_ticket_index(hash).await.unwrap();
        assert_eq!(1, old_idx, "old value must be 1");

        let new_idx = db.get_outgoing_ticket_index(hash).await.unwrap().load(Ordering::SeqCst);
        assert_eq!(0, new_idx, "new value must be 0");
    }

    #[async_std::test]
    async fn test_persist_ticket_indices() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let hash_1 = Hash::default();
        let hash_2 = Hash::from(hopr_crypto_random::random_bytes());

        db.get_outgoing_ticket_index(hash_1).await.unwrap();
        db.compare_and_set_outgoing_ticket_index(hash_2, 10).await.unwrap();

        let persisted = db.persist_outgoing_ticket_indices().await.unwrap();
        assert_eq!(1, persisted);

        let indices = hopr_db_entity::outgoing_ticket_index::Entity::find()
            .all(&db.tickets_db)
            .await
            .unwrap();
        let idx_1 = indices
            .iter()
            .find(|idx| idx.channel_id == hash_1.to_hex())
            .expect("must contain index 1");
        let idx_2 = indices
            .iter()
            .find(|idx| idx.channel_id == hash_2.to_hex())
            .expect("must contain index 2");
        assert_eq!(0, U256::from_be_bytes(&idx_1.index).as_u64(), "index must be 0");
        assert_eq!(10, U256::from_be_bytes(&idx_2.index).as_u64(), "index must be 10");

        db.compare_and_set_outgoing_ticket_index(hash_1, 3).await.unwrap();
        db.increment_outgoing_ticket_index(hash_2).await.unwrap();

        let persisted = db.persist_outgoing_ticket_indices().await.unwrap();
        assert_eq!(2, persisted);

        let indices = hopr_db_entity::outgoing_ticket_index::Entity::find()
            .all(&db.tickets_db)
            .await
            .unwrap();
        let idx_1 = indices
            .iter()
            .find(|idx| idx.channel_id == hash_1.to_hex())
            .expect("must contain index 1");
        let idx_2 = indices
            .iter()
            .find(|idx| idx.channel_id == hash_2.to_hex())
            .expect("must contain index 2");
        assert_eq!(3, U256::from_be_bytes(&idx_1.index).as_u64(), "index must be 3");
        assert_eq!(11, U256::from_be_bytes(&idx_2.index).as_u64(), "index must be 11");
    }

    #[async_std::test]
    async fn test_cache_can_be_cloned_but_referencing_the_original_cache_storage() {
        let cache: moka::future::Cache<i64, i64> = moka::future::Cache::new(5);

        assert_eq!(cache.weighted_size(), 0);

        cache.insert(1, 1).await;
        cache.insert(2, 2).await;

        let clone = cache.clone();

        cache.remove(&1).await;
        cache.remove(&2).await;

        assert_eq!(cache.get(&1).await, None);
        assert_eq!(cache.get(&1).await, clone.get(&1).await);
    }

    fn dummy_ticket_model(channel_id: Hash, idx: u64, idx_offset: u32, amount: u32) -> ticket::Model {
        ticket::Model {
            id: 0,
            channel_id: channel_id.to_string(),
            amount: U256::from(amount).to_be_bytes().to_vec(),
            index: idx.to_be_bytes().to_vec(),
            index_offset: idx_offset as i32,
            winning_probability: vec![],
            channel_epoch: vec![],
            signature: vec![],
            response: vec![],
            state: 0,
        }
    }

    #[async_std::test]
    async fn test_aggregation_prerequisites_default_filter_no_tickets() {
        let prerequisites = AggregationPrerequisites::default();
        assert_eq!(None, prerequisites.min_unaggregated_ratio);
        assert_eq!(None, prerequisites.min_ticket_count);

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(u32::MAX),
            2.into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets = vec![dummy_ticket_model(channel.get_id(), 1, 1, 1)];

        let filtered_tickets = prerequisites
            .filter_satisfying_ticket_models(dummy_tickets.clone(), &channel)
            .expect("must filter tickets");

        assert_eq!(
            dummy_tickets, filtered_tickets,
            "empty prerequisites must not filter anything"
        );
    }

    #[async_std::test]
    async fn test_aggregation_prerequisites_must_trim_tickets_exceeding_channel_balance() {
        const TICKET_COUNT: usize = 110;

        let prerequisites = AggregationPrerequisites::default();
        assert_eq!(None, prerequisites.min_unaggregated_ratio);
        assert_eq!(None, prerequisites.min_ticket_count);

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(100),
            (TICKET_COUNT + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets: Vec<ticket::Model> = (0..TICKET_COUNT)
            .into_iter()
            .map(|i| dummy_ticket_model(channel.get_id(), i as u64, 1, 1))
            .collect();

        let filtered_tickets = prerequisites
            .filter_satisfying_ticket_models(dummy_tickets.clone(), &channel)
            .expect("must filter tickets");

        assert_eq!(
            100,
            filtered_tickets.len(),
            "must take only tickets up to channel balance"
        );
        assert!(
            filtered_tickets
                .into_iter()
                .map(|t| U256::from_be_bytes(t.amount).as_u64())
                .sum::<u64>()
                <= channel.balance.amount().as_u64(),
            "filtered tickets must not exceed channel balance"
        );
    }

    #[async_std::test]
    async fn test_aggregation_prerequisites_must_return_empty_when_minimum_ticket_count_not_met() {
        const TICKET_COUNT: usize = 10;

        let prerequisites = AggregationPrerequisites {
            min_ticket_count: Some(TICKET_COUNT + 1),
            min_unaggregated_ratio: None,
        };

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(100),
            (TICKET_COUNT + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets: Vec<ticket::Model> = (0..TICKET_COUNT)
            .into_iter()
            .map(|i| dummy_ticket_model(channel.get_id(), i as u64, 1, 1))
            .collect();

        let filtered_tickets = prerequisites
            .filter_satisfying_ticket_models(dummy_tickets.clone(), &channel)
            .expect("must filter tickets");

        assert!(
            filtered_tickets.is_empty(),
            "must return empty when min_ticket_count is not met"
        )
    }

    #[async_std::test]
    async fn test_aggregation_prerequisites_must_return_empty_when_minimum_unaggregated_ratio_is_not_met() {
        const TICKET_COUNT: usize = 10;

        let prerequisites = AggregationPrerequisites {
            min_ticket_count: None,
            min_unaggregated_ratio: Some(0.9),
        };

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(100),
            (TICKET_COUNT + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets: Vec<ticket::Model> = (0..TICKET_COUNT)
            .into_iter()
            .map(|i| dummy_ticket_model(channel.get_id(), i as u64, 1, 1))
            .collect();

        let filtered_tickets = prerequisites
            .filter_satisfying_ticket_models(dummy_tickets.clone(), &channel)
            .expect("must filter tickets");

        assert!(
            filtered_tickets.is_empty(),
            "must return empty when min_unaggregated_ratio is not met"
        )
    }

    #[async_std::test]
    async fn test_aggregation_prerequisites_must_return_all_when_minimum_ticket_count_is_met() {
        const TICKET_COUNT: usize = 10;

        let prerequisites = AggregationPrerequisites {
            min_ticket_count: Some(TICKET_COUNT),
            min_unaggregated_ratio: None,
        };

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(100),
            (TICKET_COUNT + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets: Vec<ticket::Model> = (0..TICKET_COUNT)
            .into_iter()
            .map(|i| dummy_ticket_model(channel.get_id(), i as u64, 1, 1))
            .collect();

        let filtered_tickets = prerequisites
            .filter_satisfying_ticket_models(dummy_tickets.clone(), &channel)
            .expect("must filter tickets");

        assert!(!filtered_tickets.is_empty(), "must not return empty");
        assert_eq!(dummy_tickets, filtered_tickets, "return all tickets");
    }

    #[async_std::test]
    async fn test_aggregation_prerequisites_must_return_all_when_minimum_ticket_count_is_met_regardless_ratio() {
        const TICKET_COUNT: usize = 10;

        let prerequisites = AggregationPrerequisites {
            min_ticket_count: Some(TICKET_COUNT),
            min_unaggregated_ratio: Some(0.9),
        };

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(100),
            (TICKET_COUNT + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets: Vec<ticket::Model> = (0..TICKET_COUNT)
            .into_iter()
            .map(|i| dummy_ticket_model(channel.get_id(), i as u64, 1, 1))
            .collect();

        let filtered_tickets = prerequisites
            .filter_satisfying_ticket_models(dummy_tickets.clone(), &channel)
            .expect("must filter tickets");

        assert!(!filtered_tickets.is_empty(), "must not return empty");
        assert_eq!(dummy_tickets, filtered_tickets, "return all tickets");
    }

    #[async_std::test]
    async fn test_aggregation_prerequisites_must_return_all_when_minimum_unaggregated_ratio_is_met() {
        const TICKET_COUNT: usize = 90;

        let prerequisites = AggregationPrerequisites {
            min_ticket_count: None,
            min_unaggregated_ratio: Some(0.9),
        };

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(100),
            (TICKET_COUNT + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets: Vec<ticket::Model> = (0..TICKET_COUNT)
            .into_iter()
            .map(|i| dummy_ticket_model(channel.get_id(), i as u64, 1, 1))
            .collect();

        let filtered_tickets = prerequisites
            .filter_satisfying_ticket_models(dummy_tickets.clone(), &channel)
            .expect("must filter tickets");

        assert!(!filtered_tickets.is_empty(), "must not return empty");
        assert_eq!(dummy_tickets, filtered_tickets, "return all tickets");
    }

    #[async_std::test]
    async fn test_aggregation_prerequisites_must_return_all_when_minimum_unaggregated_ratio_is_met_regardless_count() {
        const TICKET_COUNT: usize = 90;

        let prerequisites = AggregationPrerequisites {
            min_ticket_count: Some(TICKET_COUNT + 1),
            min_unaggregated_ratio: Some(0.9),
        };

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(100),
            (TICKET_COUNT + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets: Vec<ticket::Model> = (0..TICKET_COUNT)
            .into_iter()
            .map(|i| dummy_ticket_model(channel.get_id(), i as u64, 1, 1))
            .collect();

        let filtered_tickets = prerequisites
            .filter_satisfying_ticket_models(dummy_tickets.clone(), &channel)
            .expect("must filter tickets");

        assert!(!filtered_tickets.is_empty(), "must not return empty");
        assert_eq!(dummy_tickets, filtered_tickets, "return all tickets");
    }

    #[async_std::test]
    async fn test_aggregation_prerequisites_must_return_empty_when_minimum_only_aggregated_ratio_is_met() {
        const TICKET_COUNT: usize = 90;

        let prerequisites = AggregationPrerequisites {
            min_ticket_count: None,
            min_unaggregated_ratio: Some(0.9),
        };

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(100),
            (TICKET_COUNT + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let mut dummy_tickets: Vec<ticket::Model> = (0..TICKET_COUNT)
            .into_iter()
            .map(|i| dummy_ticket_model(channel.get_id(), i as u64, 1, 1))
            .collect();
        dummy_tickets[0].index_offset = 2; // Make this ticket aggregated

        let filtered_tickets = prerequisites
            .filter_satisfying_ticket_models(dummy_tickets.clone(), &channel)
            .expect("must filter tickets");

        assert!(filtered_tickets.is_empty(), "must return empty");
    }

    #[async_std::test]
    async fn test_aggregation_prerequisites_must_return_empty_when_minimum_only_unaggregated_ratio_is_met_in_single_ticket_only(
    ) {
        let prerequisites = AggregationPrerequisites {
            min_ticket_count: None,
            min_unaggregated_ratio: Some(0.9),
        };

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(100),
            2.into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        // Single aggregated ticket exceeding the min_unaggregated_ratio
        let dummy_tickets = vec![dummy_ticket_model(channel.get_id(), 1, 2, 110)];

        let filtered_tickets = prerequisites
            .filter_satisfying_ticket_models(dummy_tickets.clone(), &channel)
            .expect("must filter tickets");

        assert!(filtered_tickets.is_empty(), "must return empty");
    }

    async fn create_alice_db_with_tickets_from_bob(
        ticket_count: usize,
    ) -> crate::errors::Result<(HoprDb, ChannelEntry, Vec<AcknowledgedTicket>)> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await;

        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        add_peer_mappings(
            &db,
            vec![
                (ALICE_OFFCHAIN.clone(), ALICE.clone()),
                (BOB_OFFCHAIN.clone(), BOB.clone()),
            ],
        )
        .await?;

        let (channel, tickets) = init_db_with_tickets(&db, ticket_count as u64).await;

        Ok((db, channel, tickets))
    }

    #[async_std::test]
    async fn test_ticket_aggregation_should_fail_if_any_ticket_is_being_aggregated_in_that_channel(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _ = env_logger::builder().is_test(true).try_init();

        const COUNT_TICKETS: usize = 5;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        // mark the first ticket as being aggregated
        let mut ticket = hopr_db_entity::ticket::Entity::find()
            .one(&db.tickets_db)
            .await?
            .unwrap()
            .into_active_model();
        ticket.state = Set(AcknowledgedTicketStatus::BeingAggregated as u8 as i32);
        ticket.save(&db.tickets_db).await?;

        assert!(db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
            .await
            .is_err());

        Ok(())
    }

    #[async_std::test]
    async fn test_ticket_aggregation_prepare_request_with_0_tickets_should_return_empty_result(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        const COUNT_TICKETS: usize = 0;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_0_tickets = channel.get_id();

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_0_tickets, Default::default())
            .await?;

        assert_eq!(actual, None);

        Ok(())
    }

    #[async_std::test]
    async fn test_ticket_aggregation_prepare_request_with_multiple_tickets_should_return_that_ticket(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _ = env_logger::builder().is_test(true).try_init();

        const COUNT_TICKETS: usize = 2;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
            .await?;

        assert_eq!(actual, Some((BOB_OFFCHAIN.public().clone(), tickets)));

        let actual_being_aggregated_count = hopr_db_entity::ticket::Entity::find()
            .filter(hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .count(&db.tickets_db)
            .await? as usize;

        assert_eq!(actual_being_aggregated_count, COUNT_TICKETS);

        Ok(())
    }

    #[async_std::test]
    async fn test_ticket_aggregation_prepare_request_with_a_being_redeemed_ticket_should_aggregate_only_the_tickets_following_it(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _ = env_logger::builder().is_test(true).try_init();

        const COUNT_TICKETS: usize = 5;

        let (db, channel, mut tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        // mark the first ticket as being redeemed
        let mut ticket = hopr_db_entity::ticket::Entity::find()
            .one(&db.tickets_db)
            .await?
            .unwrap()
            .into_active_model();
        ticket.state = Set(AcknowledgedTicketStatus::BeingRedeemed as u8 as i32);
        ticket.save(&db.tickets_db).await?;

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
            .await?;

        tickets.remove(0);
        assert_eq!(actual, Some((BOB_OFFCHAIN.public().clone(), tickets)));

        let actual_being_aggregated_count = hopr_db_entity::ticket::Entity::find()
            .filter(hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .count(&db.tickets_db)
            .await? as usize;

        assert_eq!(actual_being_aggregated_count, COUNT_TICKETS - 1);

        Ok(())
    }

    #[async_std::test]
    async fn test_ticket_aggregation_prepare_request_with_some_requirements_should_return_when_ticket_threshold_is_met(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        const COUNT_TICKETS: usize = 5;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let constraints = AggregationPrerequisites {
            min_ticket_count: Some(COUNT_TICKETS - 1),
            min_unaggregated_ratio: None,
        };
        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, constraints)
            .await?;

        assert_eq!(actual, Some((BOB_OFFCHAIN.public().clone(), tickets)));

        let actual_being_aggregated_count = hopr_db_entity::ticket::Entity::find()
            .filter(hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .count(&db.tickets_db)
            .await? as usize;

        assert_eq!(actual_being_aggregated_count, COUNT_TICKETS);

        Ok(())
    }

    #[async_std::test]
    async fn test_ticket_aggregation_prepare_request_with_some_requirements_should_not_return_when_ticket_threshold_is_not_met(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        const COUNT_TICKETS: usize = 2;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let constraints = AggregationPrerequisites {
            min_ticket_count: Some(COUNT_TICKETS + 1),
            min_unaggregated_ratio: None,
        };
        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, constraints)
            .await?;

        assert_eq!(actual, None);

        let actual_being_aggregated_count = hopr_db_entity::ticket::Entity::find()
            .filter(hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .count(&db.tickets_db)
            .await? as usize;

        assert_eq!(actual_being_aggregated_count, 0);

        Ok(())
    }

    #[async_std::test]
    async fn test_ticket_aggregation_prepare_request_with_no_aggregatable_tickets_should_return_nothing(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        const COUNT_TICKETS: usize = 3;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        assert_eq!(tickets.len(), COUNT_TICKETS as usize);

        let existing_channel_with_multiple_tickets = channel.get_id();

        // mark all tickets as being redeemed
        for ticket in hopr_db_entity::ticket::Entity::find()
            .all(&db.tickets_db)
            .await?
            .into_iter()
        {
            let mut ticket = ticket.into_active_model();
            ticket.state = Set(AcknowledgedTicketStatus::BeingRedeemed as u8 as i32);
            ticket.save(&db.tickets_db).await?;
        }

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
            .await?;

        assert_eq!(actual, None);

        let actual_being_aggregated_count = hopr_db_entity::ticket::Entity::find()
            .filter(hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .count(&db.tickets_db)
            .await? as usize;

        assert_eq!(actual_being_aggregated_count, 0);

        Ok(())
    }

    #[async_std::test]
    async fn test_ticket_aggregation_rollback_should_rollback_all_the_being_aggregated_tickets_but_nothing_else(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _ = env_logger::builder().is_test(true).try_init();

        const COUNT_TICKETS: usize = 5;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        // mark the first ticket as being redeemed
        let mut ticket = hopr_db_entity::ticket::Entity::find()
            .one(&db.tickets_db)
            .await?
            .unwrap()
            .into_active_model();
        ticket.state = Set(AcknowledgedTicketStatus::BeingRedeemed as u8 as i32);
        ticket.save(&db.tickets_db).await?;

        assert!(db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
            .await
            .is_ok());

        let actual_being_aggregated_count = hopr_db_entity::ticket::Entity::find()
            .filter(hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .count(&db.tickets_db)
            .await? as usize;

        assert_eq!(actual_being_aggregated_count, COUNT_TICKETS - 1);

        assert!(db
            .rollback_aggregation_in_channel(existing_channel_with_multiple_tickets)
            .await
            .is_ok());

        let actual_being_aggregated_count = hopr_db_entity::ticket::Entity::find()
            .filter(hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .count(&db.tickets_db)
            .await? as usize;

        assert_eq!(actual_being_aggregated_count, 0);

        Ok(())
    }

    #[async_std::test]
    async fn test_ticket_aggregation_should_replace_the_tickets_with_a_correctly_aggregated_ticket(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        const COUNT_TICKETS: usize = 5;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        let aggregated_ticket = TicketBuilder::default()
            .addresses(&*ALICE, &*BOB)
            .amount(&tickets.iter().fold(Balance::zero(BalanceType::HOPR), |acc, v| {
                acc + v.verified_ticket().amount
            }))
            .index(tickets.first().expect("should contain tickets").verified_ticket().index)
            .index_offset(
                tickets.last().expect("should contain tickets").verified_ticket().index
                    - tickets.first().expect("should contain tickets").verified_ticket().index
                    + 1,
            )
            .win_prob(1.0)
            .channel_epoch(
                tickets
                    .first()
                    .expect("should contain tickets")
                    .verified_ticket()
                    .channel_epoch,
            )
            .build_signed(&*BOB, Default::default())
            .unwrap();

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
            .await?;

        assert_eq!(actual, Some((BOB_OFFCHAIN.public().clone(), tickets)));

        let _ = db.process_received_aggregated_ticket(aggregated_ticket, &ALICE).await?;

        let actual_being_aggregated_count = hopr_db_entity::ticket::Entity::find()
            .filter(hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .count(&db.tickets_db)
            .await? as usize;

        assert_eq!(actual_being_aggregated_count, 0);

        Ok(())
    }

    #[async_std::test]
    async fn test_ticket_aggregation_should_fail_if_the_aggregated_ticket_value_is_lower_than_the_stored_one(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        const COUNT_TICKETS: usize = 5;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        let aggregated_ticket = hopr_internal_types::channels::Ticket::new(
            &ALICE.public().to_address(),
            &Balance::zero(BalanceType::HOPR),
            tickets.first().expect("should contain tickets").ticket.index.into(),
            (tickets.last().expect("should contain tickets").ticket.index
                - tickets.first().expect("should contain tickets").ticket.index
                + 1)
            .into(),
            1.0, // 100% winning probability
            (tickets.first().expect("should contain tickets").ticket.channel_epoch).into(),
            tickets
                .first()
                .expect("should contain tickets")
                .ticket
                .challenge
                .clone(),
            &BOB,
            &Hash::default(),
        )?;

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
            .await?;

        assert_eq!(actual, Some((BOB_OFFCHAIN.public().clone(), tickets)));

        assert!(db
            .process_received_aggregated_ticket(aggregated_ticket, &ALICE)
            .await
            .is_err());

        Ok(())
    }

    #[async_std::test]
    async fn test_ticket_aggregation_should_fail_if_the_aggregated_ticket_win_probability_is_not_equal_to_1(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        const COUNT_TICKETS: usize = 5;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        let aggregated_ticket = hopr_internal_types::channels::Ticket::new(
            &ALICE.public().to_address(),
            &tickets
                .iter()
                .fold(Balance::zero(BalanceType::HOPR), |acc, v| acc + v.ticket.amount),
            tickets.first().expect("should contain tickets").ticket.index.into(),
            (tickets.last().expect("should contain tickets").ticket.index
                - tickets.first().expect("should contain tickets").ticket.index
                + 1)
            .into(),
            0.5, // 50% winning probability
            (tickets.first().expect("should contain tickets").ticket.channel_epoch).into(),
            tickets
                .first()
                .expect("should contain tickets")
                .ticket
                .challenge
                .clone(),
            &BOB,
            &Hash::default(),
        )?;

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
            .await?;

        assert_eq!(actual, Some((BOB_OFFCHAIN.public().clone(), tickets)));

        assert!(db
            .process_received_aggregated_ticket(aggregated_ticket, &ALICE)
            .await
            .is_err());

        Ok(())
    }

    async fn init_db_with_channel(channel: ChannelEntry) -> HoprDb {
        let db = HoprDb::new_in_memory(BOB.clone()).await;

        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await
            .unwrap();

        add_peer_mappings(
            &db,
            vec![
                (ALICE_OFFCHAIN.clone(), ALICE.clone()),
                (BOB_OFFCHAIN.clone(), BOB.clone()),
            ],
        )
        .await
        .unwrap();

        db.upsert_channel(None, channel).await.unwrap();

        db
    }

    #[async_std::test]
    async fn test_aggregate_ticket_should_aggregate() {
        const COUNT_TICKETS: usize = 5;

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(u32::MAX),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(120))),
            4_u32.into(),
        );

        let db = init_db_with_channel(channel).await;

        let tickets = (0..COUNT_TICKETS)
            .into_iter()
            .map(|i| generate_random_ack_ticket(&BOB, &ALICE, i as u32))
            .collect::<Vec<_>>();

        let sum_value = tickets
            .iter()
            .fold(BalanceType::HOPR.zero(), |acc, x| acc + x.ticket.amount);
        let min_idx = tickets.iter().map(|t| t.ticket.index).min().unwrap();
        let max_idx = tickets.iter().map(|t| t.ticket.index).max().unwrap();

        let aggregated = db
            .aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets, &BOB)
            .await
            .expect("should aggregate");

        aggregated
            .verify(&BOB.public().to_address(), &Hash::default())
            .expect("ticket must be valid");
        assert_eq!(
            BOB.public().to_address(),
            aggregated.recover_signer(&Hash::default()).unwrap().to_address(),
            "must have correct signer"
        );

        assert!(aggregated.is_aggregated(), "must be aggregated");

        assert_eq!(
            COUNT_TICKETS, aggregated.index_offset as usize,
            "aggregated ticket must have correct offset"
        );
        assert_eq!(
            sum_value, aggregated.amount,
            "aggregated ticket token amount must be sum of individual tickets"
        );
        assert_eq!(
            1.0,
            aggregated.win_prob(),
            "aggregated ticket must have winning probability 1"
        );
        assert_eq!(min_idx, aggregated.index, "aggregated ticket must have correct index");
        assert_eq!(
            channel.get_id(),
            aggregated.channel_id,
            "aggregated ticket must have correct channel id"
        );
        assert_eq!(
            channel.channel_epoch.as_u32(),
            aggregated.channel_epoch,
            "aggregated ticket must have correct channel epoch"
        );

        assert_eq!(
            max_idx + 1,
            db.get_outgoing_ticket_index(channel.get_id())
                .await
                .unwrap()
                .load(Ordering::SeqCst)
        );
    }

    #[async_std::test]
    async fn test_aggregate_ticket_should_not_aggregate_zero_tickets() {
        let db = HoprDb::new_in_memory(BOB.clone()).await;

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), vec![], &BOB)
            .await
            .expect_err("should not aggregate empty ticket list");
    }

    #[async_std::test]
    async fn test_aggregate_ticket_should_aggregate_single_ticket_to_itself() {
        const COUNT_TICKETS: usize = 1;

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(u32::MAX),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(120))),
            4_u32.into(),
        );

        let db = init_db_with_channel(channel).await;

        let mut tickets = (0..COUNT_TICKETS)
            .into_iter()
            .map(|i| generate_random_ack_ticket(&BOB, &ALICE, i as u32))
            .collect::<Vec<_>>();

        let aggregated = db
            .aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await
            .expect("should aggregate");

        assert_eq!(tickets.pop().unwrap().ticket, aggregated);
    }

    #[async_std::test]
    async fn test_aggregate_ticket_should_not_aggregate_on_closed_channel() {
        const COUNT_TICKETS: usize = 3;

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(u32::MAX),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::Closed,
            4_u32.into(),
        );

        let db = init_db_with_channel(channel).await;

        let tickets = (0..COUNT_TICKETS)
            .into_iter()
            .map(|i| generate_random_ack_ticket(&BOB, &ALICE, i as u32))
            .collect::<Vec<_>>();

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await
            .expect_err("should not aggregate on closed channel");
    }

    #[async_std::test]
    async fn test_aggregate_ticket_should_not_aggregate_on_incoming_channel() {
        const COUNT_TICKETS: usize = 3;

        let channel = ChannelEntry::new(
            ALICE.public().to_address(),
            BOB.public().to_address(),
            BalanceType::HOPR.balance(u32::MAX),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let db = init_db_with_channel(channel).await;

        let tickets = (0..COUNT_TICKETS)
            .into_iter()
            .map(|i| generate_random_ack_ticket(&BOB, &ALICE, i as u32))
            .collect::<Vec<_>>();

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await
            .expect_err("should not aggregate on incoming channel");
    }

    #[async_std::test]
    async fn test_aggregate_ticket_should_not_aggregate_if_mismatching_channel_ids() {
        const COUNT_TICKETS: usize = 3;

        let channel = ChannelEntry::new(
            ALICE.public().to_address(),
            BOB.public().to_address(),
            BalanceType::HOPR.balance(u32::MAX),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let db = init_db_with_channel(channel).await;

        let mut tickets = (0..COUNT_TICKETS)
            .into_iter()
            .map(|i| generate_random_ack_ticket(&BOB, &ALICE, i as u32))
            .collect::<Vec<_>>();

        tickets[2] = generate_random_ack_ticket(&BOB, &ChainKeypair::random(), 2);

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await
            .expect_err("should not aggregate on mismatching channel ids");
    }

    #[async_std::test]
    async fn test_aggregate_ticket_should_not_aggregate_if_mismatching_channel_epoch() {
        const COUNT_TICKETS: usize = 3;

        let channel = ChannelEntry::new(
            ALICE.public().to_address(),
            BOB.public().to_address(),
            BalanceType::HOPR.balance(u32::MAX),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::Open,
            3_u32.into(),
        );

        let db = init_db_with_channel(channel).await;

        let tickets = (0..COUNT_TICKETS)
            .into_iter()
            .map(|i| generate_random_ack_ticket(&BOB, &ALICE, i as u32))
            .collect::<Vec<_>>();

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await
            .expect_err("should not aggregate on mismatching channel epoch");
    }

    #[async_std::test]
    async fn test_aggregate_ticket_should_not_aggregate_if_ticket_indices_overlap() {
        const COUNT_TICKETS: usize = 3;

        let channel = ChannelEntry::new(
            ALICE.public().to_address(),
            BOB.public().to_address(),
            BalanceType::HOPR.balance(u32::MAX),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::Open,
            3_u32.into(),
        );

        let db = init_db_with_channel(channel).await;

        let mut tickets = (0..COUNT_TICKETS)
            .into_iter()
            .map(|i| generate_random_ack_ticket(&BOB, &ALICE, i as u32))
            .collect::<Vec<_>>();

        tickets[1].ticket.index_offset = 2;
        tickets[1].ticket.sign(&BOB, &Hash::default());

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await
            .expect_err("should not aggregate on overlapping ticket indices");
    }

    #[async_std::test]
    async fn test_aggregate_ticket_should_not_aggregate_if_ticket_is_not_valid() {
        const COUNT_TICKETS: usize = 3;

        let channel = ChannelEntry::new(
            ALICE.public().to_address(),
            BOB.public().to_address(),
            BalanceType::HOPR.balance(u32::MAX),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::Open,
            3_u32.into(),
        );

        let db = init_db_with_channel(channel).await;

        let mut tickets = (0..COUNT_TICKETS)
            .into_iter()
            .map(|i| generate_random_ack_ticket(&BOB, &ALICE, i as u32))
            .collect::<Vec<_>>();

        // Modify the ticket and do not sign it
        tickets[1].ticket.amount = Balance::new(TICKET_VALUE - 10, BalanceType::HOPR);

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await
            .expect_err("should not aggregate on invalid tickets");
    }

    #[async_std::test]
    async fn test_aggregate_ticket_should_not_aggregate_if_ticket_is_not_winning() {
        const COUNT_TICKETS: usize = 3;

        let channel = ChannelEntry::new(
            ALICE.public().to_address(),
            BOB.public().to_address(),
            BalanceType::HOPR.balance(u32::MAX),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::Open,
            3_u32.into(),
        );

        let db = init_db_with_channel(channel).await;

        let mut tickets = (0..COUNT_TICKETS)
            .into_iter()
            .map(|i| generate_random_ack_ticket(&BOB, &ALICE, i as u32))
            .collect::<Vec<_>>();

        // Set winning probability to zero and sign the ticket again
        tickets[1].ticket.encoded_win_prob = [0u8; 7];
        tickets[1].ticket.sign(&BOB, &Hash::default());

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await
            .expect_err("should not aggregate non-winning tickets");
    }
}
