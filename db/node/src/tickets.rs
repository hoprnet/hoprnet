use std::{
    ops::Bound,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use async_stream::stream;
use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt, stream::BoxStream};
use hopr_api::db::*;
use hopr_crypto_types::prelude::*;
use hopr_db_entity::{outgoing_ticket_index, ticket, ticket_statistics};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use sea_query::{Condition, Expr, IntoCondition, Order, SimpleExpr};
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

pub(crate) async fn find_stats_for_channel(
    tx: &sea_orm::DatabaseTransaction,
    channel_id: &Hash,
) -> Result<ticket_statistics::Model, NodeDbError> {
    if let Some(model) = ticket_statistics::Entity::find()
        .filter(ticket_statistics::Column::ChannelId.eq(channel_id.to_hex()))
        .one(tx)
        .await?
    {
        Ok(model)
    } else {
        let new_stats = ticket_statistics::ActiveModel {
            channel_id: Set(channel_id.to_hex()),
            ..Default::default()
        }
        .insert(tx)
        .await?;

        Ok(new_stats)
    }
}

#[async_trait]
impl HoprDbTicketOperations for HoprNodeDb {
    type Error = NodeDbError;

    async fn stream_tickets<'c>(
        &'c self,
        selector: Option<TicketSelector>,
    ) -> Result<BoxStream<'c, AcknowledgedTicket>, Self::Error> {
        let qry = if let Some(selector) = selector.map(WrappedTicketSelector::from) {
            ticket::Entity::find().filter(selector)
        } else {
            ticket::Entity::find()
        };

        Ok(qry
            .order_by(ticket::Column::ChannelId, Order::Asc)
            .order_by(ticket::Column::ChannelEpoch, Order::Asc)
            .order_by(ticket::Column::Index, Order::Asc)
            .stream(&self.tickets_db)
            .await?
            .and_then(|model| {
                futures::future::ready(
                    AcknowledgedTicket::try_from(model).map_err(|e| sea_orm::DbErr::Custom(e.to_string())),
                )
            })
            .filter_map(|ticket| {
                futures::future::ready(ticket.inspect_err(|error| error!(%error, "invalid ticket in db")).ok())
            })
            .boxed())
    }

    async fn mark_tickets_as(&self, selector: TicketSelector, mark_as: TicketMarker) -> Result<usize, NodeDbError> {
        let myself = self.clone();
        Ok(self
            .ticket_manager
            .write_transaction(|tx| {
                Box::pin(async move {
                    let mut total_marked_count = 0;
                    for (channel_id, epoch) in selector.channel_identifiers.iter() {
                        let channel_selector = selector.clone().just_on_channel(*channel_id, epoch);

                        // Get the number of tickets and their value just for this channel
                        let (marked_count, marked_value) =
                            myself.get_tickets_value_int(tx, channel_selector.clone()).await?;
                        trace!(marked_count, ?marked_value, ?mark_as, "ticket marking");

                        if marked_count > 0 {
                            // Delete the redeemed tickets first
                            let deleted = ticket::Entity::delete_many()
                                .filter(WrappedTicketSelector::from(channel_selector.clone()))
                                .exec(tx)
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
                                        let unredeemed_value = myself
                                            .caches
                                            .unrealized_value
                                            .get(&(*channel_id, *epoch))
                                            .await
                                            .unwrap_or_default();

                                        METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                                            &["unredeemed"],
                                            (unredeemed_value - marked_value.amount()).amount().as_u128() as f64,
                                        );
                                    }
                                }

                                myself.caches.unrealized_value.invalidate(&(*channel_id, *epoch)).await;
                            } else {
                                return Err(NodeDbError::LogicalError(format!(
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

    async fn mark_unsaved_ticket_rejected(&self, ticket: &Ticket) -> Result<(), NodeDbError> {
        let channel_id = ticket.channel_id;
        let amount = ticket.amount;
        Ok(self
            .ticket_manager
            .write_transaction(|tx| {
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

    async fn update_ticket_states_and_fetch<'a>(
        &'a self,
        selector: TicketSelector,
        new_state: AcknowledgedTicketStatus,
    ) -> Result<BoxStream<'a, AcknowledgedTicket>, NodeDbError> {
        let selector: WrappedTicketSelector = selector.into();
        Ok(Box::pin(stream! {
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
                            let _g = self.ticket_manager.mutex.lock().await;
                            if let Err(error) = active_ticket.update(&self.tickets_db).await {
                                error!(%error, "failed to update ticket in the db");
                            }
                        }

                        match AcknowledgedTicket::try_from(ticket) {
                            Ok(mut ticket) => {
                                // Update the state manually, since we do not want to re-fetch the model after the update
                                ticket.status = new_state;
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

    async fn update_ticket_states(
        &self,
        selector: TicketSelector,
        new_state: AcknowledgedTicketStatus,
    ) -> Result<usize, NodeDbError> {
        let selector: WrappedTicketSelector = selector.into();
        Ok(self
            .ticket_manager
            .write_transaction(|tx| {
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

    async fn get_ticket_statistics(&self, channel_id: Option<Hash>) -> Result<ChannelTicketStatistics, NodeDbError> {
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
                                    acc.neglected_value += neglected_value;
                                    let redeemed_value = HoprBalance::from_be_bytes(stats.redeemed_value);
                                    acc.redeemed_value += redeemed_value;
                                    let rejected_value = HoprBalance::from_be_bytes(stats.rejected_value);
                                    acc.rejected_value += rejected_value;
                                    acc.winning_tickets += stats.winning_tickets as u128;
                                    acc
                                },
                            );

                            all_stats.unredeemed_value = unredeemed_value.into();

                            #[cfg(all(feature = "prometheus", not(test)))]
                            {
                                METRIC_HOPR_TICKETS_INCOMING_STATISTICS
                                    .set(&["neglected"], all_stats.neglected_value.amount().as_u128() as f64);
                                METRIC_HOPR_TICKETS_INCOMING_STATISTICS
                                    .set(&["redeemed"], all_stats.redeemed_value.amount().as_u128() as f64);
                                METRIC_HOPR_TICKETS_INCOMING_STATISTICS
                                    .set(&["rejected"], all_stats.rejected_value.amount().as_u128() as f64);
                                METRIC_HOPR_TICKETS_INCOMING_STATISTICS
                                    .set(&["winning_tickets"], all_stats.winning_tickets as f64);
                            }
                            Ok::<_, NodeDbError>(all_stats)
                        })
                    })
                    .await
            }
            Some(channel) => {
                self.tickets_db
                    .transaction(|tx| {
                        Box::pin(async move {
                            let stats = find_stats_for_channel(tx, &channel).await?;
                            let unredeemed_value = ticket::Entity::find()
                                .filter(ticket::Column::ChannelId.eq(channel.to_hex()))
                                .stream(tx)
                                .await?
                                .try_fold(U256::zero(), |amount, x| {
                                    futures::future::ok(amount + U256::from_be_bytes(x.amount))
                                })
                                .await?;

                            Ok::<_, NodeDbError>(ChannelTicketStatistics {
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
        debug!(stats = ?res, "retrieved ticket statistics");
        Ok(res?)
    }

    async fn reset_ticket_statistics(&self) -> Result<(), NodeDbError> {
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

    async fn get_tickets_value(&self, selector: TicketSelector) -> Result<(usize, HoprBalance), NodeDbError> {
        self.get_tickets_value_int(&self.tickets_db, selector).await
    }

    async fn compare_and_set_outgoing_ticket_index(&self, channel_id: Hash, index: u64) -> Result<u64, NodeDbError> {
        let old_value = self
            .get_outgoing_ticket_index(channel_id)
            .await?
            .fetch_max(index, Ordering::SeqCst);

        // TODO: should we hint the persisting mechanism to trigger the flush?
        // if old_value < index {}

        Ok(old_value)
    }

    async fn reset_outgoing_ticket_index(&self, channel_id: Hash) -> Result<u64, NodeDbError> {
        let old_value = self
            .get_outgoing_ticket_index(channel_id)
            .await?
            .swap(0, Ordering::SeqCst);

        // TODO: should we hint the persisting mechanism to trigger the flush?
        // if old_value > 0 { }

        Ok(old_value)
    }

    async fn increment_outgoing_ticket_index(&self, channel_id: Hash) -> Result<u64, NodeDbError> {
        let old_value = self
            .get_outgoing_ticket_index(channel_id)
            .await?
            .fetch_add(1, Ordering::SeqCst);

        // TODO: should we hint the persisting mechanism to trigger the flush?

        Ok(old_value)
    }

    async fn get_outgoing_ticket_index(&self, channel_id: Hash) -> Result<Arc<AtomicU64>, NodeDbError> {
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
                            .write_transaction(|tx| {
                                Box::pin(async move {
                                    outgoing_ticket_index::ActiveModel {
                                        channel_id: Set(channel_id.to_hex()),
                                        ..Default::default()
                                    }
                                    .insert(tx)
                                    .await?;
                                    Ok::<_, sea_orm::DbErr>(())
                                })
                            })
                            .await?;
                        0_u64
                    }
                })))
            })
            .await
            .map_err(|e: Arc<NodeDbError>| {
                NodeDbError::LogicalError(format!("failed to retrieve ticket index: {e}"))
            })?)
    }

    async fn persist_outgoing_ticket_indices(&self) -> Result<usize, NodeDbError> {
        let outgoing_indices = outgoing_ticket_index::Entity::find().all(&self.tickets_db).await?;

        let mut updated = 0;
        for index_model in outgoing_indices {
            let channel_id = Hash::from_hex(&index_model.channel_id).map_err(NodeDbError::from)?;
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
                        .write_transaction(|tx| {
                            Box::pin(async move {
                                index_active_model.save(tx).await?;
                                Ok::<_, sea_orm::DbErr>(())
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

        Ok(Ok::<_, NodeDbError>(updated)?)
    }
}

impl HoprNodeDb {
    /// Used only by non-SQLite code and tests.
    pub async fn upsert_ticket(&self, acknowledged_ticket: AcknowledgedTicket) -> Result<(), NodeDbError> {
        self.tickets_db
            .transaction(|tx| {
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

                    if let Some(ticket) = ticket::Entity::find().filter(selector).one(tx).await? {
                        model.id = Set(ticket.id);
                    }

                    model.save(tx).await
                })
            })
            .await?;
        Ok(())
    }

    async fn get_tickets_value_int(
        &self,
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
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use anyhow::Context;
    use futures::StreamExt;
    use hex_literal::hex;
    use hopr_api::db::{ChannelTicketStatistics, TicketMarker};
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

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
    ) -> anyhow::Result<AcknowledgedTicket> {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();
        let challenge = Response::from_half_keys(&hk1, &hk2)?.to_challenge()?;

        Ok(TicketBuilder::default()
            .addresses(src, dst)
            .amount(TICKET_VALUE)
            .index(index)
            .win_prob(win_prob.try_into()?)
            .channel_epoch(CHANNEL_EPOCH)
            .challenge(challenge)
            .build_signed(src, &Hash::default())?
            .into_acknowledged(Response::from_half_keys(&hk1, &hk2)?))
    }

    async fn init_db_with_tickets(db: &HoprNodeDb, count_tickets: u64) -> anyhow::Result<Vec<AcknowledgedTicket>> {
        let tickets: Vec<AcknowledgedTicket> = (0..count_tickets)
            .map(|i| generate_random_ack_ticket(&BOB, &ALICE, i, 1, 1.0))
            .collect::<anyhow::Result<Vec<AcknowledgedTicket>>>()?;

        for t in &tickets {
            db.upsert_ticket(t.clone()).await?;
        }

        Ok(tickets)
    }

    #[tokio::test]
    async fn test_insert_get_ticket() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory(ALICE.clone()).await?;

        let mut tickets = init_db_with_tickets(&db, 1).await?;
        let ack_ticket = tickets.pop().context("ticket should be present")?;

        assert_eq!(
            *CHANNEL_ID,
            ack_ticket.verified_ticket().channel_id,
            "channel ids must match"
        );
        assert_eq!(
            CHANNEL_EPOCH,
            ack_ticket.verified_ticket().channel_epoch,
            "epochs must match"
        );

        let db_ticket = db
            .stream_tickets(Some((&ack_ticket).into()))
            .await?
            .collect::<Vec<_>>()
            .await
            .first()
            .cloned()
            .context("ticket should exist")?;

        assert_eq!(ack_ticket, db_ticket, "tickets must be equal");

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_redeemed() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory(ALICE.clone()).await?;
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
            stats.redeemed_value,
            "there must be 0 redeemed value"
        );

        assert_eq!(
            stats,
            db.get_ticket_statistics(Some(*CHANNEL_ID)).await?,
            "per channel stats must be same"
        );

        const TO_REDEEM: u64 = 2;
        for ticket in tickets.iter().take(TO_REDEEM as usize) {
            let r = db.mark_tickets_as(ticket.into(), TicketMarker::Redeemed).await?;
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
        let db = HoprNodeDb::new_in_memory(ALICE.clone()).await?;

        let ticket = init_db_with_tickets(&db, 1)
            .await?
            .pop()
            .context("should contain a ticket")?;

        db.mark_tickets_as((&ticket).into(), TicketMarker::Redeemed).await?;
        assert_eq!(0, db.mark_tickets_as((&ticket).into(), TicketMarker::Redeemed).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_redeem_should_redeem_all_tickets() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory(ALICE.clone()).await?;

        let count_tickets = 10;
        init_db_with_tickets(&db, count_tickets).await?;

        let count_marked = db
            .mark_tickets_as(TicketSelector::new(*CHANNEL_ID, CHANNEL_EPOCH), TicketMarker::Redeemed)
            .await?;
        assert_eq!(count_tickets, count_marked as u64, "must mark all tickets in channel");

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_tickets_neglected() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory(ALICE.clone()).await?;
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
            stats.neglected_value,
            "there must be 0 redeemed value"
        );

        assert_eq!(
            stats,
            db.get_ticket_statistics(Some(*CHANNEL_ID)).await?,
            "per channel stats must be same"
        );

        db.mark_tickets_as(TicketSelector::new(*CHANNEL_ID, CHANNEL_EPOCH), TicketMarker::Neglected)
            .await?;

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
        let db = HoprNodeDb::new_in_memory(ALICE.clone()).await?;

        let mut ticket = init_db_with_tickets(&db, 1).await?;
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
    async fn test_update_tickets_states_and_fetch() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory(ALICE.clone()).await?;

        init_db_with_tickets(&db, 10).await?;

        let selector = TicketSelector::new(*CHANNEL_ID, CHANNEL_EPOCH).with_index(5);

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

        let selector = TicketSelector::new(*CHANNEL_ID, CHANNEL_EPOCH).with_state(AcknowledgedTicketStatus::Untouched);

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
        let db = HoprNodeDb::new_in_memory(ALICE.clone()).await?;

        init_db_with_tickets(&db, 10).await?;
        let selector = TicketSelector::new(*CHANNEL_ID, CHANNEL_EPOCH).with_state(AcknowledgedTicketStatus::Untouched);

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
        let db = HoprNodeDb::new_in_memory(ChainKeypair::random()).await?;

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

    #[test]
    fn test_ticket_stats_default_must_be_zero() -> anyhow::Result<()> {
        let stats = ChannelTicketStatistics::default();
        assert_eq!(stats.unredeemed_value, HoprBalance::zero());
        assert_eq!(stats.redeemed_value, HoprBalance::zero());
        assert_eq!(stats.neglected_value, HoprBalance::zero());
        assert_eq!(stats.rejected_value, HoprBalance::zero());
        assert_eq!(stats.winning_tickets, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_stats_must_be_zero_for_non_existing_channel() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory(ChainKeypair::random()).await?;

        let stats = db.get_ticket_statistics(Some(*CHANNEL_ID)).await?;

        assert_eq!(stats, ChannelTicketStatistics::default());

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_stats_must_be_zero_when_no_tickets() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory(ChainKeypair::random()).await?;

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
        let db = HoprNodeDb::new_in_memory(ChainKeypair::random()).await?;

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

        assert_eq!(HoprBalance::zero(), stats_1.neglected_value);
        assert_eq!(HoprBalance::zero(), stats_2.neglected_value);

        assert_eq!(stats_1, stats_2);

        db.mark_tickets_as(TicketSelector::new(channel_1, CHANNEL_EPOCH), TicketMarker::Neglected)
            .await?;

        let stats_1 = db.get_ticket_statistics(Some(channel_1)).await?;

        let stats_2 = db.get_ticket_statistics(Some(channel_2)).await?;

        assert_eq!(HoprBalance::zero(), stats_1.unredeemed_value);
        assert_eq!(value, stats_1.neglected_value);

        assert_eq!(HoprBalance::zero(), stats_2.neglected_value);

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_index_compare_and_set_and_increment() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory(ChainKeypair::random()).await?;

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
        let db = HoprNodeDb::new_in_memory(ChainKeypair::random()).await?;

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
        let db = HoprNodeDb::new_in_memory(ChainKeypair::random()).await?;

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
        let db = HoprNodeDb::new_in_memory(ChainKeypair::random()).await?;

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

    #[tokio::test]
    async fn test_set_ticket_statistics_when_tickets_are_in_db() -> anyhow::Result<()> {
        let db = HoprNodeDb::new_in_memory(ALICE.clone()).await?;

        let ticket = init_db_with_tickets(&db, 1).await?.pop().unwrap();

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
}
