use std::{collections::HashMap, ops::Bound};

use async_stream::stream;
use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt, stream::BoxStream};
use hopr_api::db::*;
use hopr_db_entity::{outgoing_ticket_index, ticket, ticket_statistics};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, QuerySelect, Set,
    TransactionTrait,
};
use sea_query::{Condition, Expr, ExprTrait, IntoCondition, Order};
use tracing::{debug, error, info, trace};

use crate::{db::HoprNodeDb, errors::NodeDbError};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    pub static ref METRIC_HOPR_TICKETS_INCOMING_STATISTICS: hopr_metrics::MultiGauge = hopr_metrics::MultiGauge::new(
        "hopr_tickets_incoming_statistics",
        "Ticket statistics for channels with incoming tickets.",
        &["statistic"]
    ).unwrap();
}

/// The type is necessary solely to allow
/// implementing the [`IntoCondition`] trait for the [`TicketSelector`]
/// from the `hopr_api` crate.
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
        let (channel_id, epoch) = self.0.channel_identifier;

        let mut expr = ticket::Column::ChannelId
            .eq(hex::encode(channel_id))
            .and(ticket::Column::ChannelEpoch.eq(epoch.to_be_bytes().to_vec()));

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

/// Returns a condition satisfied if any of the given selectors is satisfied.
pub(crate) fn any_selector<I: IntoIterator<Item = S>, S: Into<TicketSelector>>(selectors: I) -> Condition {
    selectors
        .into_iter()
        .map(|s| WrappedTicketSelector(s.into()).into_condition())
        .reduce(|a, b| a.or(b).into_condition())
        .unwrap_or(Condition::all())
}

pub(crate) async fn find_stats_for_channel(
    tx: &sea_orm::DatabaseTransaction,
    channel_id: &ChannelId,
) -> Result<ticket_statistics::Model, NodeDbError> {
    if let Some(model) = ticket_statistics::Entity::find()
        .filter(ticket_statistics::Column::ChannelId.eq(hex::encode(channel_id)))
        .one(tx)
        .await?
    {
        Ok(model)
    } else {
        let new_stats = ticket_statistics::ActiveModel {
            channel_id: Set(hex::encode(channel_id)),
            ..Default::default()
        }
        .insert(tx)
        .await?;

        Ok(new_stats)
    }
}

pub(crate) async fn get_tickets_value_int(
    tx: &impl TransactionTrait,
    selector: TicketSelector,
) -> Result<(usize, HoprBalance), NodeDbError> {
    let selector: WrappedTicketSelector = selector.into();
    Ok(tx
        .transaction(|tx| {
            Box::pin(async move {
                ticket::Entity::find()
                    .filter(selector)
                    .stream(tx)
                    .await?
                    .try_fold((0_usize, HoprBalance::zero()), |(count, value), t| async move {
                        Ok((count + 1, value + HoprBalance::from_be_bytes(t.amount)))
                    })
                    .await
            })
        })
        .await?)
}

#[async_trait]
impl HoprDbTicketOperations for HoprNodeDb {
    type Error = NodeDbError;

    async fn stream_tickets<'c, S: Into<TicketSelector>, I: IntoIterator<Item = S> + Send>(
        &'c self,
        selectors: I,
    ) -> Result<BoxStream<'c, RedeemableTicket>, Self::Error> {
        let qry = ticket::Entity::find().filter(any_selector(selectors));

        Ok(qry
            .order_by(ticket::Column::ChannelId, Order::Asc)
            .order_by(ticket::Column::ChannelEpoch, Order::Asc)
            .order_by(ticket::Column::Index, Order::Asc)
            .stream(&self.tickets_db)
            .await?
            .and_then(|model| {
                futures::future::ready(
                    RedeemableTicket::try_from(model).map_err(|e| sea_orm::DbErr::Custom(e.to_string())),
                )
            })
            .filter_map(|ticket| {
                futures::future::ready(ticket.inspect_err(|error| error!(%error, "invalid ticket in db")).ok())
            })
            .boxed())
    }

    async fn insert_ticket(&self, ticket: RedeemableTicket) -> Result<(), Self::Error> {
        let _lock = self.tickets_write_lock.lock().await;
        let unrealized_value = self.unrealized_value.clone();
        self.tickets_db
            .transaction(|tx| {
                Box::pin(async move {
                    // Insertion of a new acknowledged ticket
                    ticket::ActiveModel::from(ticket).insert(tx).await?;

                    // Update the ticket winning count in the statistics
                    let model = find_stats_for_channel(tx, ticket.ticket.channel_id()).await?;

                    let winning_tickets = model.winning_tickets + 1;
                    let mut active_model = model.into_active_model();
                    active_model.winning_tickets = sea_orm::Set(winning_tickets);
                    active_model.save(tx).await?;

                    #[cfg(all(feature = "prometheus", not(test)))]
                    {
                        METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                            &["unredeemed"],
                            get_tickets_value_int(tx, TicketSelector::from(&ticket).only_channel())
                                .await?
                                .1
                                .amount()
                                .as_u128() as f64,
                        );
                        METRIC_HOPR_TICKETS_INCOMING_STATISTICS.increment(&["winning_count"], 1.0f64);
                    }

                    // Increase the unrealized value in the corresponding channel
                    unrealized_value
                        .entry((*ticket.ticket.channel_id(), ticket.verified_ticket().channel_epoch))
                        .and_compute_with(|value| match value {
                            Some(value) => futures::future::ready(moka::ops::compute::Op::Put(
                                value.into_value() + ticket.verified_ticket().amount,
                            )),
                            None => {
                                futures::future::ready(moka::ops::compute::Op::Put(ticket.verified_ticket().amount))
                            }
                        })
                        .await;

                    Ok::<_, NodeDbError>(())
                })
            })
            .await?;

        Ok(())
    }

    async fn mark_tickets_as<S: Into<TicketSelector> + Send, I: IntoIterator<Item = S> + Send>(
        &self,
        selectors: I,
        mark_as: TicketMarker,
    ) -> Result<usize, Self::Error> {
        let selectors = selectors.into_iter().map(|s| s.into()).collect::<Vec<_>>();
        let unrealized_value = self.unrealized_value.clone();
        let _lock = self.tickets_write_lock.lock().await;

        let (total_count, marked_values) = self
            .tickets_db
            .transaction(|tx| {
                Box::pin(async move {
                    let mut total_marked_count = 0;
                    let mut marked_values = Vec::new();
                    for channel_selector in selectors {
                        // Get the number of tickets and their value just for this channel
                        let (marked_count, marked_value) =
                            get_tickets_value_int(tx, channel_selector.clone()).await?;
                        trace!(marked_count, ?marked_value, ?mark_as, "ticket marking");

                        if marked_count > 0 {
                            // Delete the redeemed tickets first
                            let deleted = ticket::Entity::delete_many()
                                .filter(WrappedTicketSelector::from(channel_selector.clone()))
                                .exec(tx)
                                .await?;

                            // Update the stats if successful
                            if deleted.rows_affected == marked_count as u64 {
                                let mut new_stats = find_stats_for_channel(tx, &channel_selector.channel_identifier.0)
                                    .await?
                                    .into_active_model();

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
                                new_stats.save(tx).await?;

                                #[cfg(all(feature = "prometheus", not(test)))]
                                {
                                    METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                                        &[&mark_as.to_string()],
                                        (_current_value + marked_value.amount()).as_u128() as f64,
                                    );

                                    // Tickets that are counted as rejected were never counted as unredeemed,
                                    // so skip the metric subtraction in that case.
                                    if mark_as != TicketMarker::Rejected {
                                        let unredeemed_value = get_tickets_value_int(tx, channel_selector.clone())
                                            .await
                                            .map(|(_, value)| value)
                                            .unwrap_or_default();

                                        METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                                            &["unredeemed"],
                                            (unredeemed_value - marked_value.amount()).amount().as_u128() as f64,
                                        );
                                    }
                                }
                            } else {
                                return Err(NodeDbError::LogicalError(format!(
                                    "could not mark {marked_count} ticket as {mark_as}"
                                )));
                            }

                            trace!(marked_count, channel_id = ?channel_selector.channel_identifier.0, ?mark_as, "removed tickets in channel");
                            total_marked_count += marked_count;
                            marked_values.push((channel_selector.channel_identifier.0, channel_selector.channel_identifier.1, marked_value));
                        }
                    }

                    info!(count = total_marked_count, ?mark_as, "removed tickets in channels",);
                    Ok((total_marked_count, marked_values))
                })
            })
            .await?;

        // Decrease the unrealized value in each channel.
        // This must happen no matter if tickets have been neglected, rejected or redeemed.
        for (channel_id, epoch, removed_amount) in marked_values {
            unrealized_value
                .entry((channel_id, epoch))
                .and_compute_with(|value| match value {
                    Some(value) => {
                        futures::future::ready(moka::ops::compute::Op::Put(value.into_value() - removed_amount))
                    }
                    None => futures::future::ready(moka::ops::compute::Op::Nop),
                })
                .await;
        }

        Ok(total_count)
    }

    async fn mark_unsaved_ticket_rejected(&self, issuer: &Address, ticket: &Ticket) -> Result<(), NodeDbError> {
        let channel_id = generate_channel_id(issuer, &ticket.counterparty);
        let amount = ticket.amount;
        let _lock = self.tickets_write_lock.lock().await;

        Ok(self
            .tickets_db
            .transaction(|tx| {
                Box::pin(async move {
                    let stats = find_stats_for_channel(tx, &channel_id).await?;

                    let current_rejected_value = U256::from_be_bytes(stats.rejected_value.clone());

                    let mut active_stats = stats.into_active_model();
                    active_stats.rejected_value = Set((current_rejected_value + amount.amount()).to_be_bytes().into());
                    active_stats.save(tx).await?;

                    #[cfg(all(feature = "prometheus", not(test)))]
                    {
                        METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                            &["rejected"],
                            (current_rejected_value + amount.amount()).as_u128() as f64,
                        );
                    }

                    Ok::<(), NodeDbError>(())
                })
            })
            .await?)
    }

    async fn update_ticket_states_and_fetch<'a, S: Into<TicketSelector>, I: IntoIterator<Item = S> + Send>(
        &'a self,
        selectors: I,
        new_state: AcknowledgedTicketStatus,
    ) -> Result<BoxStream<'a, RedeemableTicket>, Self::Error> {
        let selector = any_selector(selectors);
        Ok(Box::pin(stream! {
            // The stream holds the write lock until it is consumed
            let _lock = self.tickets_write_lock.lock().await;

            match ticket::Entity::find()
                .filter(selector)
                .order_by(ticket::Column::ChannelId, Order::Asc)
                .order_by(ticket::Column::ChannelEpoch, Order::Asc)
                .order_by(ticket::Column::Index, Order::Asc)
                .stream(&self.tickets_db)
                .await {
                Ok(mut stream) => {
                    while let Ok(Some(ticket)) = stream.try_next().await {
                        let active_ticket = ticket::ActiveModel {
                            id: Set(ticket.id),
                            state: Set(new_state as i8),
                            ..Default::default()
                        };

                        {
                            if let Err(error) = active_ticket.update(&self.tickets_db).await {
                                error!(%error, "failed to update ticket in the db");
                            }
                        }

                        match RedeemableTicket::try_from(ticket) {
                            Ok(ticket) => {
                                yield ticket
                            },
                            Err(error) => {
                                tracing::error!(%error, "failed to decode ticket from the db");
                            }
                        }
                    }
                },
                Err(error) => tracing::error!(%error, "failed open ticket db stream")
            }
        }))
    }

    async fn update_ticket_states<S: Into<TicketSelector>, I: IntoIterator<Item = S> + Send>(
        &self,
        selectors: I,
        new_state: AcknowledgedTicketStatus,
    ) -> Result<usize, Self::Error> {
        let selector = any_selector(selectors);
        let _lock = self.tickets_write_lock.lock().await;
        Ok(self
            .tickets_db
            .transaction(|tx| {
                Box::pin(async move {
                    ticket::Entity::update_many()
                        .filter(selector)
                        .col_expr(ticket::Column::State, Expr::value(new_state as i8))
                        .exec(tx)
                        .await
                        .map(|update| update.rows_affected as usize)
                })
            })
            .await?)
    }

    async fn get_ticket_statistics(
        &self,
        channel_id: Option<ChannelId>,
    ) -> Result<ChannelTicketStatistics, NodeDbError> {
        let res = match channel_id {
            None => {
                self.tickets_db
                    .transaction(|tx| {
                        Box::pin(async move {
                            let unredeemed_value = ticket::Entity::find()
                                .stream(tx)
                                .await?
                                .try_fold(U256::zero(), |amount, x| {
                                    let unredeemed_value = U256::from_be_bytes(x.amount);
                                    futures::future::ok(amount + unredeemed_value)
                                })
                                .await?;

                            #[cfg(all(feature = "prometheus", not(test)))]
                            METRIC_HOPR_TICKETS_INCOMING_STATISTICS
                                .set(&["unredeemed"], unredeemed_value.as_u128() as f64);

                            let mut all_stats = ticket_statistics::Entity::find().all(tx).await?.into_iter().fold(
                                ChannelTicketStatistics::default(),
                                |mut acc, stats| {
                                    let neglected_value = HoprBalance::from_be_bytes(stats.neglected_value);
                                    acc.finalized_values
                                        .entry(TicketMarker::Neglected)
                                        .and_modify(|b| *b += neglected_value)
                                        .or_insert(neglected_value);
                                    let redeemed_value = HoprBalance::from_be_bytes(stats.redeemed_value);
                                    acc.finalized_values
                                        .entry(TicketMarker::Redeemed)
                                        .and_modify(|b| *b += redeemed_value)
                                        .or_insert(redeemed_value);
                                    let rejected_value = HoprBalance::from_be_bytes(stats.rejected_value);
                                    acc.finalized_values
                                        .entry(TicketMarker::Rejected)
                                        .and_modify(|b| *b += rejected_value)
                                        .or_insert(rejected_value);
                                    acc.winning_tickets += stats.winning_tickets as u128;
                                    acc
                                },
                            );

                            all_stats.unredeemed_value = unredeemed_value.into();

                            #[cfg(all(feature = "prometheus", not(test)))]
                            {
                                METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                                    &["neglected"],
                                    all_stats
                                        .finalized_values
                                        .get(&TicketMarker::Neglected)
                                        .copied()
                                        .unwrap_or_default()
                                        .amount()
                                        .as_u128() as f64,
                                );
                                METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                                    &["redeemed"],
                                    all_stats
                                        .finalized_values
                                        .get(&TicketMarker::Redeemed)
                                        .copied()
                                        .unwrap_or_default()
                                        .amount()
                                        .as_u128() as f64,
                                );
                                METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                                    &["rejected"],
                                    all_stats
                                        .finalized_values
                                        .get(&TicketMarker::Rejected)
                                        .copied()
                                        .unwrap_or_default()
                                        .amount()
                                        .as_u128() as f64,
                                );
                                METRIC_HOPR_TICKETS_INCOMING_STATISTICS
                                    .set(&["winning_tickets"], all_stats.winning_tickets as f64);
                            }
                            Ok::<_, NodeDbError>(all_stats)
                        })
                    })
                    .await
            }
            Some(channel) => {
                let _lock = self.tickets_write_lock.lock().await;

                self.tickets_db
                    .transaction(|tx| {
                        Box::pin(async move {
                            let stats = find_stats_for_channel(tx, &channel).await?;
                            let unredeemed_value = ticket::Entity::find()
                                .filter(ticket::Column::ChannelId.eq(hex::encode(channel)))
                                .stream(tx)
                                .await?
                                .try_fold(U256::zero(), |amount, x| {
                                    futures::future::ok(amount + U256::from_be_bytes(x.amount))
                                })
                                .await?;

                            Ok::<_, NodeDbError>(ChannelTicketStatistics {
                                winning_tickets: stats.winning_tickets as u128,
                                unredeemed_value: unredeemed_value.into(),
                                finalized_values: HashMap::from_iter([
                                    (
                                        TicketMarker::Neglected,
                                        HoprBalance::from_be_bytes(stats.neglected_value),
                                    ),
                                    (TicketMarker::Redeemed, HoprBalance::from_be_bytes(stats.redeemed_value)),
                                    (TicketMarker::Rejected, HoprBalance::from_be_bytes(stats.rejected_value)),
                                ]),
                            })
                        })
                    })
                    .await
            }
        };
        debug!(stats = ?res, "retrieved ticket statistics");
        Ok(res?)
    }

    async fn reset_ticket_statistics(&self) -> Result<(), NodeDbError> {
        let _lock = self.tickets_write_lock.lock().await;

        Ok(self
            .tickets_db
            .transaction(|tx| {
                Box::pin(async move {
                    // delete statistics for the found rows
                    let deleted = ticket_statistics::Entity::delete_many().exec(tx).await?;

                    #[cfg(all(feature = "prometheus", not(test)))]
                    {
                        if deleted.rows_affected > 0 {
                            METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(&["neglected"], 0.0_f64);
                            METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(&["redeemed"], 0.0_f64);
                            METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(&["rejected"], 0.0_f64);
                        }
                    }

                    debug!("reset ticket statistics for {:} channel(s)", deleted.rows_affected);

                    Ok::<_, sea_orm::DbErr>(())
                })
            })
            .await?)
    }

    async fn get_tickets_value(&self, id: &ChannelId, epoch: u32) -> Result<HoprBalance, NodeDbError> {
        Ok(self
            .unrealized_value
            .try_get_with((*id, epoch), async {
                get_tickets_value_int(&self.tickets_db, TicketSelector::new(*id, epoch))
                    .await
                    .map(|(_, value)| value)
            })
            .await?)
    }

    async fn get_or_create_outgoing_ticket_index(
        &self,
        channel_id: &ChannelId,
        epoch: u32,
    ) -> Result<Option<u64>, Self::Error> {
        let _lock = self.tickets_write_lock.lock().await;
        let channel_id = hex::encode(channel_id);

        Ok(self
            .tickets_db
            .transaction(|tx| {
                Box::pin(async move {
                    // Delete all older epochs
                    outgoing_ticket_index::Entity::delete_many()
                        .filter(
                            outgoing_ticket_index::Column::ChannelId
                                .eq(&channel_id)
                                .and(outgoing_ticket_index::Column::Epoch.lt(epoch)),
                        )
                        .exec(tx)
                        .await?;

                    // Check if there are no newer epochs for this channel
                    let newer_epoch = outgoing_ticket_index::Entity::find()
                        .filter(
                            outgoing_ticket_index::Column::ChannelId
                                .eq(&channel_id)
                                .and(outgoing_ticket_index::Column::Epoch.gt(epoch)),
                        )
                        .limit(1)
                        .one(tx)
                        .await?;

                    if let Some(newer_epoch) = newer_epoch {
                        return Err(NodeDbError::LogicalError(format!(
                            "attempted to get or insert outgoing index for older epoch {epoch} < {} in channel \
                             {channel_id}",
                            newer_epoch.epoch
                        )));
                    }

                    // Look if this epoch already has an existing index
                    let maybe_index = outgoing_ticket_index::Entity::find()
                        .filter(
                            outgoing_ticket_index::Column::ChannelId
                                .eq(&channel_id)
                                .and(outgoing_ticket_index::Column::Epoch.eq(epoch)),
                        )
                        .order_by_asc(outgoing_ticket_index::Column::Epoch)
                        .one(tx)
                        .await?;

                    Ok(match maybe_index {
                        Some(model) => Some(u64::from_be_bytes(model.index.try_into().map_err(|_| {
                            NodeDbError::LogicalError(format!(
                                "could not convert outgoing ticket index to u64 for channel {channel_id}"
                            ))
                        })?)),
                        None => {
                            outgoing_ticket_index::ActiveModel {
                                channel_id: Set(channel_id),
                                epoch: Set(epoch as i32),
                                ..Default::default()
                            }
                            .insert(tx)
                            .await?;

                            None
                        }
                    })
                })
            })
            .await?)
    }

    async fn update_outgoing_ticket_index(
        &self,
        channel_id: &ChannelId,
        epoch: u32,
        index: u64,
    ) -> Result<(), Self::Error> {
        let _lock = self.tickets_write_lock.lock().await;
        let channel_id = hex::encode(channel_id);
        Ok(self
            .tickets_db
            .transaction(|tx| {
                Box::pin(async move {
                    let maybe_index = outgoing_ticket_index::Entity::find()
                        .filter(
                            outgoing_ticket_index::Column::ChannelId
                                .eq(&channel_id)
                                .and(outgoing_ticket_index::Column::Epoch.eq(epoch)),
                        )
                        .one(tx)
                        .await?;

                    if let Some(model) = maybe_index {
                        let current_index = U256::from_be_bytes(model.index.as_slice()).as_u64();
                        if current_index > index {
                            return Err(NodeDbError::LogicalError(format!(
                                "cannot set outgoing ticket index to {index} for channel {channel_id} - index is \
                                 already {current_index}"
                            )));
                        }

                        // Save us the writing, if the indices are equal.
                        if current_index != index {
                            let mut active_model = model.into_active_model();
                            active_model.index = Set(index.to_be_bytes().to_vec());
                            active_model.save(tx).await?;

                            tracing::debug!(%channel_id, %epoch, %index, "updated outgoing ticket index");
                        }
                    } else {
                        tracing::debug!("ignoring attempt to update outgoing ticket index for non-existing entry");
                    }

                    Ok::<_, NodeDbError>(())
                })
            })
            .await?)
    }

    async fn remove_outgoing_ticket_index(&self, channel_id: &ChannelId, epoch: u32) -> Result<(), Self::Error> {
        let _lock = self.tickets_write_lock.lock().await;
        let res = outgoing_ticket_index::Entity::delete_many()
            .filter(
                outgoing_ticket_index::Column::ChannelId
                    .eq(hex::encode(channel_id))
                    .and(outgoing_ticket_index::Column::Epoch.eq(epoch)),
            )
            .exec(&self.tickets_db)
            .await?;

        if res.rows_affected > 0 {
            tracing::debug!(%channel_id, %epoch, "removed outgoing ticket index");
        } else {
            tracing::warn!(%channel_id, %epoch, "outgoing ticket index not found");
        }

        Ok(())
    }
}

#[cfg(test)]
impl HoprNodeDb {
    pub(crate) async fn upsert_ticket(&self, acknowledged_ticket: RedeemableTicket) -> Result<(), NodeDbError> {
        let _lock = self.tickets_write_lock.lock().await;

        self.tickets_db
            .transaction(|tx| {
                Box::pin(async move {
                    // For upserting, we must select only by the triplet (channel id, epoch, index)
                    let selector = WrappedTicketSelector::from(TicketSelector::from(&acknowledged_ticket));

                    debug!(%acknowledged_ticket, "upserting ticket");
                    let mut model = ticket::ActiveModel::from(acknowledged_ticket);

                    if let Some(ticket) = ticket::Entity::find().filter(selector).one(tx).await? {
                        model.id = Set(ticket.id);
                    }

                    model.save(tx).await
                })
            })
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use futures::StreamExt;
    use hex_literal::hex;
    use hopr_api::db::{ChannelTicketStatistics, TicketMarker};
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;

    use crate::{
        db::HoprNodeDb,
        tickets::{HoprDbTicketOperations, TicketSelector},
    };

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).expect("lazy static keypair should be valid");
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).expect("lazy static keypair should be valid");
        static ref CHANNEL_ID: Hash = generate_channel_id(BOB.public().as_ref(), ALICE.public().as_ref());
    }

    lazy_static::lazy_static! {
        static ref ALICE_OFFCHAIN: OffchainKeypair = OffchainKeypair::random();
        static ref BOB_OFFCHAIN: OffchainKeypair = OffchainKeypair::random();
    }

    const TICKET_VALUE: u64 = 100_000;
    const CHANNEL_EPOCH: u32 = 4;

    fn generate_random_ack_ticket(
        src: &ChainKeypair,
        dst: &ChainKeypair,
        index: u64,
        _index_offset: u32,
        win_prob: f64,
    ) -> anyhow::Result<RedeemableTicket> {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();
        let challenge = Response::from_half_keys(&hk1, &hk2)?.to_challenge()?;

        Ok(TicketBuilder::default()
            .counterparty(dst)
            .amount(TICKET_VALUE)
            .index(index)
            .win_prob(win_prob.try_into()?)
            .channel_epoch(CHANNEL_EPOCH)
            .challenge(challenge)
            .build_signed(src, &Hash::default())?
            .into_acknowledged(Response::from_half_keys(&hk1, &hk2)?)
            .into_redeemable(dst, &Hash::default())?)
    }

    async fn init_db_with_tickets(db: &HoprNodeDb, count_tickets: u64) -> anyhow::Result<Vec<RedeemableTicket>> {
        let tickets: Vec<RedeemableTicket> = (0..count_tickets)
            .map(|i| generate_random_ack_ticket(&BOB, &ALICE, i, 1, 1.0))
            .collect::<anyhow::Result<Vec<RedeemableTicket>>>()?;

        for t in &tickets {
            db.upsert_ticket(t.clone()).await?;
        }

        Ok(tickets)
    }

    #[test_log::test(tokio::test)]
    async fn test_insert_get_ticket() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;

        let mut tickets = init_db_with_tickets(&db, 1).await?;
        let ack_ticket = tickets.pop().context("ticket should be present")?;

        assert_eq!(*CHANNEL_ID, *ack_ticket.ticket.channel_id(), "channel ids must match");
        assert_eq!(
            CHANNEL_EPOCH,
            ack_ticket.verified_ticket().channel_epoch,
            "epochs must match"
        );

        let db_ticket = db
            .stream_tickets([&ack_ticket])
            .await?
            .collect::<Vec<_>>()
            .await
            .first()
            .cloned()
            .context("ticket should exist 1")?;

        assert_eq!(ack_ticket, db_ticket, "tickets must be equal");

        let db_ticket = db
            .stream_tickets(None::<TicketSelector>)
            .await?
            .collect::<Vec<_>>()
            .await
            .first()
            .cloned()
            .context("ticket should exist 2")?;

        assert_eq!(ack_ticket, db_ticket, "tickets must be equal");

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_redeemed() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;
        const COUNT_TICKETS: u64 = 10;

        let tickets = init_db_with_tickets(&db, COUNT_TICKETS).await?;

        let stats = db.get_ticket_statistics(None).await?;
        assert_eq!(
            HoprBalance::from(TICKET_VALUE * COUNT_TICKETS),
            stats.unredeemed_value,
            "unredeemed balance must match"
        );
        assert_eq!(
            HoprBalance::zero(),
            stats.redeemed_value(),
            "there must be 0 redeemed value"
        );

        assert_eq!(
            stats,
            db.get_ticket_statistics(Some(*CHANNEL_ID)).await?,
            "per channel stats must be same"
        );

        const TO_REDEEM: u64 = 2;
        for ticket in tickets.iter().take(TO_REDEEM as usize) {
            let r = db.mark_tickets_as([ticket], TicketMarker::Redeemed).await?;
            assert_eq!(1, r, "must redeem only a single ticket");
        }

        let stats = db.get_ticket_statistics(None).await?;
        assert_eq!(
            HoprBalance::from(TICKET_VALUE * (COUNT_TICKETS - TO_REDEEM)),
            stats.unredeemed_value,
            "unredeemed balance must match"
        );
        assert_eq!(
            HoprBalance::from(TICKET_VALUE * TO_REDEEM),
            stats.redeemed_value(),
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
        let db = HoprNodeDb::new_in_memory().await?;

        let ticket = init_db_with_tickets(&db, 1)
            .await?
            .pop()
            .context("should contain a ticket")?;

        db.mark_tickets_as([&ticket], TicketMarker::Redeemed).await?;
        assert_eq!(0, db.mark_tickets_as([&ticket], TicketMarker::Redeemed).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_redeem_should_redeem_all_tickets() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;

        let count_tickets = 10;
        init_db_with_tickets(&db, count_tickets).await?;

        let count_marked = db
            .mark_tickets_as(
                [TicketSelector::new(*CHANNEL_ID, CHANNEL_EPOCH)],
                TicketMarker::Redeemed,
            )
            .await?;
        assert_eq!(count_tickets, count_marked as u64, "must mark all tickets in channel");

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_tickets_neglected() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;
        const COUNT_TICKETS: u64 = 10;

        init_db_with_tickets(&db, COUNT_TICKETS).await?;

        let stats = db.get_ticket_statistics(None).await?;
        assert_eq!(
            HoprBalance::from(TICKET_VALUE * COUNT_TICKETS),
            stats.unredeemed_value,
            "unredeemed balance must match"
        );
        assert_eq!(
            HoprBalance::zero(),
            stats.neglected_value(),
            "there must be 0 redeemed value"
        );

        assert_eq!(
            stats,
            db.get_ticket_statistics(Some(*CHANNEL_ID)).await?,
            "per channel stats must be same"
        );

        db.mark_tickets_as(
            [TicketSelector::new(*CHANNEL_ID, CHANNEL_EPOCH)],
            TicketMarker::Neglected,
        )
        .await?;

        let stats = db.get_ticket_statistics(None).await?;
        assert_eq!(
            HoprBalance::zero(),
            stats.unredeemed_value,
            "unredeemed balance must be zero"
        );
        assert_eq!(
            HoprBalance::from(TICKET_VALUE * COUNT_TICKETS),
            stats.neglected_value(),
            "there must be a neglected value"
        );

        assert_eq!(
            stats,
            db.get_ticket_statistics(Some(*CHANNEL_ID)).await?,
            "per channel stats must be same"
        );

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_mark_unsaved_ticket_rejected() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;

        let mut ticket = init_db_with_tickets(&db, 1).await?;
        let ticket = ticket.pop().context("ticket should be present")?.ticket;

        let stats = db.get_ticket_statistics(None).await?;
        assert_eq!(HoprBalance::zero(), stats.rejected_value());
        assert_eq!(
            stats,
            db.get_ticket_statistics(Some(*CHANNEL_ID)).await?,
            "per channel stats must be same"
        );

        tracing::debug!("marking ticket as rejected");
        db.mark_unsaved_ticket_rejected(BOB.public().as_ref(), ticket.verified_ticket())
            .await?;

        let stats = db.get_ticket_statistics(None).await?;
        assert_eq!(ticket.verified_ticket().amount, stats.rejected_value());
        assert_eq!(
            stats,
            db.get_ticket_statistics(Some(*CHANNEL_ID)).await?,
            "per channel stats must be same"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_update_tickets_states_and_fetch() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;

        init_db_with_tickets(&db, 10).await?;

        let selector = TicketSelector::new(*CHANNEL_ID, CHANNEL_EPOCH).with_index(5);

        let v: Vec<RedeemableTicket> = db
            .update_ticket_states_and_fetch([selector], AcknowledgedTicketStatus::BeingRedeemed)
            .await?
            .collect()
            .await;

        assert_eq!(1, v.len(), "single ticket must be updated");

        let selector = TicketSelector::new(*CHANNEL_ID, CHANNEL_EPOCH).with_state(AcknowledgedTicketStatus::Untouched);

        let v: Vec<RedeemableTicket> = db
            .update_ticket_states_and_fetch([selector], AcknowledgedTicketStatus::BeingRedeemed)
            .await?
            .collect()
            .await;

        assert_eq!(9, v.len(), "only specific tickets must have state set");
        assert!(
            v.iter().all(|t| t.verified_ticket().index != 5),
            "only tickets with different state must update"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_update_tickets_states() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;

        init_db_with_tickets(&db, 10).await?;
        let selector = TicketSelector::new(*CHANNEL_ID, CHANNEL_EPOCH).with_state(AcknowledgedTicketStatus::Untouched);

        db.update_ticket_states([selector.clone()], AcknowledgedTicketStatus::BeingRedeemed)
            .await?;

        let v: Vec<RedeemableTicket> = db
            .update_ticket_states_and_fetch([selector], AcknowledgedTicketStatus::BeingRedeemed)
            .await?
            .collect()
            .await;

        assert!(v.is_empty(), "must not update if already updated");

        Ok(())
    }

    #[tokio::test]
    async fn test_outgoing_ticket_index_should_be_zero_if_not_yet_present() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;

        let hash = Hash::default();

        let idx = db.get_or_create_outgoing_ticket_index(&hash, 1).await?;
        assert_eq!(None, idx, "initial index must be None");

        let idx = db.get_or_create_outgoing_ticket_index(&hash, 1).await?;
        assert_eq!(Some(0), idx, "index must be zero");

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_outgoing_ticket_index_should_not_allow_old_epochs() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;

        let hash = Hash::default();

        db.get_or_create_outgoing_ticket_index(&hash, 1).await?;
        db.get_or_create_outgoing_ticket_index(&hash, 2).await?;

        assert!(db.get_or_create_outgoing_ticket_index(&hash, 1).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_outgoing_ticket_index_should_be_updated_if_already_present() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;

        let hash = Hash::default();

        db.get_or_create_outgoing_ticket_index(&hash, 1).await?;
        db.update_outgoing_ticket_index(&hash, 1, 2).await?;

        assert_eq!(Some(2), db.get_or_create_outgoing_ticket_index(&hash, 1).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_outgoing_ticket_index_should_not_be_updated_if_not_present() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;

        let hash = Hash::default();

        db.update_outgoing_ticket_index(&hash, 1, 2).await?;

        assert_eq!(None, db.get_or_create_outgoing_ticket_index(&hash, 1).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_outgoing_ticket_index_should_not_be_updated_if_lower() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;

        let hash = Hash::default();

        db.get_or_create_outgoing_ticket_index(&hash, 1).await?;
        db.update_outgoing_ticket_index(&hash, 1, 2).await?;

        assert_eq!(Some(2), db.get_or_create_outgoing_ticket_index(&hash, 1).await?);

        assert!(db.update_outgoing_ticket_index(&hash, 1, 1).await.is_err());

        assert_eq!(Some(2), db.get_or_create_outgoing_ticket_index(&hash, 1).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_outgoing_ticket_index_should_be_removed() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;

        let hash = Hash::default();

        let idx = db.get_or_create_outgoing_ticket_index(&hash, 1).await?;
        assert!(idx.is_none());

        let idx = db.get_or_create_outgoing_ticket_index(&hash, 1).await?;
        assert_eq!(Some(0), idx);

        db.remove_outgoing_ticket_index(&hash, 1).await?;

        let idx = db.get_or_create_outgoing_ticket_index(&hash, 1).await?;
        assert!(idx.is_none());

        Ok(())
    }

    #[test]
    fn test_ticket_stats_default_must_be_zero() -> anyhow::Result<()> {
        let stats = ChannelTicketStatistics::default();
        assert_eq!(stats.unredeemed_value, HoprBalance::zero());
        assert_eq!(stats.redeemed_value(), HoprBalance::zero());
        assert_eq!(stats.neglected_value(), HoprBalance::zero());
        assert_eq!(stats.rejected_value(), HoprBalance::zero());
        assert_eq!(stats.winning_tickets, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_stats_must_be_zero_for_non_existing_channel() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;

        let stats = db.get_ticket_statistics(Some(*CHANNEL_ID)).await?;

        assert_eq!(stats, ChannelTicketStatistics::default());

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_stats_must_be_zero_when_no_tickets() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;

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
        let db = HoprNodeDb::new_in_memory().await?;

        let channel_1 = generate_channel_id(BOB.public().as_ref(), ALICE.public().as_ref());
        let channel_2 = generate_channel_id(ALICE.public().as_ref(), BOB.public().as_ref());

        let t1 = generate_random_ack_ticket(&BOB, &ALICE, 1, 1, 1.0)?;
        let t2 = generate_random_ack_ticket(&ALICE, &BOB, 1, 1, 1.0)?;

        let value = t1.verified_ticket().amount;

        db.upsert_ticket(t1).await?;
        db.upsert_ticket(t2).await?;

        let stats_1 = db.get_ticket_statistics(Some(channel_1)).await?;

        let stats_2 = db.get_ticket_statistics(Some(channel_2)).await?;

        assert_eq!(value, stats_1.unredeemed_value);
        assert_eq!(value, stats_2.unredeemed_value);

        assert_eq!(HoprBalance::zero(), stats_1.neglected_value());
        assert_eq!(HoprBalance::zero(), stats_2.neglected_value());

        assert_eq!(stats_1, stats_2);

        db.mark_tickets_as([TicketSelector::new(channel_1, CHANNEL_EPOCH)], TicketMarker::Neglected)
            .await?;

        let stats_1 = db.get_ticket_statistics(Some(channel_1)).await?;

        let stats_2 = db.get_ticket_statistics(Some(channel_2)).await?;

        assert_eq!(HoprBalance::zero(), stats_1.unredeemed_value);
        assert_eq!(value, stats_1.neglected_value());

        assert_eq!(HoprBalance::zero(), stats_2.neglected_value());

        Ok(())
    }

    #[tokio::test]
    async fn test_set_ticket_statistics_when_tickets_are_in_db() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory().await?;

        let ticket = init_db_with_tickets(&db, 1).await?.pop().unwrap();

        db.mark_tickets_as([&ticket], TicketMarker::Redeemed)
            .await
            .expect("must not fail");

        let stats = db.get_ticket_statistics(None).await.expect("must not fail");
        assert_ne!(stats.redeemed_value(), HoprBalance::zero());

        db.reset_ticket_statistics().await.expect("must not fail");

        let stats = db.get_ticket_statistics(None).await.expect("must not fail");
        assert_eq!(stats.redeemed_value(), HoprBalance::zero());

        Ok(())
    }
}
