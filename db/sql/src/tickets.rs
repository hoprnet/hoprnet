use std::{
    cmp,
    ops::{Add, Bound},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use async_stream::stream;
use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt, stream::BoxStream};
use hopr_crypto_types::prelude::*;
use hopr_db_api::{
    errors::Result,
    info::DomainSeparator,
    prelude::{TicketIndexSelector, TicketMarker},
    resolver::HoprDbResolverOperations,
    tickets::{AggregationPrerequisites, ChannelTicketStatistics, HoprDbTicketOperations, TicketSelector},
};
use hopr_db_entity::{outgoing_ticket_index, ticket, ticket_statistics};
use hopr_internal_types::prelude::*;
#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::MultiGauge;
use hopr_primitive_types::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, QuerySelect, Set};
use sea_query::{Condition, Expr, IntoCondition, SimpleExpr};
use tracing::{debug, error, info, trace, warn};

use crate::{
    HoprDbGeneralModelOperations, OpenTransaction, OptTx, TargetDb,
    channels::HoprDbChannelOperations,
    db::HoprDb,
    errors::{DbSqlError, DbSqlError::LogicalError},
    info::HoprDbInfoOperations,
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    pub static ref METRIC_HOPR_TICKETS_INCOMING_STATISTICS: MultiGauge = MultiGauge::new(
        "hopr_tickets_incoming_statistics",
        "Ticket statistics for channels with incoming tickets.",
        &["channel", "statistic"]
    ).unwrap();
}

/// The maximum number of tickets that can sent for aggregation in a single request.
pub const MAX_TICKETS_TO_AGGREGATE_BATCH: u64 = 500;

/// The type is necessary solely to allow
/// implementing the [`IntoCondition`] trait for [`TicketSelector`]
/// from the `hopr_db_api` crate.
#[derive(Clone)]
pub(crate) struct WrappedTicketSelector(pub(crate) TicketSelector);

impl From<TicketSelector> for WrappedTicketSelector {
    fn from(selector: TicketSelector) -> Self {
        Self(selector)
    }
}

impl AsRef<TicketSelector> for WrappedTicketSelector {
    fn as_ref(&self) -> &TicketSelector {
        &self.0
    }
}

impl IntoCondition for WrappedTicketSelector {
    fn into_condition(self) -> Condition {
        let expr = self
            .0
            .channel_identifiers
            .into_iter()
            .map(|(channel_id, epoch)| {
                ticket::Column::ChannelId
                    .eq(channel_id.to_hex())
                    .and(ticket::Column::ChannelEpoch.eq(epoch.to_be_bytes().to_vec()))
            })
            .reduce(SimpleExpr::or);

        // This cannot happen, but instead of panicking, return an impossible condition object
        if expr.is_none() {
            return Condition::any().not();
        }

        let mut expr = expr.unwrap();

        match self.0.index {
            TicketIndexSelector::None => {
                // This will always be the case if there were multiple channel identifiers
            }
            TicketIndexSelector::Single(idx) => expr = expr.and(ticket::Column::Index.eq(idx.to_be_bytes().to_vec())),
            TicketIndexSelector::Multiple(idxs) => {
                expr = expr.and(ticket::Column::Index.is_in(idxs.into_iter().map(|i| i.to_be_bytes().to_vec())));
            }
            TicketIndexSelector::Range((lb, ub)) => {
                expr = match lb {
                    Bound::Included(gte) => expr.and(ticket::Column::Index.gte(gte.to_be_bytes().to_vec())),
                    Bound::Excluded(gt) => expr.and(ticket::Column::Index.gt(gt.to_be_bytes().to_vec())),
                    Bound::Unbounded => expr,
                };
                expr = match ub {
                    Bound::Included(lte) => expr.and(ticket::Column::Index.lte(lte.to_be_bytes().to_vec())),
                    Bound::Excluded(lt) => expr.and(ticket::Column::Index.lt(lt.to_be_bytes().to_vec())),
                    Bound::Unbounded => expr,
                };
            }
        }

        if let Some(state) = self.0.state {
            expr = expr.and(ticket::Column::State.eq(state as u8))
        }

        if self.0.only_aggregated {
            expr = expr.and(ticket::Column::IndexOffset.gt(1));
        }

        // Win prob lower bound
        expr = match self.0.win_prob.0 {
            Bound::Included(gte) => expr.and(ticket::Column::WinningProbability.gte(gte.as_encoded().to_vec())),
            Bound::Excluded(gt) => expr.and(ticket::Column::WinningProbability.gt(gt.as_encoded().to_vec())),
            Bound::Unbounded => expr,
        };

        // Win prob upper bound
        expr = match self.0.win_prob.1 {
            Bound::Included(lte) => expr.and(ticket::Column::WinningProbability.lte(lte.as_encoded().to_vec())),
            Bound::Excluded(lt) => expr.and(ticket::Column::WinningProbability.lt(lt.as_encoded().to_vec())),
            Bound::Unbounded => expr,
        };

        // Amount lower bound
        expr = match self.0.amount.0 {
            Bound::Included(gte) => expr.and(ticket::Column::Amount.gte(gte.amount().to_be_bytes().to_vec())),
            Bound::Excluded(gt) => expr.and(ticket::Column::Amount.gt(gt.amount().to_be_bytes().to_vec())),
            Bound::Unbounded => expr,
        };

        // Amount upper bound
        expr = match self.0.amount.1 {
            Bound::Included(lte) => expr.and(ticket::Column::Amount.lte(lte.amount().to_be_bytes().to_vec())),
            Bound::Excluded(lt) => expr.and(ticket::Column::Amount.lt(lt.amount().to_be_bytes().to_vec())),
            Bound::Unbounded => expr,
        };

        expr.into_condition()
    }
}

/// Filters the list of ticket models according to the prerequisites.
/// **NOTE:** the input list is assumed to be sorted by ticket index in ascending order.
///
/// The following is applied:
/// - the list of tickets is reduced so that the total amount on the tickets does not exceed the channel balance
/// - it is checked whether the list size is greater than `min_unaggregated_ratio`
/// - it is checked whether the ratio of total amount on the unaggregated tickets on the list and the channel balance
///   ratio is greater than `min_unaggregated_ratio`
pub(crate) fn filter_satisfying_ticket_models(
    prerequisites: AggregationPrerequisites,
    models: Vec<ticket::Model>,
    channel_entry: &ChannelEntry,
    min_win_prob: WinningProbability,
) -> crate::errors::Result<Vec<ticket::Model>> {
    let channel_id = channel_entry.get_id();

    let mut to_be_aggregated = Vec::with_capacity(models.len());
    let mut total_balance = HoprBalance::zero();

    for m in models {
        let ticket_wp: WinningProbability = m
            .winning_probability
            .as_slice()
            .try_into()
            .map_err(|_| DbSqlError::DecodingError)?;

        if ticket_wp.approx_cmp(&min_win_prob).is_lt() {
            warn!(
                channel_id = %channel_entry.get_id(),
                %ticket_wp, %min_win_prob, "encountered ticket with winning probability lower than the minimum threshold"
            );
            continue;
        }

        let to_add = HoprBalance::from_be_bytes(&m.amount);

        // Do a balance check to be sure not to aggregate more than the current channel stake
        total_balance += to_add;
        if total_balance.gt(&channel_entry.balance) {
            // Remove the last sub-balance which led to the overflow before breaking out of the loop.
            total_balance -= to_add;
            break;
        }

        to_be_aggregated.push(m);
    }

    // If there are no criteria, just send everything for aggregation
    if prerequisites.min_ticket_count.is_none() && prerequisites.min_unaggregated_ratio.is_none() {
        info!(channel = %channel_id, "Aggregation check OK, no aggregation prerequisites were given");
        return Ok(to_be_aggregated);
    }

    let to_be_agg_count = to_be_aggregated.len();

    // Check the aggregation threshold
    if let Some(agg_threshold) = prerequisites.min_ticket_count {
        if to_be_agg_count >= agg_threshold {
            info!(channel = %channel_id, count = to_be_agg_count, threshold = agg_threshold, "Aggregation check OK aggregated value greater than threshold");
            return Ok(to_be_aggregated);
        } else {
            debug!(channel = %channel_id, count = to_be_agg_count, threshold = agg_threshold,"Aggregation check FAIL not enough resources to aggregate");
        }
    }

    if let Some(unrealized_threshold) = prerequisites.min_unaggregated_ratio {
        let diminished_balance = channel_entry.balance.mul_f64(unrealized_threshold)?;

        // Trigger aggregation if unrealized balance greater or equal to X percentage of the current balance
        // and there are at least two tickets
        if total_balance.ge(&diminished_balance) {
            if to_be_agg_count > 1 {
                info!(channel = %channel_id, count = to_be_agg_count, balance = ?total_balance, ?diminished_balance, "Aggregation check OK: more unrealized than diminished balance");
                return Ok(to_be_aggregated);
            } else {
                debug!(channel = %channel_id, count = to_be_agg_count, balance = ?total_balance, ?diminished_balance, "Aggregation check FAIL: more unrealized than diminished balance but only 1 ticket");
            }
        } else {
            debug!(channel = %channel_id, count = to_be_agg_count, balance = ?total_balance, ?diminished_balance, "Aggregation check FAIL: less unrealized than diminished balance");
        }
    }

    debug!(channel = %channel_id,"Aggregation check FAIL: no prerequisites were met");
    Ok(vec![])
}

pub(crate) async fn find_stats_for_channel(
    tx: &OpenTransaction,
    channel_id: &Hash,
) -> crate::errors::Result<ticket_statistics::Model> {
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

impl HoprDb {
    async fn get_tickets_value_int<'a>(
        &'a self,
        tx: OptTx<'a>,
        selector: TicketSelector,
    ) -> Result<(usize, HoprBalance)> {
        let selector: WrappedTicketSelector = selector.into();
        Ok(self
            .nest_transaction_in_db(tx, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    ticket::Entity::find()
                        .filter(selector)
                        .stream(tx.as_ref())
                        .await
                        .map_err(DbSqlError::from)?
                        .map_err(DbSqlError::from)
                        .try_fold((0_usize, HoprBalance::zero()), |(count, value), t| async move {
                            Ok((count + 1, value + HoprBalance::from_be_bytes(t.amount)))
                        })
                        .await
                })
            })
            .await?)
    }
}

#[async_trait]
impl HoprDbTicketOperations for HoprDb {
    async fn get_all_tickets(&self) -> Result<Vec<AcknowledgedTicket>> {
        Ok(self
            .nest_transaction_in_db(None, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    ticket::Entity::find()
                        .all(tx.as_ref())
                        .await?
                        .into_iter()
                        .map(AcknowledgedTicket::try_from)
                        .collect::<hopr_db_entity::errors::Result<Vec<_>>>()
                        .map_err(DbSqlError::from)
                })
            })
            .await?)
    }

    async fn get_tickets(&self, selector: TicketSelector) -> Result<Vec<AcknowledgedTicket>> {
        debug!("fetching tickets via {selector}");
        let selector: WrappedTicketSelector = selector.into();

        Ok(self
            .nest_transaction_in_db(None, TargetDb::Tickets)
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
                        .map_err(DbSqlError::from)
                })
            })
            .await?)
    }

    async fn mark_tickets_as(&self, selector: TicketSelector, mark_as: TicketMarker) -> Result<usize> {
        let myself = self.clone();
        Ok(self
            .ticket_manager
            .with_write_locked_db(|tx| {
                Box::pin(async move {
                    let mut total_marked_count = 0;
                    for (channel_id, epoch) in selector.channel_identifiers.iter() {
                        let channel_selector = selector.clone().just_on_channel(*channel_id, epoch);

                        // Get the number of tickets and their value just for this channel
                        let (marked_count, marked_value) =
                            myself.get_tickets_value_int(Some(tx), channel_selector.clone()).await?;
                        trace!(marked_count, ?marked_value, ?mark_as, "ticket marking");

                        if marked_count > 0 {
                            // Delete the redeemed tickets first
                            let deleted = ticket::Entity::delete_many()
                                .filter(WrappedTicketSelector::from(channel_selector.clone()))
                                .exec(tx.as_ref())
                                .await?;

                            // Update the stats if successful
                            if deleted.rows_affected == marked_count as u64 {
                                let mut new_stats = find_stats_for_channel(tx, channel_id).await?.into_active_model();
                                let _current_value = match mark_as {
                                    TicketMarker::Redeemed => {
                                        let current_value = U256::from_be_bytes(new_stats.redeemed_value.as_ref());
                                        new_stats.redeemed_value =
                                            Set((current_value + marked_value.amount()).to_be_bytes().into());
                                        current_value
                                    }
                                    TicketMarker::Rejected => {
                                        let current_value = U256::from_be_bytes(new_stats.rejected_value.as_ref());
                                        new_stats.rejected_value =
                                            Set((current_value + marked_value.amount()).to_be_bytes().into());
                                        current_value
                                    }
                                    TicketMarker::Neglected => {
                                        let current_value = U256::from_be_bytes(new_stats.neglected_value.as_ref());
                                        new_stats.neglected_value =
                                            Set((current_value + marked_value.amount()).to_be_bytes().into());
                                        current_value
                                    }
                                };
                                new_stats.save(tx.as_ref()).await?;

                                #[cfg(all(feature = "prometheus", not(test)))]
                                {
                                    let channel = channel_id.to_string();
                                    METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                                        &[&channel, &mark_as.to_string()],
                                        (_current_value + marked_value.amount()).as_u128() as f64,
                                    );

                                    // Tickets that are counted as rejected were never counted as unredeemed,
                                    // so skip the metric subtraction in that case.
                                    if mark_as != TicketMarker::Rejected {
                                        let unredeemed_value = myself
                                            .caches
                                            .unrealized_value
                                            .get(&(*channel_id, *epoch))
                                            .await
                                            .unwrap_or_default();

                                        METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                                            &[&channel, "unredeemed"],
                                            (unredeemed_value - marked_value.amount()).amount().as_u128() as f64,
                                        );
                                    }
                                }

                                myself.caches.unrealized_value.invalidate(&(*channel_id, *epoch)).await;
                            } else {
                                return Err(DbSqlError::LogicalError(format!(
                                    "could not mark {marked_count} ticket as {mark_as}"
                                )));
                            }

                            trace!(marked_count, ?channel_id, ?mark_as, "removed tickets in channel");
                            total_marked_count += marked_count;
                        }
                    }

                    info!(
                        count = total_marked_count,
                        ?mark_as,
                        channel_count = selector.channel_identifiers.len(),
                        "removed tickets in channels",
                    );
                    Ok(total_marked_count)
                })
            })
            .await?)
    }

    async fn mark_unsaved_ticket_rejected(&self, ticket: &Ticket) -> Result<()> {
        let channel_id = ticket.channel_id;
        let amount = ticket
            .amount
            .mul_f64(ticket.win_prob().as_f64())
            .map_err(DbSqlError::from)?;

        Ok(self
            .ticket_manager
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

                    Ok::<(), DbSqlError>(())
                })
            })
            .await?)
    }

    async fn update_ticket_states_and_fetch<'a>(
        &'a self,
        selector: TicketSelector,
        new_state: AcknowledgedTicketStatus,
    ) -> Result<BoxStream<'a, AcknowledgedTicket>> {
        let selector: WrappedTicketSelector = selector.into();
        Ok(Box::pin(stream! {
            match ticket::Entity::find()
                .filter(selector)
                .stream(self.conn(TargetDb::Tickets))
                .await {
                Ok(mut stream) => {
                    while let Ok(Some(ticket)) = stream.try_next().await {
                        let active_ticket = ticket::ActiveModel {
                            id: Set(ticket.id),
                            state: Set(new_state as i8),
                            ..Default::default()
                        };

                        {
                            let _g = self.ticket_manager.mutex.lock();
                            if let Err(e) = active_ticket.update(self.conn(TargetDb::Tickets)).await {
                                error!(error = %e,"failed to update ticket in the db");
                            }
                        }

                        match AcknowledgedTicket::try_from(ticket) {
                            Ok(mut ticket) => {
                                // Update the state manually, since we do not want to re-fetch the model after the update
                                ticket.status = new_state;
                                yield ticket
                            },
                            Err(e) => {
                                tracing::error!(error = %e, "failed to decode ticket from the db");
                            }
                        }
                    }
                },
                Err(e) => tracing::error!(error = %e, "failed open ticket db stream")
            }
        }))
    }

    async fn update_ticket_states(
        &self,
        selector: TicketSelector,
        new_state: AcknowledgedTicketStatus,
    ) -> Result<usize> {
        let selector: WrappedTicketSelector = selector.into();
        Ok(self
            .ticket_manager
            .with_write_locked_db(|tx| {
                Box::pin(async move {
                    let update = ticket::Entity::update_many()
                        .filter(selector)
                        .col_expr(ticket::Column::State, Expr::value(new_state as u8))
                        .exec(tx.as_ref())
                        .await?;
                    Ok::<_, DbSqlError>(update.rows_affected as usize)
                })
            })
            .await?)
    }

    async fn get_ticket_statistics(&self, channel_id: Option<Hash>) -> Result<ChannelTicketStatistics> {
        let res = match channel_id {
            None => {
                #[cfg(all(feature = "prometheus", not(test)))]
                let mut per_channel_unredeemed = std::collections::HashMap::new();

                self.nest_transaction_in_db(None, TargetDb::Tickets)
                    .await?
                    .perform(|tx| {
                        Box::pin(async move {
                            let unredeemed_value = ticket::Entity::find()
                                .stream(tx.as_ref())
                                .await?
                                .try_fold(U256::zero(), |amount, x| {
                                    let unredeemed_value = U256::from_be_bytes(x.amount);

                                    #[cfg(all(feature = "prometheus", not(test)))]
                                    per_channel_unredeemed
                                        .entry(x.channel_id)
                                        .and_modify(|v| *v += unredeemed_value)
                                        .or_insert(unredeemed_value);

                                    futures::future::ok(amount + unredeemed_value)
                                })
                                .await?;

                            #[cfg(all(feature = "prometheus", not(test)))]
                            for (channel_id, unredeemed_value) in per_channel_unredeemed {
                                METRIC_HOPR_TICKETS_INCOMING_STATISTICS
                                    .set(&[&channel_id, "unredeemed"], unredeemed_value.as_u128() as f64);
                            }

                            let mut all_stats = ticket_statistics::Entity::find()
                                .all(tx.as_ref())
                                .await?
                                .into_iter()
                                .fold(ChannelTicketStatistics::default(), |mut acc, stats| {
                                    let neglected_value = HoprBalance::from_be_bytes(stats.neglected_value);
                                    acc.neglected_value += neglected_value;
                                    let redeemed_value = HoprBalance::from_be_bytes(stats.redeemed_value);
                                    acc.redeemed_value += redeemed_value;
                                    let rejected_value = HoprBalance::from_be_bytes(stats.rejected_value);
                                    acc.rejected_value += rejected_value;
                                    acc.winning_tickets += stats.winning_tickets as u128;

                                    #[cfg(all(feature = "prometheus", not(test)))]
                                    {
                                        METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                                            &[&stats.channel_id, "neglected"],
                                            neglected_value.amount().as_u128() as f64,
                                        );

                                        METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                                            &[&stats.channel_id, "redeemed"],
                                            redeemed_value.amount().as_u128() as f64,
                                        );

                                        METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                                            &[&stats.channel_id, "rejected"],
                                            rejected_value.amount().as_u128() as f64,
                                        );
                                    }

                                    acc
                                });

                            all_stats.unredeemed_value = unredeemed_value.into();

                            Ok::<_, DbSqlError>(all_stats)
                        })
                    })
                    .await
            }
            Some(channel) => {
                // We need to make sure the channel exists to avoid creating
                // statistic entry for a non-existing channel
                if self.get_channel_by_id(None, &channel).await?.is_none() {
                    return Err(DbSqlError::ChannelNotFound(channel).into());
                }

                self.nest_transaction_in_db(None, TargetDb::Tickets)
                    .await?
                    .perform(|tx| {
                        Box::pin(async move {
                            let stats = find_stats_for_channel(tx, &channel).await?;
                            let unredeemed_value = ticket::Entity::find()
                                .filter(ticket::Column::ChannelId.eq(channel.to_hex()))
                                .stream(tx.as_ref())
                                .await?
                                .try_fold(U256::zero(), |amount, x| {
                                    futures::future::ok(amount + U256::from_be_bytes(x.amount))
                                })
                                .await?;

                            Ok::<_, DbSqlError>(ChannelTicketStatistics {
                                winning_tickets: stats.winning_tickets as u128,
                                neglected_value: HoprBalance::from_be_bytes(stats.neglected_value),
                                redeemed_value: HoprBalance::from_be_bytes(stats.redeemed_value),
                                unredeemed_value: unredeemed_value.into(),
                                rejected_value: HoprBalance::from_be_bytes(stats.rejected_value),
                            })
                        })
                    })
                    .await
            }
        };
        Ok(res?)
    }

    async fn reset_ticket_statistics(&self) -> Result<()> {
        let res = self
            .nest_transaction_in_db(None, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    #[cfg(all(feature = "prometheus", not(test)))]
                    let rows = ticket_statistics::Entity::find().all(tx.as_ref()).await?;

                    // delete statistics for the found rows
                    let deleted = ticket_statistics::Entity::delete_many().exec(tx.as_ref()).await?;

                    #[cfg(all(feature = "prometheus", not(test)))]
                    {
                        if deleted.rows_affected > 0 {
                            for row in rows {
                                METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(&[&row.channel_id, "neglected"], 0.0_f64);

                                METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(&[&row.channel_id, "redeemed"], 0.0_f64);

                                METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(&[&row.channel_id, "rejected"], 0.0_f64);
                            }
                        }
                    }

                    debug!("reset ticket statistics for {:} channel(s)", deleted.rows_affected);

                    Ok::<_, DbSqlError>(())
                })
            })
            .await;

        Ok(res?)
    }

    async fn get_tickets_value(&self, selector: TicketSelector) -> Result<(usize, HoprBalance)> {
        self.get_tickets_value_int(None, selector).await
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

        Ok(self
            .caches
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
                                    Ok::<_, DbSqlError>(())
                                })
                            })
                            .await?;
                        0_u64
                    }
                })))
            })
            .await
            .map_err(|e: Arc<DbSqlError>| LogicalError(format!("failed to retrieve ticket index: {e}")))?)
    }

    async fn persist_outgoing_ticket_indices(&self) -> Result<usize> {
        let outgoing_indices = outgoing_ticket_index::Entity::find()
            .all(&self.tickets_db)
            .await
            .map_err(DbSqlError::from)?;

        let mut updated = 0;
        for index_model in outgoing_indices {
            let channel_id = Hash::from_hex(&index_model.channel_id).map_err(DbSqlError::from)?;
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
                                Ok::<_, DbSqlError>(())
                            })
                        })
                        .await?;

                    debug!("updated ticket index in channel {channel_id} from {db_index} to {cached_index}");
                    updated += 1;
                }
            } else {
                // The value is not yet in the cache, meaning there's low traffic on this
                // channel, so the value has not been yet fetched.
                trace!(?channel_id, "channel not in cache yet");
            }
        }

        Ok(Ok::<_, DbSqlError>(updated)?)
    }

    async fn prepare_aggregation_in_channel(
        &self,
        channel: &Hash,
        prerequisites: AggregationPrerequisites,
    ) -> Result<Option<(OffchainPublicKey, Vec<TransferableWinningTicket>, Hash)>> {
        let myself = self.clone();

        let channel_id = *channel;

        let (channel_entry, peer, domain_separator, min_win_prob) = self
            .nest_transaction_in_db(None, TargetDb::Index)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let entry = myself
                        .get_channel_by_id(Some(tx), &channel_id)
                        .await?
                        .ok_or(DbSqlError::ChannelNotFound(channel_id))?;

                    if entry.status == ChannelStatus::Closed {
                        return Err(DbSqlError::LogicalError(format!("channel '{channel_id}' is closed")));
                    } else if entry.direction(&myself.me_onchain) != Some(ChannelDirection::Incoming) {
                        return Err(DbSqlError::LogicalError(format!(
                            "channel '{channel_id}' is not incoming"
                        )));
                    }

                    let pk = myself
                        .resolve_packet_key(&entry.source)
                        .await?
                        .ok_or(DbSqlError::LogicalError(format!(
                            "peer '{}' has no offchain key record",
                            entry.source
                        )))?;

                    let indexer_data = myself.get_indexer_data(Some(tx)).await?;

                    let domain_separator = indexer_data
                        .channels_dst
                        .ok_or_else(|| crate::errors::DbSqlError::LogicalError("domain separator missing".into()))?;

                    Ok((
                        entry,
                        pk,
                        domain_separator,
                        indexer_data.minimum_incoming_ticket_winning_prob,
                    ))
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
                        .filter(WrappedTicketSelector::from(
                            TicketSelector::from(&channel_entry).with_state(AcknowledgedTicketStatus::BeingAggregated),
                        ))
                        .one(tx.as_ref())
                        .await?
                        .is_some()
                    {
                        return Err(DbSqlError::LogicalError(format!(
                            "'{channel_entry}' is already being aggregated",
                        )));
                    }

                    // find the index of the last ticket being redeemed
                    let first_idx_to_take = ticket::Entity::find()
                        .filter(WrappedTicketSelector::from(
                            TicketSelector::from(&channel_entry).with_state(AcknowledgedTicketStatus::BeingRedeemed),
                        ))
                        .order_by_desc(ticket::Column::Index)
                        .one(tx.as_ref())
                        .await?
                        .map(|m| U256::from_be_bytes(m.index).as_u64() + 1)
                        .unwrap_or(0_u64) // go from the lowest possible index of none is found
                        .max(channel_entry.ticket_index.as_u64()); // but cannot be less than the ticket index on the Channel entry

                    // get the list of all tickets to be aggregated
                    let to_be_aggregated = ticket::Entity::find()
                        .filter(WrappedTicketSelector::from(TicketSelector::from(&channel_entry)))
                        .filter(ticket::Column::Index.gte(first_idx_to_take.to_be_bytes().to_vec()))
                        .filter(ticket::Column::State.ne(AcknowledgedTicketStatus::BeingAggregated as u8))
                        .order_by_asc(ticket::Column::Index)// tickets must be sorted by indices in ascending order
                        .limit(MAX_TICKETS_TO_AGGREGATE_BATCH)
                        .all(tx.as_ref())
                        .await?;

                    // Filter the list of tickets according to the prerequisites
                    let mut to_be_aggregated: Vec<TransferableWinningTicket> =
                        filter_satisfying_ticket_models(prerequisites, to_be_aggregated, &channel_entry, min_win_prob)?
                            .into_iter()
                            .map(|model| {
                                AcknowledgedTicket::try_from(model)
                                    .map_err(DbSqlError::from)
                                    .and_then(|ack| {
                                        ack.into_transferable(&myself.chain_key, &domain_separator)
                                            .map_err(DbSqlError::from)
                                    })
                            })
                            .collect::<crate::errors::Result<Vec<_>>>()?;

                    let mut neglected_idxs = Vec::new();

                    if !to_be_aggregated.is_empty() {
                        // Clean up any tickets in this channel that are already inside an aggregated ticket.
                        // This situation cannot be avoided 100% as aggregation can be triggered when out-of-order
                        // tickets arrive and only some of them are necessary to satisfy the aggregation threshold.
                        // The following code *assumes* that only the first ticket with the lowest index *can be* an aggregate.
                        let first_ticket = to_be_aggregated[0].ticket.clone();
                        let mut i = 1;
                        while i < to_be_aggregated.len() {
                            let current_idx = to_be_aggregated[i].ticket.index;
                            if (first_ticket.index..first_ticket.index + first_ticket.index_offset as u64).contains(&current_idx) {
                                // Cleanup is the only reasonable thing to do at this point,
                                // since the aggregator will check for index range overlaps and deny
                                // the aggregation of the entire batch otherwise.
                                warn!(ticket_id = current_idx, channel = %channel_id, ?first_ticket, "ticket in channel has been already aggregated and will be removed");
                                neglected_idxs.push(current_idx);
                                to_be_aggregated.remove(i);
                            } else {
                                i += 1;
                            }
                        }

                        // The cleanup (neglecting of tickets) is not made directly here but on the next ticket redemption in this channel
                        // See handler.rs around L402
                        if !neglected_idxs.is_empty() {
                            warn!(count = neglected_idxs.len(), channel = %channel_id, "tickets were neglected due to duplication in an aggregated ticket!");
                        }

                        // mark all tickets with appropriate characteristics as being aggregated
                        let marked: sea_orm::UpdateResult = ticket::Entity::update_many()
                            .filter(WrappedTicketSelector::from(TicketSelector::from(&channel_entry)))
                            .filter(ticket::Column::Index.is_in(to_be_aggregated.iter().map(|t| t.ticket.index.to_be_bytes().to_vec())))
                            .filter(ticket::Column::State.ne(AcknowledgedTicketStatus::BeingAggregated as u8))
                            .col_expr(
                                ticket::Column::State,
                                Expr::value(AcknowledgedTicketStatus::BeingAggregated as i8),
                            )
                            .exec(tx.as_ref())
                            .await?;

                        if marked.rows_affected as usize != to_be_aggregated.len() {
                            return Err(DbSqlError::LogicalError(format!(
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

        Ok((!tickets.is_empty()).then_some((peer, tickets, domain_separator)))
    }

    async fn rollback_aggregation_in_channel(&self, channel: Hash) -> Result<()> {
        let channel_entry = self
            .get_channel_by_id(None, &channel)
            .await?
            .ok_or(DbSqlError::ChannelNotFound(channel))?;

        let selector = TicketSelector::from(channel_entry).with_state(AcknowledgedTicketStatus::BeingAggregated);

        let reverted = self
            .update_ticket_states(selector, AcknowledgedTicketStatus::Untouched)
            .await?;

        info!(
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
            return Err(DbSqlError::LogicalError(
                "chain key for ticket aggregation does not match the DB public address".into(),
            )
            .into());
        }

        let myself = self.clone();
        let channel_id = aggregated_ticket.channel_id;

        let (channel_entry, domain_separator) = self
            .nest_transaction_in_db(None, TargetDb::Index)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let entry = myself
                        .get_channel_by_id(Some(tx), &channel_id)
                        .await?
                        .ok_or(DbSqlError::ChannelNotFound(channel_id))?;

                    if entry.status == ChannelStatus::Closed {
                        return Err(DbSqlError::LogicalError(format!("channel '{channel_id}' is closed")));
                    } else if entry.direction(&myself.me_onchain) != Some(ChannelDirection::Incoming) {
                        return Err(DbSqlError::LogicalError(format!(
                            "channel '{channel_id}' is not incoming"
                        )));
                    }

                    let domain_separator =
                        myself.get_indexer_data(Some(tx)).await?.channels_dst.ok_or_else(|| {
                            crate::errors::DbSqlError::LogicalError("domain separator missing".into())
                        })?;

                    Ok((entry, domain_separator))
                })
            })
            .await?;

        // Verify the ticket first
        let aggregated_ticket = aggregated_ticket
            .verify(&channel_entry.source, &domain_separator)
            .map_err(|e| {
                DbSqlError::LogicalError(format!(
                    "failed to verify received aggregated ticket in {channel_id}: {e}"
                ))
            })?;

        // Aggregated tickets always have 100% winning probability
        if !aggregated_ticket.win_prob().approx_eq(&WinningProbability::ALWAYS) {
            return Err(DbSqlError::LogicalError("Aggregated tickets must have 100% win probability".into()).into());
        }

        let acknowledged_tickets = self
            .nest_transaction_in_db(None, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    ticket::Entity::find()
                        .filter(WrappedTicketSelector::from(
                            TicketSelector::from(&channel_entry).with_state(AcknowledgedTicketStatus::BeingAggregated),
                        ))
                        .all(tx.as_ref())
                        .await
                        .map_err(DbSqlError::BackendError)
                })
            })
            .await?;

        if acknowledged_tickets.is_empty() {
            debug!("Received unexpected aggregated ticket in channel {channel_id}");
            return Err(DbSqlError::LogicalError(format!(
                "failed insert aggregated ticket, because no tickets seem to be aggregated for '{channel_id}'",
            ))
            .into());
        }

        let stored_value = acknowledged_tickets
            .iter()
            .map(|m| HoprBalance::from_be_bytes(&m.amount))
            .sum();

        // The value of a received ticket can be higher (profit for us) but not lower
        if aggregated_ticket.verified_ticket().amount.lt(&stored_value) {
            error!(channel = %channel_id, "Aggregated ticket value in channel is lower than sum of stored tickets");
            return Err(DbSqlError::LogicalError("Value of received aggregated ticket is too low".into()).into());
        }

        let acknowledged_tickets = acknowledged_tickets
            .into_iter()
            .map(AcknowledgedTicket::try_from)
            .collect::<hopr_db_entity::errors::Result<Vec<AcknowledgedTicket>>>()
            .map_err(DbSqlError::from)?;

        // can be done, because the tickets collection is tested for emptiness before
        let first_stored_ticket = acknowledged_tickets.first().unwrap();

        // calculate the new current ticket index
        #[allow(unused_variables)]
        let current_ticket_index_from_aggregated_ticket =
            U256::from(aggregated_ticket.verified_ticket().index).add(aggregated_ticket.verified_ticket().index_offset);

        let acked_aggregated_ticket = aggregated_ticket.into_acknowledged(first_stored_ticket.response.clone());

        let ticket = acked_aggregated_ticket.clone();
        self.ticket_manager.replace_tickets(ticket).await?;

        info!(%acked_aggregated_ticket, "successfully processed received aggregated ticket");
        Ok(acked_aggregated_ticket)
    }

    async fn aggregate_tickets(
        &self,
        destination: OffchainPublicKey,
        mut acked_tickets: Vec<TransferableWinningTicket>,
        me: &ChainKeypair,
    ) -> Result<VerifiedTicket> {
        if me.public().to_address() != self.me_onchain {
            return Err(DbSqlError::LogicalError(
                "chain key for ticket aggregation does not match the DB public address".into(),
            )
            .into());
        }

        let domain_separator = self
            .get_indexer_data(None)
            .await?
            .domain_separator(DomainSeparator::Channel)
            .ok_or_else(|| DbSqlError::LogicalError("domain separator missing".into()))?;

        if acked_tickets.is_empty() {
            return Err(DbSqlError::LogicalError("at least one ticket required for aggregation".to_owned()).into());
        }

        if acked_tickets.len() == 1 {
            let single = acked_tickets
                .pop()
                .unwrap()
                .into_redeemable(&self.me_onchain, &domain_separator)
                .map_err(DbSqlError::from)?;

            self.compare_and_set_outgoing_ticket_index(
                single.verified_ticket().channel_id,
                single.verified_ticket().index + 1,
            )
            .await?;

            return Ok(single.ticket);
        }

        acked_tickets.sort_by(|a, b| a.partial_cmp(b).unwrap_or(cmp::Ordering::Equal));
        acked_tickets.dedup();

        let myself = self.clone();
        let address = myself
            .resolve_chain_key(&destination)
            .await?
            .ok_or(DbSqlError::LogicalError(format!(
                "peer '{}' has no chain key record",
                destination.to_peerid_str()
            )))?;

        let (channel_entry, destination, min_win_prob) = self
            .nest_transaction_in_db(None, TargetDb::Index)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let entry = myself
                        .get_channel_by_parties(Some(tx), &myself.me_onchain, &address, false)
                        .await?
                        .ok_or_else(|| {
                            DbSqlError::ChannelNotFound(generate_channel_id(&myself.me_onchain, &address))
                        })?;

                    if entry.status == ChannelStatus::Closed {
                        return Err(DbSqlError::LogicalError(format!("{entry} is closed")));
                    } else if entry.direction(&myself.me_onchain) != Some(ChannelDirection::Outgoing) {
                        return Err(DbSqlError::LogicalError(format!("{entry} is not outgoing")));
                    }

                    let min_win_prob = myself
                        .get_indexer_data(Some(tx))
                        .await?
                        .minimum_incoming_ticket_winning_prob;
                    Ok((entry, address, min_win_prob))
                })
            })
            .await?;

        let channel_balance = channel_entry.balance;
        let channel_epoch = channel_entry.channel_epoch.as_u32();
        let channel_id = channel_entry.get_id();

        let mut final_value = HoprBalance::zero();

        // Validate all received tickets and turn them into RedeemableTickets
        let verified_tickets = acked_tickets
            .into_iter()
            .map(|t| t.into_redeemable(&self.me_onchain, &domain_separator))
            .collect::<hopr_internal_types::errors::Result<Vec<_>>>()
            .map_err(|e| {
                DbSqlError::LogicalError(format!("trying to aggregate an invalid or a non-winning ticket: {e}"))
            })?;

        // Perform additional consistency check on the verified tickets
        for (i, acked_ticket) in verified_tickets.iter().enumerate() {
            if channel_id != acked_ticket.verified_ticket().channel_id {
                return Err(DbSqlError::LogicalError(format!(
                    "ticket for aggregation has an invalid channel id {}",
                    acked_ticket.verified_ticket().channel_id
                ))
                .into());
            }

            if acked_ticket.verified_ticket().channel_epoch != channel_epoch {
                return Err(DbSqlError::LogicalError("channel epochs do not match".into()).into());
            }

            if i + 1 < verified_tickets.len()
                && acked_ticket.verified_ticket().index + acked_ticket.verified_ticket().index_offset as u64
                    > verified_tickets[i + 1].verified_ticket().index
            {
                return Err(DbSqlError::LogicalError("tickets with overlapping index intervals".into()).into());
            }

            if acked_ticket
                .verified_ticket()
                .win_prob()
                .approx_cmp(&min_win_prob)
                .is_lt()
            {
                return Err(DbSqlError::LogicalError(
                    "cannot aggregate ticket with lower than minimum winning probability in network".into(),
                )
                .into());
            }

            final_value += acked_ticket.verified_ticket().amount;
            if final_value.gt(&channel_balance) {
                return Err(DbSqlError::LogicalError(format!(
                    "ticket amount to aggregate {final_value} is greater than the balance {channel_balance} of \
                     channel {channel_id}"
                ))
                .into());
            }
        }

        info!(
            "aggregated {} tickets in channel {channel_id} with total value {final_value}",
            verified_tickets.len()
        );

        let first_acked_ticket = verified_tickets.first().unwrap();
        let last_acked_ticket = verified_tickets.last().unwrap();

        // calculate the minimum current ticket index as the larger value from the acked ticket index and on-chain
        // ticket_index from channel_entry
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
            .win_prob(WinningProbability::ALWAYS) // Aggregated tickets have always 100% winning probability
            .channel_epoch(channel_epoch)
            .eth_challenge(first_acked_ticket.verified_ticket().challenge)
            .build_signed(me, &domain_separator)
            .map_err(DbSqlError::from)?)
    }

    async fn fix_channels_next_ticket_state(&self) -> Result<()> {
        let channels = self.get_incoming_channels(None).await?;

        for channel in channels.into_iter() {
            let selector = TicketSelector::from(&channel)
                .with_state(AcknowledgedTicketStatus::BeingRedeemed)
                .with_index(channel.ticket_index.as_u64());

            let mut tickets_stream = self
                .update_ticket_states_and_fetch(selector, AcknowledgedTicketStatus::Untouched)
                .await?;

            while let Some(ticket) = tickets_stream.next().await {
                let channel_id = channel.get_id();
                let ticket_index = ticket.verified_ticket().index;
                let ticket_amount = ticket.verified_ticket().amount;
                info!(%channel_id, %ticket_index, %ticket_amount, "fixed next out-of-sync ticket");
            }
        }

        Ok(())
    }
}

impl HoprDb {
    /// Used only by non-SQLite code and tests.
    pub async fn upsert_ticket<'a>(&'a self, tx: OptTx<'a>, acknowledged_ticket: AcknowledgedTicket) -> Result<()> {
        self.nest_transaction_in_db(tx, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    // For upserting, we must select only by the triplet (channel id, epoch, index)
                    let selector = WrappedTicketSelector::from(
                        TicketSelector::new(
                            acknowledged_ticket.verified_ticket().channel_id,
                            acknowledged_ticket.verified_ticket().channel_epoch,
                        )
                        .with_index(acknowledged_ticket.verified_ticket().index),
                    );

                    debug!("upserting ticket {acknowledged_ticket}");
                    let mut model = ticket::ActiveModel::from(acknowledged_ticket);

                    if let Some(ticket) = ticket::Entity::find().filter(selector).one(tx.as_ref()).await? {
                        model.id = Set(ticket.id);
                    }

                    Ok::<_, DbSqlError>(model.save(tx.as_ref()).await?)
                })
            })
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ops::Add,
        sync::atomic::Ordering,
        time::{Duration, SystemTime},
    };

    use anyhow::{Context, anyhow};
    use futures::{StreamExt, pin_mut};
    use hex_literal::hex;
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::prelude::*;
    use hopr_db_api::{
        info::DomainSeparator,
        prelude::{DbError, TicketMarker},
        tickets::ChannelTicketStatistics,
    };
    use hopr_db_entity::ticket;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, Set};

    use crate::{
        HoprDbGeneralModelOperations, TargetDb,
        accounts::HoprDbAccountOperations,
        channels::HoprDbChannelOperations,
        db::HoprDb,
        errors::DbSqlError,
        info::HoprDbInfoOperations,
        tickets::{AggregationPrerequisites, HoprDbTicketOperations, TicketSelector, filter_satisfying_ticket_models},
    };

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).expect("lazy static keypair should be valid");
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).expect("lazy static keypair should be valid");
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
                    published_at: 0,
                },
            )
            .await?
        }

        Ok(())
    }

    fn generate_random_ack_ticket(
        src: &ChainKeypair,
        dst: &ChainKeypair,
        index: u64,
        index_offset: u32,
        win_prob: f64,
    ) -> anyhow::Result<AcknowledgedTicket> {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();
        let challenge = Response::from_half_keys(&hk1, &hk2)?.to_challenge()?;

        Ok(TicketBuilder::default()
            .addresses(src, dst)
            .amount(TICKET_VALUE)
            .index(index)
            .index_offset(index_offset)
            .win_prob(win_prob.try_into()?)
            .channel_epoch(4)
            .challenge(challenge)
            .build_signed(src, &Hash::default())?
            .into_acknowledged(Response::from_half_keys(&hk1, &hk2)?))
    }

    async fn init_db_with_tickets(
        db: &HoprDb,
        count_tickets: u64,
    ) -> anyhow::Result<(ChannelEntry, Vec<AcknowledgedTicket>)> {
        init_db_with_tickets_and_channel(db, count_tickets, None, 1.0).await
    }

    async fn init_db_with_low_win_prob_tickets(
        db: &HoprDb,
        count_tickets: u64,
        win_prob: f64,
    ) -> anyhow::Result<(ChannelEntry, Vec<AcknowledgedTicket>)> {
        init_db_with_tickets_and_channel(db, count_tickets, None, win_prob).await
    }

    async fn init_db_with_tickets_and_channel(
        db: &HoprDb,
        count_tickets: u64,
        channel_ticket_index: Option<u32>,
        win_prob: f64,
    ) -> anyhow::Result<(ChannelEntry, Vec<AcknowledgedTicket>)> {
        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            u32::MAX.into(),
            channel_ticket_index.unwrap_or(0u32).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        db.upsert_channel(None, channel).await?;

        let tickets: Vec<AcknowledgedTicket> = (0..count_tickets)
            .map(|i| generate_random_ack_ticket(&BOB, &ALICE, i, 1, win_prob))
            .collect::<anyhow::Result<Vec<AcknowledgedTicket>>>()?;

        let db_clone = db.clone();
        let tickets_clone = tickets.clone();
        db.begin_transaction_in_db(TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    for t in tickets_clone {
                        db_clone.upsert_ticket(Some(tx), t).await?;
                    }
                    Ok::<(), DbSqlError>(())
                })
            })
            .await?;

        Ok((channel, tickets))
    }

    #[tokio::test]
    async fn test_insert_get_ticket() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Hash::default())
            .await?;

        let (channel, mut tickets) = init_db_with_tickets(&db, 1).await?;
        let ack_ticket = tickets.pop().context("ticket should be present")?;

        assert_eq!(
            channel.get_id(),
            ack_ticket.verified_ticket().channel_id,
            "channel ids must match"
        );
        assert_eq!(
            channel.channel_epoch.as_u32(),
            ack_ticket.verified_ticket().channel_epoch,
            "epochs must match"
        );

        let db_ticket = db
            .get_tickets((&ack_ticket).into())
            .await?
            .first()
            .cloned()
            .context("ticket should exist")?;

        assert_eq!(ack_ticket, db_ticket, "tickets must be equal");

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_redeemed() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        const COUNT_TICKETS: u64 = 10;

        let (_, tickets) = init_db_with_tickets(&db, COUNT_TICKETS).await?;

        let stats = db.get_ticket_statistics(None).await?;
        assert_eq!(
            HoprBalance::from(TICKET_VALUE * COUNT_TICKETS),
            stats.unredeemed_value,
            "unredeemed balance must match"
        );
        assert_eq!(
            HoprBalance::zero(),
            stats.redeemed_value,
            "there must be 0 redeemed value"
        );

        assert_eq!(
            stats,
            db.get_ticket_statistics(Some(*CHANNEL_ID)).await?,
            "per channel stats must be same"
        );

        const TO_REDEEM: u64 = 2;
        let db_clone = db.clone();
        db.begin_transaction_in_db(TargetDb::Tickets)
            .await?
            .perform(|_tx| {
                Box::pin(async move {
                    for ticket in tickets.iter().take(TO_REDEEM as usize) {
                        let r = db_clone.mark_tickets_as(ticket.into(), TicketMarker::Redeemed).await?;
                        assert_eq!(1, r, "must redeem only a single ticket");
                    }
                    Ok::<(), DbSqlError>(())
                })
            })
            .await?;

        let stats = db.get_ticket_statistics(None).await?;
        assert_eq!(
            HoprBalance::from(TICKET_VALUE * (COUNT_TICKETS - TO_REDEEM)),
            stats.unredeemed_value,
            "unredeemed balance must match"
        );
        assert_eq!(
            HoprBalance::from(TICKET_VALUE * TO_REDEEM),
            stats.redeemed_value,
            "there must be a redeemed value"
        );

        assert_eq!(
            stats,
            db.get_ticket_statistics(Some(*CHANNEL_ID)).await?,
            "per channel stats must be same"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_redeem_should_not_mark_redeem_twice() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;

        let ticket = init_db_with_tickets(&db, 1)
            .await?
            .1
            .pop()
            .context("should contain a ticket")?;

        db.mark_tickets_as((&ticket).into(), TicketMarker::Redeemed).await?;
        assert_eq!(0, db.mark_tickets_as((&ticket).into(), TicketMarker::Redeemed).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_redeem_should_redeem_all_tickets() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;

        let count_tickets = 10;
        let channel = init_db_with_tickets(&db, count_tickets).await?.0;

        let count_marked = db.mark_tickets_as((&channel).into(), TicketMarker::Redeemed).await?;
        assert_eq!(count_tickets, count_marked as u64, "must mark all tickets in channel");

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_tickets_neglected() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        const COUNT_TICKETS: u64 = 10;

        let (channel, _) = init_db_with_tickets(&db, COUNT_TICKETS).await?;

        let stats = db.get_ticket_statistics(None).await?;
        assert_eq!(
            HoprBalance::from(TICKET_VALUE * COUNT_TICKETS),
            stats.unredeemed_value,
            "unredeemed balance must match"
        );
        assert_eq!(
            HoprBalance::zero(),
            stats.neglected_value,
            "there must be 0 redeemed value"
        );

        assert_eq!(
            stats,
            db.get_ticket_statistics(Some(*CHANNEL_ID)).await?,
            "per channel stats must be same"
        );

        db.mark_tickets_as((&channel).into(), TicketMarker::Neglected).await?;

        let stats = db.get_ticket_statistics(None).await?;
        assert_eq!(
            HoprBalance::zero(),
            stats.unredeemed_value,
            "unredeemed balance must be zero"
        );
        assert_eq!(
            HoprBalance::from(TICKET_VALUE * COUNT_TICKETS),
            stats.neglected_value,
            "there must be a neglected value"
        );

        assert_eq!(
            stats,
            db.get_ticket_statistics(Some(*CHANNEL_ID)).await?,
            "per channel stats must be same"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_unsaved_ticket_rejected() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;

        let (_, mut ticket) = init_db_with_tickets(&db, 1).await?;
        let ticket = ticket.pop().context("ticket should be present")?.ticket;

        let stats = db.get_ticket_statistics(None).await?;
        assert_eq!(HoprBalance::zero(), stats.rejected_value);
        assert_eq!(
            stats,
            db.get_ticket_statistics(Some(*CHANNEL_ID)).await?,
            "per channel stats must be same"
        );

        db.mark_unsaved_ticket_rejected(ticket.verified_ticket()).await?;

        let stats = db.get_ticket_statistics(None).await?;
        assert_eq!(ticket.verified_ticket().amount, stats.rejected_value);
        assert_eq!(
            stats,
            db.get_ticket_statistics(Some(*CHANNEL_ID)).await?,
            "per channel stats must be same"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_unsaved_low_win_prob_ticket_rejected() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        let win_prob: f64 = 0.25;

        let tickets = init_db_with_low_win_prob_tickets(&db, 10, win_prob).await?.1;

        assert!(!tickets.is_empty(), "tickets must be present");

        let stats = db.get_ticket_statistics(None).await?;
        assert_eq!(HoprBalance::zero(), stats.rejected_value);
        assert_eq!(
            stats,
            db.get_ticket_statistics(Some(*CHANNEL_ID)).await?,
            "per channel stats must be same"
        );

        for ticket in tickets.iter() {
            db.mark_unsaved_ticket_rejected(ticket.verified_ticket()).await?;
        }

        let stats = db.get_ticket_statistics(None).await?;
        let sum_all_tickets: HoprBalance = tickets.iter().map(|t| t.verified_ticket().amount).sum();

        assert_eq!(sum_all_tickets.mul_f64(win_prob)?, stats.rejected_value);
        assert_eq!(
            stats,
            db.get_ticket_statistics(Some(*CHANNEL_ID)).await?,
            "per channel stats must be same"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_update_tickets_states_and_fetch() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        let channel = init_db_with_tickets(&db, 10).await?.0;

        let selector = TicketSelector::from(&channel).with_index(5);

        let v: Vec<AcknowledgedTicket> = db
            .update_ticket_states_and_fetch(selector, AcknowledgedTicketStatus::BeingRedeemed)
            .await?
            .collect()
            .await;

        assert_eq!(1, v.len(), "single ticket must be updated");
        assert_eq!(
            AcknowledgedTicketStatus::BeingRedeemed,
            v.first().context("should contain a ticket")?.status,
            "status must be set"
        );

        let selector = TicketSelector::from(&channel).with_state(AcknowledgedTicketStatus::Untouched);

        let v: Vec<AcknowledgedTicket> = db
            .update_ticket_states_and_fetch(selector, AcknowledgedTicketStatus::BeingRedeemed)
            .await?
            .collect()
            .await;

        assert_eq!(9, v.len(), "only specific tickets must have state set");
        assert!(
            v.iter().all(|t| t.verified_ticket().index != 5),
            "only tickets with different state must update"
        );
        assert!(
            v.iter().all(|t| t.status == AcknowledgedTicketStatus::BeingRedeemed),
            "tickets must have updated state"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_update_tickets_states() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        let channel = init_db_with_tickets(&db, 10).await?.0;
        let selector = TicketSelector::from(&channel).with_state(AcknowledgedTicketStatus::Untouched);

        db.update_ticket_states(selector.clone(), AcknowledgedTicketStatus::BeingRedeemed)
            .await?;

        let v: Vec<AcknowledgedTicket> = db
            .update_ticket_states_and_fetch(selector, AcknowledgedTicketStatus::BeingRedeemed)
            .await?
            .collect()
            .await;

        assert!(v.is_empty(), "must not update if already updated");

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_index_should_be_zero_if_not_yet_present() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let hash = Hash::default();

        let idx = db.get_outgoing_ticket_index(hash).await?;
        assert_eq!(0, idx.load(Ordering::SeqCst), "initial index must be zero");

        let r = hopr_db_entity::outgoing_ticket_index::Entity::find()
            .filter(hopr_db_entity::outgoing_ticket_index::Column::ChannelId.eq(hash.to_hex()))
            .one(&db.tickets_db)
            .await?
            .context("index must exist")?;

        assert_eq!(0, U256::from_be_bytes(r.index).as_u64(), "index must be zero");

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_stats_must_fail_for_non_existing_channel() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        db.get_ticket_statistics(Some(*CHANNEL_ID))
            .await
            .expect_err("must fail for non-existing channel");

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_stats_must_be_zero_when_no_tickets() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            u32::MAX.into(),
            0.into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        db.upsert_channel(None, channel).await?;

        let stats = db.get_ticket_statistics(Some(*CHANNEL_ID)).await?;

        assert_eq!(
            ChannelTicketStatistics::default(),
            stats,
            "must be equal to default which is all zeros"
        );

        assert_eq!(
            stats,
            db.get_ticket_statistics(None).await?,
            "per-channel stats must be the same as global stats"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_stats_must_be_different_per_channel() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let channel_1 = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            u32::MAX.into(),
            0.into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        db.upsert_channel(None, channel_1).await?;

        let channel_2 = ChannelEntry::new(
            ALICE.public().to_address(),
            BOB.public().to_address(),
            u32::MAX.into(),
            0.into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        db.upsert_channel(None, channel_2).await?;

        let t1 = generate_random_ack_ticket(&BOB, &ALICE, 1, 1, 1.0)?;
        let t2 = generate_random_ack_ticket(&ALICE, &BOB, 1, 1, 1.0)?;

        let value = t1.verified_ticket().amount;

        db.upsert_ticket(None, t1).await?;
        db.upsert_ticket(None, t2).await?;

        let stats_1 = db
            .get_ticket_statistics(Some(generate_channel_id(
                &BOB.public().to_address(),
                &ALICE.public().to_address(),
            )))
            .await?;

        let stats_2 = db
            .get_ticket_statistics(Some(generate_channel_id(
                &ALICE.public().to_address(),
                &BOB.public().to_address(),
            )))
            .await?;

        assert_eq!(value, stats_1.unredeemed_value);
        assert_eq!(value, stats_2.unredeemed_value);

        assert_eq!(HoprBalance::zero(), stats_1.neglected_value);
        assert_eq!(HoprBalance::zero(), stats_2.neglected_value);

        assert_eq!(stats_1, stats_2);

        db.mark_tickets_as(channel_1.into(), TicketMarker::Neglected).await?;

        let stats_1 = db
            .get_ticket_statistics(Some(generate_channel_id(
                &BOB.public().to_address(),
                &ALICE.public().to_address(),
            )))
            .await?;

        let stats_2 = db
            .get_ticket_statistics(Some(generate_channel_id(
                &ALICE.public().to_address(),
                &BOB.public().to_address(),
            )))
            .await?;

        assert_eq!(HoprBalance::zero(), stats_1.unredeemed_value);
        assert_eq!(value, stats_1.neglected_value);

        assert_eq!(HoprBalance::zero(), stats_2.neglected_value);

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_index_compare_and_set_and_increment() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let hash = Hash::default();

        let old_idx = db.compare_and_set_outgoing_ticket_index(hash, 1).await?;
        assert_eq!(0, old_idx, "old value must be 0");

        let new_idx = db.get_outgoing_ticket_index(hash).await?.load(Ordering::SeqCst);
        assert_eq!(1, new_idx, "new value must be 1");

        let old_idx = db.increment_outgoing_ticket_index(hash).await?;
        assert_eq!(1, old_idx, "old value must be 1");

        let new_idx = db.get_outgoing_ticket_index(hash).await?.load(Ordering::SeqCst);
        assert_eq!(2, new_idx, "new value must be 2");

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_index_compare_and_set_must_not_decrease() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let hash = Hash::default();

        let new_idx = db.get_outgoing_ticket_index(hash).await?.load(Ordering::SeqCst);
        assert_eq!(0, new_idx, "value must be 0");

        db.compare_and_set_outgoing_ticket_index(hash, 1).await?;

        let new_idx = db.get_outgoing_ticket_index(hash).await?.load(Ordering::SeqCst);
        assert_eq!(1, new_idx, "new value must be 1");

        let old_idx = db.compare_and_set_outgoing_ticket_index(hash, 0).await?;
        assert_eq!(1, old_idx, "old value must be 1");
        assert_eq!(1, new_idx, "new value must be 1");

        let old_idx = db.compare_and_set_outgoing_ticket_index(hash, 1).await?;
        assert_eq!(1, old_idx, "old value must be 1");
        assert_eq!(1, new_idx, "new value must be 1");

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_index_reset() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let hash = Hash::default();

        let new_idx = db.get_outgoing_ticket_index(hash).await?.load(Ordering::SeqCst);
        assert_eq!(0, new_idx, "value must be 0");

        db.compare_and_set_outgoing_ticket_index(hash, 1).await?;

        let new_idx = db.get_outgoing_ticket_index(hash).await?.load(Ordering::SeqCst);
        assert_eq!(1, new_idx, "new value must be 1");

        let old_idx = db.reset_outgoing_ticket_index(hash).await?;
        assert_eq!(1, old_idx, "old value must be 1");

        let new_idx = db.get_outgoing_ticket_index(hash).await?.load(Ordering::SeqCst);
        assert_eq!(0, new_idx, "new value must be 0");
        Ok(())
    }

    #[tokio::test]
    async fn test_persist_ticket_indices() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let hash_1 = Hash::default();
        let hash_2 = Hash::from(hopr_crypto_random::random_bytes());

        db.get_outgoing_ticket_index(hash_1).await?;
        db.compare_and_set_outgoing_ticket_index(hash_2, 10).await?;

        let persisted = db.persist_outgoing_ticket_indices().await?;
        assert_eq!(1, persisted);

        let indices = hopr_db_entity::outgoing_ticket_index::Entity::find()
            .all(&db.tickets_db)
            .await?;
        let idx_1 = indices
            .iter()
            .find(|idx| idx.channel_id == hash_1.to_hex())
            .context("must contain index 1")?;
        let idx_2 = indices
            .iter()
            .find(|idx| idx.channel_id == hash_2.to_hex())
            .context("must contain index 2")?;
        assert_eq!(0, U256::from_be_bytes(&idx_1.index).as_u64(), "index must be 0");
        assert_eq!(10, U256::from_be_bytes(&idx_2.index).as_u64(), "index must be 10");

        db.compare_and_set_outgoing_ticket_index(hash_1, 3).await?;
        db.increment_outgoing_ticket_index(hash_2).await?;

        let persisted = db.persist_outgoing_ticket_indices().await?;
        assert_eq!(2, persisted);

        let indices = hopr_db_entity::outgoing_ticket_index::Entity::find()
            .all(&db.tickets_db)
            .await?;
        let idx_1 = indices
            .iter()
            .find(|idx| idx.channel_id == hash_1.to_hex())
            .context("must contain index 1")?;
        let idx_2 = indices
            .iter()
            .find(|idx| idx.channel_id == hash_2.to_hex())
            .context("must contain index 2")?;
        assert_eq!(3, U256::from_be_bytes(&idx_1.index).as_u64(), "index must be 3");
        assert_eq!(11, U256::from_be_bytes(&idx_2.index).as_u64(), "index must be 11");
        Ok(())
    }

    #[tokio::test]
    async fn test_cache_can_be_cloned_but_referencing_the_original_cache_storage() -> anyhow::Result<()> {
        let cache: moka::future::Cache<i64, i64> = moka::future::Cache::new(5);

        assert_eq!(cache.weighted_size(), 0);

        cache.insert(1, 1).await;
        cache.insert(2, 2).await;

        let clone = cache.clone();

        cache.remove(&1).await;
        cache.remove(&2).await;

        assert_eq!(cache.get(&1).await, None);
        assert_eq!(cache.get(&1).await, clone.get(&1).await);
        Ok(())
    }

    fn dummy_ticket_model(channel_id: Hash, idx: u64, idx_offset: u32, amount: u32) -> ticket::Model {
        ticket::Model {
            id: 0,
            channel_id: channel_id.to_string(),
            amount: U256::from(amount).to_be_bytes().to_vec(),
            index: idx.to_be_bytes().to_vec(),
            index_offset: idx_offset as i32,
            winning_probability: hex!("0020C49BA5E34F").to_vec(), // 0.0005
            channel_epoch: vec![],
            signature: vec![],
            response: vec![],
            state: 0,
            hash: vec![],
        }
    }

    #[tokio::test]
    async fn test_aggregation_prerequisites_default_filter_no_tickets() -> anyhow::Result<()> {
        let prerequisites = AggregationPrerequisites::default();
        assert_eq!(None, prerequisites.min_unaggregated_ratio);
        assert_eq!(None, prerequisites.min_ticket_count);

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            u32::MAX.into(),
            2.into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets = vec![dummy_ticket_model(channel.get_id(), 1, 1, 1)];

        let filtered_tickets =
            filter_satisfying_ticket_models(prerequisites, dummy_tickets.clone(), &channel, 0.0005.try_into()?)?;

        assert_eq!(
            dummy_tickets, filtered_tickets,
            "empty prerequisites must not filter anything"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_aggregation_prerequisites_should_filter_out_tickets_with_lower_than_min_win_prob()
    -> anyhow::Result<()> {
        let prerequisites = AggregationPrerequisites::default();
        assert_eq!(None, prerequisites.min_unaggregated_ratio);
        assert_eq!(None, prerequisites.min_ticket_count);

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            u32::MAX.into(),
            2.into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets = vec![dummy_ticket_model(channel.get_id(), 1, 1, 1)];

        let filtered_tickets =
            filter_satisfying_ticket_models(prerequisites, dummy_tickets.clone(), &channel, 0.0006.try_into()?)?;

        assert!(
            filtered_tickets.is_empty(),
            "must filter out tickets with lower win prob"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_aggregation_prerequisites_must_trim_tickets_exceeding_channel_balance() -> anyhow::Result<()> {
        const TICKET_COUNT: usize = 110;

        let prerequisites = AggregationPrerequisites::default();
        assert_eq!(None, prerequisites.min_unaggregated_ratio);
        assert_eq!(None, prerequisites.min_ticket_count);

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            100.into(),
            (TICKET_COUNT + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets: Vec<ticket::Model> = (0..TICKET_COUNT)
            .map(|i| dummy_ticket_model(channel.get_id(), i as u64, 1, 1))
            .collect();

        let filtered_tickets =
            filter_satisfying_ticket_models(prerequisites, dummy_tickets.clone(), &channel, 0.0005.try_into()?)?;

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
        Ok(())
    }

    #[tokio::test]
    async fn test_aggregation_prerequisites_must_return_empty_when_minimum_ticket_count_not_met() -> anyhow::Result<()>
    {
        const TICKET_COUNT: usize = 10;

        let prerequisites = AggregationPrerequisites {
            min_ticket_count: Some(TICKET_COUNT + 1),
            min_unaggregated_ratio: None,
        };

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            100.into(),
            (TICKET_COUNT + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets: Vec<ticket::Model> = (0..TICKET_COUNT)
            .map(|i| dummy_ticket_model(channel.get_id(), i as u64, 1, 1))
            .collect();

        let filtered_tickets =
            filter_satisfying_ticket_models(prerequisites, dummy_tickets.clone(), &channel, 0.0005.try_into()?)?;

        assert!(
            filtered_tickets.is_empty(),
            "must return empty when min_ticket_count is not met"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_aggregation_prerequisites_must_return_empty_when_minimum_unaggregated_ratio_is_not_met()
    -> anyhow::Result<()> {
        const TICKET_COUNT: usize = 10;

        let prerequisites = AggregationPrerequisites {
            min_ticket_count: None,
            min_unaggregated_ratio: Some(0.9),
        };

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            100.into(),
            (TICKET_COUNT + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets: Vec<ticket::Model> = (0..TICKET_COUNT)
            .map(|i| dummy_ticket_model(channel.get_id(), i as u64, 1, 1))
            .collect();

        let filtered_tickets =
            filter_satisfying_ticket_models(prerequisites, dummy_tickets.clone(), &channel, 0.0005.try_into()?)?;

        assert!(
            filtered_tickets.is_empty(),
            "must return empty when min_unaggregated_ratio is not met"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_aggregation_prerequisites_must_return_all_when_minimum_ticket_count_is_met() -> anyhow::Result<()> {
        const TICKET_COUNT: usize = 10;

        let prerequisites = AggregationPrerequisites {
            min_ticket_count: Some(TICKET_COUNT),
            min_unaggregated_ratio: None,
        };

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            100.into(),
            (TICKET_COUNT + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets: Vec<ticket::Model> = (0..TICKET_COUNT)
            .map(|i| dummy_ticket_model(channel.get_id(), i as u64, 1, 1))
            .collect();

        let filtered_tickets =
            filter_satisfying_ticket_models(prerequisites, dummy_tickets.clone(), &channel, 0.0005.try_into()?)?;

        assert!(!filtered_tickets.is_empty(), "must not return empty");
        assert_eq!(dummy_tickets, filtered_tickets, "return all tickets");
        Ok(())
    }

    #[tokio::test]
    async fn test_aggregation_prerequisites_must_return_all_when_minimum_ticket_count_is_met_regardless_ratio()
    -> anyhow::Result<()> {
        const TICKET_COUNT: usize = 10;

        let prerequisites = AggregationPrerequisites {
            min_ticket_count: Some(TICKET_COUNT),
            min_unaggregated_ratio: Some(0.9),
        };

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            100.into(),
            (TICKET_COUNT + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets: Vec<ticket::Model> = (0..TICKET_COUNT)
            .map(|i| dummy_ticket_model(channel.get_id(), i as u64, 1, 1))
            .collect();

        let filtered_tickets =
            filter_satisfying_ticket_models(prerequisites, dummy_tickets.clone(), &channel, 0.0005.try_into()?)?;

        assert!(!filtered_tickets.is_empty(), "must not return empty");
        assert_eq!(dummy_tickets, filtered_tickets, "return all tickets");
        Ok(())
    }

    #[tokio::test]
    async fn test_aggregation_prerequisites_must_return_all_when_minimum_unaggregated_ratio_is_met()
    -> anyhow::Result<()> {
        const TICKET_COUNT: usize = 90;

        let prerequisites = AggregationPrerequisites {
            min_ticket_count: None,
            min_unaggregated_ratio: Some(0.9),
        };

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            100.into(),
            (TICKET_COUNT + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets: Vec<ticket::Model> = (0..TICKET_COUNT)
            .map(|i| dummy_ticket_model(channel.get_id(), i as u64, 1, 1))
            .collect();

        let filtered_tickets =
            filter_satisfying_ticket_models(prerequisites, dummy_tickets.clone(), &channel, 0.0005.try_into()?)?;

        assert!(!filtered_tickets.is_empty(), "must not return empty");
        assert_eq!(dummy_tickets, filtered_tickets, "return all tickets");
        Ok(())
    }

    #[tokio::test]
    async fn test_aggregation_prerequisites_must_return_all_when_minimum_unaggregated_ratio_is_met_regardless_count()
    -> anyhow::Result<()> {
        const TICKET_COUNT: usize = 90;

        let prerequisites = AggregationPrerequisites {
            min_ticket_count: Some(TICKET_COUNT + 1),
            min_unaggregated_ratio: Some(0.9),
        };

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            100.into(),
            (TICKET_COUNT + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let dummy_tickets: Vec<ticket::Model> = (0..TICKET_COUNT)
            .map(|i| dummy_ticket_model(channel.get_id(), i as u64, 1, 1))
            .collect();

        let filtered_tickets =
            filter_satisfying_ticket_models(prerequisites, dummy_tickets.clone(), &channel, 0.0005.try_into()?)?;

        assert!(!filtered_tickets.is_empty(), "must not return empty");
        assert_eq!(dummy_tickets, filtered_tickets, "return all tickets");
        Ok(())
    }

    #[tokio::test]
    async fn test_aggregation_prerequisites_must_return_tickets_when_minimum_incl_aggregated_ratio_is_met()
    -> anyhow::Result<()> {
        const TICKET_COUNT: usize = 90;

        let prerequisites = AggregationPrerequisites {
            min_ticket_count: None,
            min_unaggregated_ratio: Some(0.9),
        };

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            100.into(),
            (TICKET_COUNT + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let mut dummy_tickets: Vec<ticket::Model> = (0..TICKET_COUNT)
            .map(|i| dummy_ticket_model(channel.get_id(), i as u64, 1, 1))
            .collect();
        dummy_tickets[0].index_offset = 2; // Make this ticket aggregated

        let filtered_tickets =
            filter_satisfying_ticket_models(prerequisites, dummy_tickets.clone(), &channel, 0.0005.try_into()?)?;

        assert_eq!(filtered_tickets.len(), TICKET_COUNT);
        Ok(())
    }

    #[tokio::test]
    async fn test_aggregation_prerequisites_must_return_empty_when_minimum_only_unaggregated_ratio_is_met_in_single_ticket_only()
    -> anyhow::Result<()> {
        let prerequisites = AggregationPrerequisites {
            min_ticket_count: None,
            min_unaggregated_ratio: Some(0.9),
        };

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            100.into(),
            2.into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        // Single aggregated ticket exceeding the min_unaggregated_ratio
        let dummy_tickets = vec![dummy_ticket_model(channel.get_id(), 1, 2, 110)];

        let filtered_tickets =
            filter_satisfying_ticket_models(prerequisites, dummy_tickets.clone(), &channel, 0.0005.try_into()?)?;

        assert!(filtered_tickets.is_empty(), "must return empty");
        Ok(())
    }

    async fn create_alice_db_with_tickets_from_bob(
        ticket_count: usize,
    ) -> anyhow::Result<(HoprDb, ChannelEntry, Vec<AcknowledgedTicket>)> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;

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

        let (channel, tickets) = init_db_with_tickets(&db, ticket_count as u64).await?;

        Ok((db, channel, tickets))
    }

    #[tokio::test]
    async fn test_ticket_aggregation_should_fail_if_any_ticket_is_being_aggregated_in_that_channel()
    -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 5;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        // mark the first ticket as being aggregated
        let mut ticket = hopr_db_entity::ticket::Entity::find()
            .one(&db.tickets_db)
            .await?
            .context("should have an active model")?
            .into_active_model();
        ticket.state = Set(AcknowledgedTicketStatus::BeingAggregated as i8);
        ticket.save(&db.tickets_db).await?;

        assert!(
            db.prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
                .await
                .is_err()
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_aggregation_should_not_offer_tickets_with_lower_than_min_win_prob() -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 5;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        // Decrease the win prob of one ticket
        let mut ticket = hopr_db_entity::ticket::Entity::find()
            .one(&db.tickets_db)
            .await?
            .context("should have an active model")?
            .into_active_model();
        ticket.winning_probability = Set(WinningProbability::try_from_f64(0.5)?.as_ref().to_vec());
        ticket.save(&db.tickets_db).await?;

        let prepared_tickets = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
            .await?
            .ok_or(anyhow!("should contain tickets"))?
            .1;

        assert_eq!(COUNT_TICKETS - 1, prepared_tickets.len());

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_aggregation_prepare_request_with_0_tickets_should_return_empty_result() -> anyhow::Result<()> {
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

    #[tokio::test]
    async fn test_ticket_aggregation_prepare_request_with_multiple_tickets_should_return_that_ticket()
    -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 2;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;
        let tickets: Vec<TransferableWinningTicket> = tickets
            .into_iter()
            .map(|t| t.into_transferable(&ALICE, &Hash::default()))
            .collect::<hopr_internal_types::errors::Result<Vec<TransferableWinningTicket>>>()?;

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
            .await?;

        assert_eq!(actual, Some((*BOB_OFFCHAIN.public(), tickets, Default::default())));

        let actual_being_aggregated_count = hopr_db_entity::ticket::Entity::find()
            .filter(hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .count(&db.tickets_db)
            .await? as usize;

        assert_eq!(actual_being_aggregated_count, COUNT_TICKETS);

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_aggregation_prepare_request_with_duplicate_tickets_should_return_dedup_aggregated_ticket()
    -> anyhow::Result<()> {
        let (db, channel, _) = create_alice_db_with_tickets_from_bob(0).await?;
        let tickets = vec![
            generate_random_ack_ticket(&BOB, &ALICE, 1, 1, 1.0),
            generate_random_ack_ticket(&BOB, &ALICE, 0, 2, 1.0),
            generate_random_ack_ticket(&BOB, &ALICE, 2, 1, 1.0),
            generate_random_ack_ticket(&BOB, &ALICE, 3, 1, 1.0),
        ]
        .into_iter()
        .collect::<anyhow::Result<Vec<AcknowledgedTicket>>>()?;

        let tickets_clone = tickets.clone();
        let db_clone = db.clone();
        db.nest_transaction_in_db(None, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    for ticket in tickets_clone {
                        db_clone.upsert_ticket(tx.into(), ticket).await?;
                    }
                    Ok::<_, DbError>(())
                })
            })
            .await?;

        let existing_channel_with_multiple_tickets = channel.get_id();
        let stats = db.get_ticket_statistics(Some(channel.get_id())).await?;
        assert_eq!(stats.neglected_value, HoprBalance::zero());

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
            .await?;

        let mut tickets = tickets
            .into_iter()
            .map(|t| t.into_transferable(&ALICE, &Hash::default()))
            .collect::<hopr_internal_types::errors::Result<Vec<_>>>()?;

        // We expect the first ticket to be removed
        tickets.remove(0);
        assert_eq!(actual, Some((*BOB_OFFCHAIN.public(), tickets, Default::default())));

        let actual_being_aggregated_count = hopr_db_entity::ticket::Entity::find()
            .filter(hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .count(&db.tickets_db)
            .await? as usize;

        assert_eq!(actual_being_aggregated_count, 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_aggregation_prepare_request_with_a_being_redeemed_ticket_should_aggregate_only_the_tickets_following_it()
    -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 5;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;
        let mut tickets = tickets
            .into_iter()
            .map(|t| t.into_transferable(&ALICE, &Hash::default()).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        // mark the first ticket as being redeemed
        let mut ticket = hopr_db_entity::ticket::Entity::find()
            .one(&db.tickets_db)
            .await?
            .context("should have 1 active model")?
            .into_active_model();
        ticket.state = Set(AcknowledgedTicketStatus::BeingRedeemed as i8);
        ticket.save(&db.tickets_db).await?;

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
            .await?;

        tickets.remove(0);
        assert_eq!(actual, Some((*BOB_OFFCHAIN.public(), tickets, Default::default())));

        let actual_being_aggregated_count = hopr_db_entity::ticket::Entity::find()
            .filter(hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .count(&db.tickets_db)
            .await? as usize;

        assert_eq!(actual_being_aggregated_count, COUNT_TICKETS - 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_aggregation_prepare_request_with_some_requirements_should_return_when_ticket_threshold_is_met()
    -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 5;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;
        let tickets = tickets
            .into_iter()
            .map(|t| t.into_transferable(&ALICE, &Hash::default()).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let constraints = AggregationPrerequisites {
            min_ticket_count: Some(COUNT_TICKETS - 1),
            min_unaggregated_ratio: None,
        };
        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, constraints)
            .await?;

        assert_eq!(actual, Some((*BOB_OFFCHAIN.public(), tickets, Default::default())));

        let actual_being_aggregated_count = hopr_db_entity::ticket::Entity::find()
            .filter(hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .count(&db.tickets_db)
            .await? as usize;

        assert_eq!(actual_being_aggregated_count, COUNT_TICKETS);

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_aggregation_prepare_request_with_some_requirements_should_not_return_when_ticket_threshold_is_not_met()
    -> anyhow::Result<()> {
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

    #[tokio::test]
    async fn test_ticket_aggregation_prepare_request_with_no_aggregatable_tickets_should_return_nothing()
    -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 3;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        assert_eq!(tickets.len(), { COUNT_TICKETS });

        let existing_channel_with_multiple_tickets = channel.get_id();

        // mark all tickets as being redeemed
        for ticket in hopr_db_entity::ticket::Entity::find()
            .all(&db.tickets_db)
            .await?
            .into_iter()
        {
            let mut ticket = ticket.into_active_model();
            ticket.state = Set(AcknowledgedTicketStatus::BeingRedeemed as i8);
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

    #[tokio::test]
    async fn test_ticket_aggregation_rollback_should_rollback_all_the_being_aggregated_tickets_but_nothing_else()
    -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 5;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        // mark the first ticket as being redeemed
        let mut ticket = hopr_db_entity::ticket::Entity::find()
            .one(&db.tickets_db)
            .await?
            .context("should have one active model")?
            .into_active_model();
        ticket.state = Set(AcknowledgedTicketStatus::BeingRedeemed as i8);
        ticket.save(&db.tickets_db).await?;

        assert!(
            db.prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
                .await
                .is_ok()
        );

        let actual_being_aggregated_count = hopr_db_entity::ticket::Entity::find()
            .filter(hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .count(&db.tickets_db)
            .await? as usize;

        assert_eq!(actual_being_aggregated_count, COUNT_TICKETS - 1);

        assert!(
            db.rollback_aggregation_in_channel(existing_channel_with_multiple_tickets)
                .await
                .is_ok()
        );

        let actual_being_aggregated_count = hopr_db_entity::ticket::Entity::find()
            .filter(hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .count(&db.tickets_db)
            .await? as usize;

        assert_eq!(actual_being_aggregated_count, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_aggregation_should_replace_the_tickets_with_a_correctly_aggregated_ticket()
    -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 5;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        let (notifier_tx, notifier_rx) = futures::channel::mpsc::unbounded();
        db.start_ticket_processing(Some(notifier_tx))?;

        let tickets = tickets
            .into_iter()
            .map(|t| t.into_transferable(&ALICE, &Hash::default()).unwrap())
            .collect::<Vec<_>>();

        let first_ticket = tickets.first().context("should contain tickets")?.ticket.clone();
        let aggregated_ticket = TicketBuilder::default()
            .addresses(&*BOB, &*ALICE)
            .amount(
                tickets
                    .iter()
                    .fold(U256::zero(), |acc, v| acc + v.ticket.amount.amount()),
            )
            .index(first_ticket.index)
            .index_offset(
                tickets.last().context("should contain tickets")?.ticket.index as u32 - first_ticket.index as u32 + 1,
            )
            .win_prob(1.0.try_into()?)
            .channel_epoch(first_ticket.channel_epoch)
            .eth_challenge(first_ticket.challenge)
            .build_signed(&BOB, &Hash::default())?;

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
            .await?;

        assert_eq!(actual, Some((*BOB_OFFCHAIN.public(), tickets, Default::default())));

        let agg_ticket = aggregated_ticket.clone();

        let _ = db
            .process_received_aggregated_ticket(aggregated_ticket.leak(), &ALICE)
            .await?;

        pin_mut!(notifier_rx);
        let notified_ticket = notifier_rx.next().await.ok_or(anyhow!("must have ticket"))?;

        assert_eq!(notified_ticket.verified_ticket(), agg_ticket.verified_ticket());

        let actual_being_aggregated_count = hopr_db_entity::ticket::Entity::find()
            .filter(ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
            .count(&db.tickets_db)
            .await? as usize;

        assert_eq!(actual_being_aggregated_count, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_aggregation_should_fail_if_the_aggregated_ticket_value_is_lower_than_the_stored_one()
    -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 5;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;
        let tickets = tickets
            .into_iter()
            .map(|t| t.into_transferable(&ALICE, &Hash::default()).unwrap())
            .collect::<Vec<_>>();

        let first_ticket = tickets.first().context("should contain tickets")?.ticket.clone();
        let aggregated_ticket = TicketBuilder::default()
            .addresses(&*BOB, &*ALICE)
            .amount(0)
            .index(first_ticket.index)
            .index_offset(
                tickets.last().context("should contain tickets")?.ticket.index as u32 - first_ticket.index as u32 + 1,
            )
            .win_prob(1.0.try_into()?)
            .channel_epoch(first_ticket.channel_epoch)
            .eth_challenge(first_ticket.challenge)
            .build_signed(&BOB, &Hash::default())?;

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
            .await?;

        assert_eq!(actual, Some((*BOB_OFFCHAIN.public(), tickets, Default::default())));

        assert!(
            db.process_received_aggregated_ticket(aggregated_ticket.leak(), &ALICE)
                .await
                .is_err()
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_aggregation_should_fail_if_the_aggregated_ticket_win_probability_is_not_equal_to_1()
    -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 5;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;
        let tickets = tickets
            .into_iter()
            .map(|t| t.into_transferable(&ALICE, &Hash::default()).unwrap())
            .collect::<Vec<_>>();

        let first_ticket = tickets.first().context("should contain tickets")?.ticket.clone();
        let aggregated_ticket = TicketBuilder::default()
            .addresses(&*BOB, &*ALICE)
            .amount(0)
            .index(first_ticket.index)
            .index_offset(
                tickets.last().context("should contain tickets")?.ticket.index as u32 - first_ticket.index as u32 + 1,
            )
            .win_prob(0.5.try_into()?) // 50% winning prob
            .channel_epoch(first_ticket.channel_epoch)
            .eth_challenge(first_ticket.challenge)
            .build_signed(&BOB, &Hash::default())?;

        assert_eq!(tickets.len(), COUNT_TICKETS);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, Default::default())
            .await?;

        assert_eq!(actual, Some((*BOB_OFFCHAIN.public(), tickets, Default::default())));

        assert!(
            db.process_received_aggregated_ticket(aggregated_ticket.leak(), &ALICE)
                .await
                .is_err()
        );

        Ok(())
    }

    async fn init_db_with_channel(channel: ChannelEntry) -> anyhow::Result<HoprDb> {
        let db = HoprDb::new_in_memory(BOB.clone()).await?;

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

        db.upsert_channel(None, channel).await?;

        Ok(db)
    }

    #[tokio::test]
    async fn test_aggregate_ticket_should_aggregate() -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 5;

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            u32::MAX.into(),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(120))),
            4_u32.into(),
        );

        let db = init_db_with_channel(channel).await?;

        let tickets = (0..COUNT_TICKETS)
            .map(|i| {
                generate_random_ack_ticket(&BOB, &ALICE, i as u64, 1, 1.0)
                    .and_then(|v| Ok(v.into_transferable(&ALICE, &Hash::default())?))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let sum_value = tickets.iter().fold(HoprBalance::zero(), |acc, x| acc + x.ticket.amount);
        let min_idx = tickets
            .iter()
            .map(|t| t.ticket.index)
            .min()
            .context("min index should be present")?;
        let max_idx = tickets
            .iter()
            .map(|t| t.ticket.index)
            .max()
            .context("max index should be present")?;

        let aggregated = db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets, &BOB).await?;

        assert_eq!(
            &BOB.public().to_address(),
            aggregated.verified_issuer(),
            "must have correct signer"
        );

        assert!(aggregated.verified_ticket().is_aggregated(), "must be aggregated");

        assert_eq!(
            COUNT_TICKETS,
            aggregated.verified_ticket().index_offset as usize,
            "aggregated ticket must have correct offset"
        );
        assert_eq!(
            sum_value,
            aggregated.verified_ticket().amount,
            "aggregated ticket token amount must be sum of individual tickets"
        );
        assert_eq!(
            1.0,
            aggregated.win_prob(),
            "aggregated ticket must have winning probability 1"
        );
        assert_eq!(
            min_idx,
            aggregated.verified_ticket().index,
            "aggregated ticket must have correct index"
        );
        assert_eq!(
            channel.get_id(),
            aggregated.verified_ticket().channel_id,
            "aggregated ticket must have correct channel id"
        );
        assert_eq!(
            channel.channel_epoch.as_u32(),
            aggregated.verified_ticket().channel_epoch,
            "aggregated ticket must have correct channel epoch"
        );

        assert_eq!(
            max_idx + 1,
            db.get_outgoing_ticket_index(channel.get_id())
                .await?
                .load(Ordering::SeqCst)
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_aggregate_ticket_should_aggregate_including_aggregated() -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 5;

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            u32::MAX.into(),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(120))),
            4_u32.into(),
        );

        let db = init_db_with_channel(channel).await?;

        let offset = 10_usize;

        let mut tickets = (1..COUNT_TICKETS)
            .map(|i| {
                generate_random_ack_ticket(&BOB, &ALICE, (i + offset) as u64, 1, 1.0)
                    .and_then(|v| Ok(v.into_transferable(&ALICE, &Hash::default())?))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        // Add an aggregated ticket to the set too
        tickets.push(
            generate_random_ack_ticket(&BOB, &ALICE, 0, offset as u32, 1.0)
                .and_then(|v| Ok(v.into_transferable(&ALICE, &Hash::default())?))?,
        );

        let sum_value = tickets.iter().fold(HoprBalance::zero(), |acc, x| acc + x.ticket.amount);
        let min_idx = tickets
            .iter()
            .map(|t| t.ticket.index)
            .min()
            .context("min index should be present")?;
        let max_idx = tickets
            .iter()
            .map(|t| t.ticket.index)
            .max()
            .context("max index should be present")?;

        let aggregated = db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets, &BOB).await?;

        assert_eq!(
            &BOB.public().to_address(),
            aggregated.verified_issuer(),
            "must have correct signer"
        );

        assert!(aggregated.verified_ticket().is_aggregated(), "must be aggregated");

        assert_eq!(
            COUNT_TICKETS + offset,
            aggregated.verified_ticket().index_offset as usize,
            "aggregated ticket must have correct offset"
        );
        assert_eq!(
            sum_value,
            aggregated.verified_ticket().amount,
            "aggregated ticket token amount must be sum of individual tickets"
        );
        assert_eq!(
            1.0,
            aggregated.win_prob(),
            "aggregated ticket must have winning probability 1"
        );
        assert_eq!(
            min_idx,
            aggregated.verified_ticket().index,
            "aggregated ticket must have correct index"
        );
        assert_eq!(
            channel.get_id(),
            aggregated.verified_ticket().channel_id,
            "aggregated ticket must have correct channel id"
        );
        assert_eq!(
            channel.channel_epoch.as_u32(),
            aggregated.verified_ticket().channel_epoch,
            "aggregated ticket must have correct channel epoch"
        );

        assert_eq!(
            max_idx + 1,
            db.get_outgoing_ticket_index(channel.get_id())
                .await?
                .load(Ordering::SeqCst)
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_aggregate_ticket_should_not_aggregate_zero_tickets() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(BOB.clone()).await?;

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), vec![], &BOB)
            .await
            .expect_err("should not aggregate empty ticket list");

        Ok(())
    }

    #[tokio::test]
    async fn test_aggregate_ticket_should_aggregate_single_ticket_to_itself() -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 1;

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            u32::MAX.into(),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(120))),
            4_u32.into(),
        );

        let db = init_db_with_channel(channel).await?;

        let mut tickets = (0..COUNT_TICKETS)
            .map(|i| {
                generate_random_ack_ticket(&BOB, &ALICE, i as u64, 1, 1.0)
                    .and_then(|v| Ok(v.into_transferable(&ALICE, &Hash::default())?))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let aggregated = db
            .aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await?;

        assert_eq!(
            &tickets.pop().context("ticket should be present")?.ticket,
            aggregated.verified_ticket()
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_aggregate_ticket_should_not_aggregate_on_closed_channel() -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 3;

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            u32::MAX.into(),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::Closed,
            4_u32.into(),
        );

        let db = init_db_with_channel(channel).await?;

        let tickets = (0..COUNT_TICKETS)
            .map(|i| {
                generate_random_ack_ticket(&BOB, &ALICE, i as u64, 1, 1.0)
                    .and_then(|v| Ok(v.into_transferable(&ALICE, &Hash::default())?))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await
            .expect_err("should not aggregate on closed channel");

        Ok(())
    }

    #[tokio::test]
    async fn test_aggregate_ticket_should_not_aggregate_on_incoming_channel() -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 3;

        let channel = ChannelEntry::new(
            ALICE.public().to_address(),
            BOB.public().to_address(),
            u32::MAX.into(),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let db = init_db_with_channel(channel).await?;

        let tickets = (0..COUNT_TICKETS)
            .map(|i| {
                generate_random_ack_ticket(&BOB, &ALICE, i as u64, 1, 1.0)
                    .and_then(|v| Ok(v.into_transferable(&ALICE, &Hash::default())?))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await
            .expect_err("should not aggregate on incoming channel");

        Ok(())
    }

    #[tokio::test]
    async fn test_aggregate_ticket_should_not_aggregate_if_mismatching_channel_ids() -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 3;

        let channel = ChannelEntry::new(
            ALICE.public().to_address(),
            BOB.public().to_address(),
            u32::MAX.into(),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let db = init_db_with_channel(channel).await?;

        let mut tickets = (0..COUNT_TICKETS)
            .map(|i| {
                generate_random_ack_ticket(&BOB, &ALICE, i as u64, 1, 1.0)
                    .and_then(|v| Ok(v.into_transferable(&ALICE, &Hash::default())?))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        tickets[2] = generate_random_ack_ticket(&BOB, &ChainKeypair::random(), 2, 1, 1.0)
            .and_then(|v| Ok(v.into_transferable(&ALICE, &Hash::default())?))?;

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await
            .expect_err("should not aggregate on mismatching channel ids");

        Ok(())
    }

    #[tokio::test]
    async fn test_aggregate_ticket_should_not_aggregate_if_mismatching_channel_epoch() -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 3;

        let channel = ChannelEntry::new(
            ALICE.public().to_address(),
            BOB.public().to_address(),
            100.into(),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::Open,
            3_u32.into(),
        );

        let db = init_db_with_channel(channel).await?;

        let tickets = (0..COUNT_TICKETS)
            .map(|i| {
                generate_random_ack_ticket(&BOB, &ALICE, i as u64, 1, 1.0)
                    .and_then(|v| Ok(v.into_transferable(&ALICE, &Hash::default())?))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await
            .expect_err("should not aggregate on mismatching channel epoch");

        Ok(())
    }

    #[tokio::test]
    async fn test_aggregate_ticket_should_not_aggregate_if_ticket_indices_overlap() -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 3;

        let channel = ChannelEntry::new(
            ALICE.public().to_address(),
            BOB.public().to_address(),
            u32::MAX.into(),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::Open,
            3_u32.into(),
        );

        let db = init_db_with_channel(channel).await?;

        let mut tickets = (0..COUNT_TICKETS)
            .map(|i| {
                generate_random_ack_ticket(&BOB, &ALICE, i as u64, 1, 1.0)
                    .and_then(|v| Ok(v.into_transferable(&ALICE, &Hash::default())?))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        tickets[1] = generate_random_ack_ticket(&BOB, &ALICE, 1, 2, 1.0)
            .and_then(|v| Ok(v.into_transferable(&ALICE, &Hash::default())?))?;

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await
            .expect_err("should not aggregate on overlapping ticket indices");
        Ok(())
    }

    #[tokio::test]
    async fn test_aggregate_ticket_should_not_aggregate_if_ticket_is_not_valid() -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 3;

        let channel = ChannelEntry::new(
            ALICE.public().to_address(),
            BOB.public().to_address(),
            u32::MAX.into(),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::Open,
            3_u32.into(),
        );

        let db = init_db_with_channel(channel).await?;

        let mut tickets = (0..COUNT_TICKETS)
            .map(|i| {
                generate_random_ack_ticket(&BOB, &ALICE, i as u64, 1, 1.0)
                    .and_then(|v| Ok(v.into_transferable(&ALICE, &Hash::default())?))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        // Modify the ticket and do not sign it
        tickets[1].ticket.amount = (TICKET_VALUE - 10).into();

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await
            .expect_err("should not aggregate on invalid tickets");

        Ok(())
    }

    #[tokio::test]
    async fn test_aggregate_ticket_should_not_aggregate_if_ticket_has_lower_than_min_win_prob() -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 3;

        let channel = ChannelEntry::new(
            ALICE.public().to_address(),
            BOB.public().to_address(),
            u32::MAX.into(),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::Open,
            3_u32.into(),
        );

        let db = init_db_with_channel(channel).await?;

        let tickets = (0..COUNT_TICKETS)
            .map(|i| {
                generate_random_ack_ticket(&BOB, &ALICE, i as u64, 1, if i > 0 { 1.0 } else { 0.9 })
                    .and_then(|v| Ok(v.into_transferable(&ALICE, &Hash::default())?))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await
            .expect_err("should not aggregate tickets with less than minimum win prob");

        Ok(())
    }

    #[tokio::test]
    async fn test_aggregate_ticket_should_not_aggregate_if_ticket_is_not_winning() -> anyhow::Result<()> {
        const COUNT_TICKETS: usize = 3;

        let channel = ChannelEntry::new(
            ALICE.public().to_address(),
            BOB.public().to_address(),
            u32::MAX.into(),
            (COUNT_TICKETS + 1).into(),
            ChannelStatus::Open,
            3_u32.into(),
        );

        let db = init_db_with_channel(channel).await?;

        let mut tickets = (0..COUNT_TICKETS)
            .map(|i| {
                generate_random_ack_ticket(&BOB, &ALICE, i as u64, 1, 1.0)
                    .and_then(|v| Ok(v.into_transferable(&ALICE, &Hash::default())?))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        // Set the winning probability to zero and sign the ticket again
        let resp = Response::from_half_keys(&HalfKey::random(), &HalfKey::random())?;
        tickets[1] = TicketBuilder::from(tickets[1].ticket.clone())
            .win_prob(0.0.try_into()?)
            .challenge(resp.to_challenge()?)
            .build_signed(&BOB, &Hash::default())?
            .into_acknowledged(resp)
            .into_transferable(&ALICE, &Hash::default())?;

        db.aggregate_tickets(*ALICE_OFFCHAIN.public(), tickets.clone(), &BOB)
            .await
            .expect_err("should not aggregate non-winning tickets");

        Ok(())
    }

    #[tokio::test]
    async fn test_set_ticket_statistics_when_tickets_are_in_db() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;

        let ticket = init_db_with_tickets(&db, 1).await?.1.pop().unwrap();

        db.mark_tickets_as((&ticket).into(), TicketMarker::Redeemed)
            .await
            .expect("must not fail");

        let stats = db.get_ticket_statistics(None).await.expect("must not fail");
        assert_ne!(stats.redeemed_value, HoprBalance::zero());

        db.reset_ticket_statistics().await.expect("must not fail");

        let stats = db.get_ticket_statistics(None).await.expect("must not fail");
        assert_eq!(stats.redeemed_value, HoprBalance::zero());

        Ok(())
    }

    #[tokio::test]
    async fn test_fix_channels_ticket_state() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        const COUNT_TICKETS: u64 = 1;

        let (..) = init_db_with_tickets(&db, COUNT_TICKETS).await?;

        // mark the first ticket as being redeemed
        let mut ticket = hopr_db_entity::ticket::Entity::find()
            .one(&db.tickets_db)
            .await?
            .context("should have one active model")?
            .into_active_model();
        ticket.state = Set(AcknowledgedTicketStatus::BeingRedeemed as i8);
        ticket.save(&db.tickets_db).await?;

        assert!(
            hopr_db_entity::ticket::Entity::find()
                .one(&db.tickets_db)
                .await?
                .context("should have one active model")?
                .state
                == AcknowledgedTicketStatus::BeingRedeemed as i8,
        );

        db.fix_channels_next_ticket_state().await.expect("must not fail");

        assert!(
            hopr_db_entity::ticket::Entity::find()
                .one(&db.tickets_db)
                .await?
                .context("should have one active model")?
                .state
                == AcknowledgedTicketStatus::Untouched as i8,
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_dont_fix_correct_channels_ticket_state() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        const COUNT_TICKETS: u64 = 2;

        // we set up the channel to have ticket index 1, and ensure that fix does not trigger
        let (..) = init_db_with_tickets_and_channel(&db, COUNT_TICKETS, Some(1u32), 1.0).await?;

        // mark the first ticket as being redeemed
        let mut ticket = hopr_db_entity::ticket::Entity::find()
            .filter(ticket::Column::Index.eq(0u64.to_be_bytes().to_vec()))
            .one(&db.tickets_db)
            .await?
            .context("should have one active model")?
            .into_active_model();
        ticket.state = Set(AcknowledgedTicketStatus::BeingRedeemed as i8);
        ticket.save(&db.tickets_db).await?;

        assert!(
            hopr_db_entity::ticket::Entity::find()
                .filter(ticket::Column::Index.eq(0u64.to_be_bytes().to_vec()))
                .one(&db.tickets_db)
                .await?
                .context("should have one active model")?
                .state
                == AcknowledgedTicketStatus::BeingRedeemed as i8,
        );

        db.fix_channels_next_ticket_state().await.expect("must not fail");

        // first ticket should still be in BeingRedeemed state
        let ticket0 = hopr_db_entity::ticket::Entity::find()
            .filter(ticket::Column::Index.eq(0u64.to_be_bytes().to_vec()))
            .one(&db.tickets_db)
            .await?
            .context("should have one active model")?;
        assert_eq!(ticket0.state, AcknowledgedTicketStatus::BeingRedeemed as i8);

        // second ticket should be in Untouched state
        let ticket1 = hopr_db_entity::ticket::Entity::find()
            .filter(ticket::Column::Index.eq(1u64.to_be_bytes().to_vec()))
            .one(&db.tickets_db)
            .await?
            .context("should have one active model")?;
        assert_eq!(ticket1.state, AcknowledgedTicketStatus::Untouched as i8);

        Ok(())
    }
}
