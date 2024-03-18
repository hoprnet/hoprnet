use async_stream::stream;
use std::ops::Add;
use std::ops::Sub;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tracing::error;
use tracing::info;

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::TryStreamExt;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Set};
use sea_query::{Expr, SimpleExpr};
use tracing::{debug, instrument, trace};

use hopr_crypto_packet::{chain::ChainPacketComponents, validation::validate_unacknowledged_ticket};
use hopr_crypto_types::prelude::*;
use hopr_db_entity::conversions::tickets::model_to_acknowledged_ticket;
use hopr_db_entity::prelude::{Ticket, TicketStatistics};
use hopr_db_entity::ticket_statistics;
use hopr_db_entity::{outgoing_ticket_index, ticket};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::channels::HoprDbChannelOperations;
use crate::db::HoprDb;
use crate::errors::DbError::LogicalError;
use crate::errors::{DbError, Result};
use crate::info::HoprDbInfoOperations;
use crate::resolver::HoprDbResolverOperations;
use crate::{HoprDbGeneralModelOperations, OptTx, TargetDb, SINGULAR_TABLE_FIXED_ID};

#[allow(clippy::large_enum_variant)] // TODO: Uses too large objects
#[derive(Debug)]
pub enum AckResult {
    Sender(HalfKeyChallenge),
    RelayerWinning(AcknowledgedTicket),
    RelayerLosing,
}

pub enum TransportPacketWithChainData {
    /// Packet is intended for us
    Final {
        packet_tag: PacketTag,
        previous_hop: OffchainPublicKey,
        plain_text: Box<[u8]>,
        ack: Acknowledgement,
    },
    /// Packet must be forwarded
    Forwarded {
        packet_tag: PacketTag,
        previous_hop: OffchainPublicKey,
        next_hop: OffchainPublicKey,
        data: Box<[u8]>,
        ack: Acknowledgement,
    },
    /// Packet that is being sent out by us
    Outgoing {
        next_hop: OffchainPublicKey,
        ack_challenge: HalfKeyChallenge,
        data: Box<[u8]>,
    },
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
            channel_id: value.ticket.channel_id,
            epoch: value.ticket.channel_epoch.into(),
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

impl From<TicketSelector> for SimpleExpr {
    fn from(value: TicketSelector) -> Self {
        let mut expr = ticket::Column::ChannelId
            .eq(value.channel_id.to_hex())
            .and(ticket::Column::ChannelEpoch.eq(value.epoch.to_be_bytes().to_vec()));

        if let Some(index) = value.index {
            expr = expr.and(ticket::Column::Index.eq(index.to_be_bytes().to_vec()));
        }

        if let Some(state) = value.state {
            expr = expr.and(ticket::Column::State.eq(state as u8))
        }

        if value.only_aggregated {
            expr = expr.and(ticket::Column::IndexOffset.gt(1));
        }

        expr
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AggregationPrerequisites {
    pub min_ticket_count: usize,
    pub min_unaggregated_ratio: f64,
}

#[async_trait]
pub trait HoprDbTicketOperations {
    /// Retrieve acknowledged winning tickets according to the given `selector`.
    /// The optional transaction `tx` must be in the [tickets database](TargetDb::Tickets).
    async fn get_tickets<'a>(&'a self, tx: OptTx<'a>, selector: TicketSelector) -> Result<Vec<AcknowledgedTicket>>;

    /// Marks tickets as redeemed (removing them from the DB) and updating the statistics.
    /// Returns the number of tickets that were redeemed.
    async fn mark_tickets_redeemed(&self, selector: TicketSelector) -> Result<usize>;

    /// Marks tickets as redeemed (removing them from the DB) and updating the statistics.
    /// Returns the number of tickets that were neglected.
    async fn mark_tickets_neglected(&self, selector: TicketSelector) -> Result<usize>;

    /// Updates [state](AcknowledgedTicketStatus) of the tickets matching the given `selector`.
    /// Returns the updated tickets in the new state.
    async fn update_ticket_states_and_fetch<'a>(
        &'a self,
        selector: TicketSelector,
        new_state: AcknowledgedTicketStatus,
    ) -> Result<BoxStream<'a, AcknowledgedTicket>>;

    /// Updates [state](AcknowledgedTicketStatus) of the tickets matching the given `selector`.
    async fn update_ticket_states(&self, selector: TicketSelector, new_state: AcknowledgedTicketStatus) -> Result<()>;

    /// Retrieves the ticket statistics.
    ///
    /// The optional transaction `tx` must be in the [tickets database](TargetDb::Tickets).
    async fn get_ticket_statistics<'a>(&'a self, tx: OptTx<'a>) -> Result<AllTicketStatistics>;

    /// Counts the tickets matching the given `selector` and their total value.
    ///
    /// The optional transaction `tx` must be in the [tickets database](TargetDb::Tickets).
    async fn get_tickets_value<'a>(&'a self, tx: OptTx<'a>, selector: TicketSelector) -> Result<(usize, Balance)>;

    /// Sets the stored outgoing ticket index to `index`, only if the currently stored value
    /// is less than `index`. This ensures the stored value can only be growing.
    /// Returns the old value.
    ///
    /// If the entry is not yet present for the given ID, it is initialized to 0.
    async fn compare_and_set_ticket_index(&self, channel_id: Hash, index: u64) -> Result<u64>;

    /// Resets the outgoing ticket index to 0 for the given channel id.
    /// Returns the old value before reset.
    ///
    /// If the entry is not yet present for the given ID, it is initialized to 0.
    async fn reset_ticket_index(&self, channel_id: Hash) -> Result<u64>;

    /// Increments the outgoing ticket index in the given channel ID and returns the value before incrementing.
    ///
    /// If the entry is not yet present for the given ID, it is initialized to 0 and incremented.
    async fn increment_ticket_index(&self, channel_id: Hash) -> Result<u64>;

    /// Gets the current outgoing ticket index for the given channel id.
    ///
    /// If the entry is not yet present for the given ID, it is initialized to 0.
    async fn get_ticket_index(&self, channel_id: Hash) -> Result<Arc<AtomicU64>>;

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
        prerequisites: Option<AggregationPrerequisites>,
    ) -> Result<Option<(OffchainPublicKey, Vec<AcknowledgedTicket>)>>;

    /// Perform a ticket aggregation rollback in the channel.
    ///
    /// If a ticket aggregation fails, this callback can be invoked to make sure that
    /// resources are properly restored and cleaned up in the database, allowing further
    /// aggregations.
    async fn rollback_aggregation_in_channel(&self, channel: Hash) -> Result<()>;

    /// Replace the aggregated tickets locally with an aggregated ticket from the counterparty.
    async fn process_received_aggregated_ticket(
        &self,
        aggregated_ticket: hopr_internal_types::channels::Ticket,
        chain_keypair: &ChainKeypair,
    ) -> Result<AcknowledgedTicket>;

    async fn aggregate_tickets(
        &self,
        destination: OffchainPublicKey,
        acked_tickets: Vec<AcknowledgedTicket>,
        me: &ChainKeypair,
    ) -> Result<hopr_internal_types::channels::Ticket>;

    /// Processes the acknowledgements for the pending tickets
    ///
    /// There are three cases:
    /// 1. There is an unacknowledged ticket and we are awaiting a half key.
    /// 2. We were the creator of the packet, hence we do not wait for any half key
    /// 3. The acknowledgement is unexpected and stems from a protocol bug or an attacker
    async fn handle_acknowledgement(&self, ack: Acknowledgement, me: ChainKeypair) -> Result<AckResult>;

    /// Process the data into an outgoing packet
    async fn to_send(
        &self,
        data: Box<[u8]>,
        me: ChainKeypair,
        path: Vec<OffchainPublicKey>,
    ) -> Result<TransportPacketWithChainData>;

    /// Process the incoming packet into data
    #[allow(clippy::wrong_self_convention)]
    async fn from_recv(
        &self,
        data: Box<[u8]>,
        me: ChainKeypair,
        pkt_keypair: &OffchainKeypair,
        sender: OffchainPublicKey,
    ) -> Result<TransportPacketWithChainData>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AllTicketStatistics {
    pub last_updated: SystemTime,
    pub losing_tickets: u64,
    pub neglected_tickets: u64,
    pub neglected_value: Balance,
    pub redeemed_tickets: u64,
    pub redeemed_value: Balance,
    pub unredeemed_tickets: u64,
    pub unredeemed_value: Balance,
    pub rejected_tickets: u64,
    pub rejected_value: Balance,
}

#[async_trait]
impl HoprDbTicketOperations for HoprDb {
    async fn get_tickets<'a>(&'a self, tx: OptTx<'a>, selector: TicketSelector) -> Result<Vec<AcknowledgedTicket>> {
        let channel_dst = self
            .get_indexer_data(tx)
            .await?
            .channels_dst
            .ok_or(LogicalError("missing channel dst".into()))?;

        let ckp = self.chain_key.clone();
        self.nest_transaction_in_db(tx, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    ticket::Entity::find()
                        .filter(SimpleExpr::from(selector))
                        .all(tx.as_ref())
                        .await?
                        .into_iter()
                        .map(|m| model_to_acknowledged_ticket(&m, channel_dst, &ckp).map_err(DbError::from))
                        .collect::<Result<Vec<_>>>()
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
                        let deleted = ticket::Entity::delete_many()
                            .filter(SimpleExpr::from(selector))
                            .exec(tx.as_ref())
                            .await?;

                        // Update the stats if successful
                        if deleted.rows_affected == redeemed_count as u64 {
                            let stats = ticket_statistics::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                                .one(tx.as_ref())
                                .await?
                                .ok_or(DbError::MissingFixedTableEntry("ticket_statistics".into()))?;

                            let current_redeemed_value = U256::from_be_bytes(stats.redeemed_value.clone());
                            let current_redeemed_count = stats.redeemed_tickets;

                            let mut active_stats = stats.into_active_model();
                            active_stats.redeemed_tickets = Set(current_redeemed_count + redeemed_count as i32);
                            active_stats.redeemed_value =
                                Set((current_redeemed_value + redeemed_value.amount()).to_be_bytes().into());
                            active_stats.save(tx.as_ref()).await?;
                        } else {
                            return Err(DbError::LogicalError(format!(
                                "could not mark {redeemed_count} ticket as redeemed"
                            )));
                        }
                    }

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
                        let deleted = ticket::Entity::delete_many()
                            .filter(SimpleExpr::from(selector))
                            .exec(tx.as_ref())
                            .await?;

                        // Update the stats if successful
                        if deleted.rows_affected == neglectable_count as u64 {
                            let stats = ticket_statistics::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                                .one(tx.as_ref())
                                .await?
                                .ok_or(DbError::MissingFixedTableEntry("ticket_statistics".into()))?;

                            let current_neglected_value = U256::from_be_bytes(stats.neglected_value.clone());
                            let current_neglected_count = stats.neglected_tickets;

                            let mut active_stats = stats.into_active_model();
                            active_stats.neglected_tickets = Set(current_neglected_count + neglectable_count as i32);
                            active_stats.neglected_value = Set((current_neglected_value + neglectable_value.amount())
                                .to_be_bytes()
                                .into());
                            active_stats.save(tx.as_ref()).await?;
                        } else {
                            return Err(DbError::LogicalError(format!(
                                "could not mark {neglectable_count} ticket as neglected"
                            )));
                        }
                    }

                    Ok(neglectable_count)
                })
            })
            .await
    }

    async fn aggregate_tickets(
        &self,
        destination: OffchainPublicKey,
        mut acked_tickets: Vec<AcknowledgedTicket>,
        me: &ChainKeypair,
    ) -> Result<hopr_internal_types::channels::Ticket> {
        if me.public().to_address() != self.me_onchain {
            return Err(DbError::LogicalError(
                "chain key for ticket aggregation does not match the DB public address".into(),
            ));
        }

        if acked_tickets.is_empty() {
            return Err(DbError::LogicalError(
                "at least one ticket required for aggregation".to_owned(),
            ));
        }

        if acked_tickets.len() == 1 {
            return Ok(acked_tickets[0].ticket.clone());
        }

        acked_tickets.sort();
        acked_tickets.dedup();

        let myself = self.clone();

        let (channel_entry, destination, domain_separator) =
            self.nest_transaction_in_db(None, TargetDb::Index)
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

                        let channel_id = generate_channel_id(&myself.me_onchain, &address);
                        let entry = myself
                            .get_channel_by_id(Some(tx), channel_id)
                            .await?
                            .ok_or(DbError::ChannelNotFound(channel_id))?;

                        if entry.status == ChannelStatus::Closed {
                            return Err(DbError::LogicalError(format!("channel '{channel_id}' is closed")));
                        } else if entry.direction(&myself.me_onchain) != Some(ChannelDirection::Outgoing) {
                            return Err(DbError::LogicalError(format!("channel '{channel_id}' is not outgoing")));
                        }
                        let domain_separator =
                            myself.get_indexer_data(Some(tx)).await?.channels_dst.ok_or_else(|| {
                                crate::errors::DbError::LogicalError("domain separator missing".into())
                            })?;

                        Ok((entry, address, domain_separator))
                    })
                })
                .await?;

        let channel_balance = channel_entry.balance;
        let channel_epoch = channel_entry.channel_epoch;
        let channel_id = channel_entry.get_id();

        let mut final_value = Balance::zero(BalanceType::HOPR);

        for (i, acked_ticket) in acked_tickets.iter().enumerate() {
            if channel_id != acked_ticket.ticket.channel_id {
                return Err(DbError::LogicalError(format!(
                    "aggregated ticket has an invalid channel id {}",
                    acked_ticket.ticket.channel_id
                )));
            }

            if U256::from(acked_ticket.ticket.channel_epoch) != channel_epoch {
                return Err(DbError::LogicalError("Channel epochs do not match".to_owned()));
            }

            if i + 1 < acked_tickets.len()
                && acked_ticket.ticket.index + acked_ticket.ticket.index_offset as u64
                    > acked_tickets[i + 1].ticket.index
            {
                return Err(DbError::LogicalError(
                    "Tickets with overlapping index intervals".to_owned(),
                ));
            }

            if acked_ticket
                .verify(&(me).into(), &destination, &domain_separator)
                .is_err()
            {
                return Err(DbError::LogicalError("Not a valid ticket".to_owned()));
            }

            if !acked_ticket.is_winning_ticket(&domain_separator) {
                return Err(DbError::LogicalError("Not a winning ticket".to_owned()));
            }

            final_value = final_value.add(&acked_ticket.ticket.amount);
            if final_value.gt(&channel_balance) {
                return Err(DbError::LogicalError(format!("ticket amount to aggregate {final_value} is greater than the balance {channel_balance} of channel {channel_id}")));
            }
        }

        info!(
            "aggregated {} tickets in channel {channel_id} with total value {final_value}",
            acked_tickets.len()
        );

        let first_acked_ticket = acked_tickets.first().unwrap();
        let last_acked_ticket = acked_tickets.last().unwrap();

        // calculate the minimum current ticket index as the larger value from the acked ticket index and on-chain ticket_index from channel_entry
        let current_ticket_index_from_acked_tickets = last_acked_ticket.ticket.index + 1;
        self.compare_and_set_ticket_index(channel_id, current_ticket_index_from_acked_tickets)
            .await?;

        hopr_internal_types::channels::Ticket::new(
            &destination,
            &final_value,
            first_acked_ticket.ticket.index.into(),
            (last_acked_ticket.ticket.index - first_acked_ticket.ticket.index + 1).into(),
            1.0, // Aggregated tickets have always 100% winning probability
            channel_epoch,
            first_acked_ticket.ticket.challenge.clone(),
            me,
            &domain_separator,
        )
        .map_err(|e| e.into())
    }

    async fn process_received_aggregated_ticket(
        &self,
        aggregated_ticket: hopr_internal_types::channels::Ticket,
        chain_keypair: &ChainKeypair,
    ) -> Result<AcknowledgedTicket> {
        if chain_keypair.public().to_address() != self.me_onchain {
            return Err(DbError::LogicalError(
                "chain key for ticket aggregation does not match the DB public address".into(),
            ));
        }

        if aggregated_ticket.win_prob() != 1.0f64 {
            return Err(DbError::LogicalError(
                "Aggregated tickets must have 100% win probability".into(),
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
                            .get_channel_by_id(Some(tx), channel_id)
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

        let acknowledged_tickets = self
            .nest_transaction_in_db(None, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    ticket::Entity::find()
                        .filter(hopr_db_entity::ticket::Column::ChannelId.eq(channel_entry.get_id().to_hex()))
                        .filter(
                            hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8),
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
        if aggregated_ticket.amount.lt(&stored_value) {
            error!("Aggregated ticket value in '{channel_id}' is lower than sum of stored tickets",);
            return Err(DbError::LogicalError(
                "Value of received aggregated ticket is too low".into(),
            ));
        }

        let acknowledged_tickets = acknowledged_tickets
            .into_iter()
            .map(|m| model_to_acknowledged_ticket(&m, domain_separator, &self.chain_key).map_err(DbError::from))
            .collect::<Result<Vec<AcknowledgedTicket>>>()?;

        // can be done, because the tickets collection is tested for emptiness before
        let first_stored_ticket = acknowledged_tickets.first().unwrap();

        // calculate the new current ticket index
        #[allow(unused_variables)]
        let current_ticket_index_from_aggregated_ticket =
            U256::from(aggregated_ticket.index).add(aggregated_ticket.index_offset);

        let acked_aggregated_ticket = AcknowledgedTicket::new(
            aggregated_ticket,
            first_stored_ticket.response.clone(),
            first_stored_ticket.signer,
            chain_keypair,
            &domain_separator,
        )?;

        if acked_aggregated_ticket
            .verify(&first_stored_ticket.signer, &(chain_keypair).into(), &domain_separator)
            .is_err()
        {
            debug!("Aggregated ticket in '{channel_id}' is invalid. Dropping ticket.",);
            return Err(DbError::LogicalError("Aggregated ticket is invalid".into()));
        }

        let ticket = acked_aggregated_ticket.clone();
        self.ticket_manager
            .with_write_locked_db(|tx| {
                Box::pin(async move {
                    let deleted = ticket::Entity::delete_many()
                        .filter(hopr_db_entity::ticket::Column::ChannelId.eq(channel_entry.get_id().to_hex()))
                        .filter(
                            hopr_db_entity::ticket::Column::ChannelEpoch
                                .eq(channel_entry.channel_epoch.to_be_bytes().to_vec()),
                        )
                        .filter(
                            hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8),
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

                    ticket::Entity::insert::<hopr_db_entity::ticket::ActiveModel>(ticket.into())
                        .exec(tx.as_ref())
                        .await?;

                    // TODO: is this necessary for an incoming channel?
                    // self.db
                    //     .write()
                    //     .await
                    //     .ensure_current_ticket_index_gte(&channel_id, current_ticket_index_from_aggregated_ticket)
                    //     .await?;

                    Ok::<(), DbError>(())
                })
            })
            .await?;

        Ok(acked_aggregated_ticket)
    }

    async fn prepare_aggregation_in_channel(
        &self,
        channel: &Hash,
        prerequisites: Option<AggregationPrerequisites>,
    ) -> Result<Option<(OffchainPublicKey, Vec<AcknowledgedTicket>)>> {
        let myself = self.clone();
        let chain_keypair = self.chain_key.clone();

        let channel = *channel;

        let (channel_entry, peer, ds) =
            self.nest_transaction_in_db(None, TargetDb::Index)
                .await?
                .perform(|tx| {
                    Box::pin(async move {
                        let entry = myself
                            .get_channel_by_id(Some(tx), channel)
                            .await?
                            .ok_or(DbError::ChannelNotFound(channel))?;

                        if entry.status == ChannelStatus::Closed {
                            return Err(DbError::LogicalError(format!("channel '{channel}' is closed")));
                        } else if entry.direction(&myself.me_onchain) != Some(ChannelDirection::Incoming) {
                            return Err(DbError::LogicalError(format!("channel '{channel}' is not incoming")));
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

        let tickets = self
            .ticket_manager
            .with_write_locked_db(|tx| {
                Box::pin(async move {
                    let channel_id = channel_entry.get_id().to_hex();

                    // verify that no aggregation is in progress in the channel
                    if ticket::Entity::find()
                        .filter(ticket::Column::ChannelId.eq(channel_id.clone()))
                        .filter(ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
                        .one(tx.as_ref())
                        .await?
                        .is_some()
                    {
                        return Err(DbError::LogicalError(format!(
                            "channel '{}' is already being aggregated",
                            channel_entry.get_id()
                        )));
                    }

                    // find the index of the last ticket being redeemed
                    let first_idx_to_take = ticket::Entity::find()
                        .filter(ticket::Column::ChannelId.eq(channel_id.clone()))
                        .filter(ticket::Column::ChannelEpoch.eq(channel_entry.channel_epoch.to_be_bytes().to_vec()))
                        .filter(ticket::Column::State.eq(AcknowledgedTicketStatus::BeingRedeemed as u8))
                        .order_by_desc(ticket::Column::Index)
                        .one(tx.as_ref())
                        .await?
                        .map(|m| U256::from_be_bytes(m.index).as_u64() + 1)
                        .unwrap_or(0_u64); // go from the lowest possible index of none is found

                    // get the list of all tickets to be aggregated
                    let to_be_aggregated = ticket::Entity::find()
                        .filter(ticket::Column::ChannelId.eq(channel_id.clone()))
                        .filter(ticket::Column::ChannelEpoch.eq(channel_entry.channel_epoch.to_be_bytes().to_vec()))
                        .filter(ticket::Column::Index.gte(first_idx_to_take.to_be_bytes().to_vec()))
                        .filter(ticket::Column::State.ne(AcknowledgedTicketStatus::BeingAggregated as u8))
                        .order_by_asc(ticket::Column::Index)
                        .all(tx.as_ref())
                        .await?;

                    // do a balance check to be sure not to aggregate more than current channel stake
                    let mut total_balance = Balance::zero(BalanceType::HOPR);
                    let mut aggregated_balance = Balance::zero(BalanceType::HOPR);
                    let mut last_idx_to_take = 0_u64;

                    for m in &to_be_aggregated {
                        let to_add = BalanceType::HOPR.balance_bytes(&m.amount);
                        if m.index_offset > 1 {
                            aggregated_balance = aggregated_balance.add(to_add);
                        }

                        total_balance = total_balance.add(to_add);
                        if total_balance.gt(&channel_entry.balance) {
                            break;
                        } else {
                            last_idx_to_take = U256::from_be_bytes(&m.index).as_u64();
                        }
                    }

                    let to_be_aggregated = to_be_aggregated
                        .into_iter()
                        .take_while(|m| U256::from_be_bytes(&m.index).as_u64() <= last_idx_to_take)
                        .map(|m| model_to_acknowledged_ticket(&m, ds, &chain_keypair).map_err(DbError::from))
                        .collect::<Result<Vec<AcknowledgedTicket>>>()?;

                    if let Some(prerequisites) = prerequisites {
                        if to_be_aggregated.len() < prerequisites.min_ticket_count
                            && total_balance
                                .sub(aggregated_balance)
                                .gt(&channel_entry.balance.mul_f64(prerequisites.min_unaggregated_ratio)?)
                        {
                            // aborting, because the constraints have not been met
                            return Ok(vec![]);
                        }
                    };

                    // mark all tickets with appropriate characteristics as being aggregated
                    let to_be_aggregated_count = to_be_aggregated.len();
                    if to_be_aggregated_count > 0 {
                        let marked: sea_orm::UpdateResult = ticket::Entity::update_many()
                            .filter(ticket::Column::ChannelId.eq(channel_id))
                            .filter(ticket::Column::ChannelEpoch.eq(channel_entry.channel_epoch.to_be_bytes().to_vec()))
                            .filter(ticket::Column::Index.gte(first_idx_to_take.to_be_bytes().to_vec()))
                            .filter(ticket::Column::Index.lte(last_idx_to_take.to_be_bytes().to_vec()))
                            .filter(ticket::Column::State.ne(AcknowledgedTicketStatus::BeingAggregated as u8))
                            .col_expr(
                                ticket::Column::State,
                                Expr::value(AcknowledgedTicketStatus::BeingAggregated as u8 as i32),
                            )
                            .exec(tx.as_ref())
                            .await?;

                        if marked.rows_affected as usize != to_be_aggregated_count {
                            return Err(DbError::LogicalError(format!(
                                "expected to mark {to_be_aggregated_count}, but was able to mark {}",
                                marked.rows_affected
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
        self.ticket_manager
            .with_write_locked_db(|tx| {
                Box::pin(async move {
                    // mark all being aggregated tickets as untouched
                    let rolled_back: sea_orm::UpdateResult = ticket::Entity::update_many()
                        .filter(ticket::Column::ChannelId.eq(channel.to_hex()))
                        .filter(ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8))
                        .col_expr(ticket::Column::State, Expr::value(AcknowledgedTicketStatus::Untouched as u8 as i32))
                        .exec(tx.as_ref())
                        .await?;

                    debug!(
                        "rollback happened for ticket aggregation in '{channel}' with {} tickets rolled back as a result",
                        rolled_back.rows_affected
                    );

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
        let channel_dst = self
            .get_indexer_data(None)
            .await?
            .channels_dst
            .ok_or(LogicalError("missing channel dst".into()))?;

        Ok(Box::pin(stream! {
            match ticket::Entity::find()
                .filter(SimpleExpr::from(selector))
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

                        match model_to_acknowledged_ticket(&ticket, channel_dst, &self.chain_key) {
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

    async fn update_ticket_states(&self, selector: TicketSelector, new_state: AcknowledgedTicketStatus) -> Result<()> {
        self.ticket_manager
            .with_write_locked_db(|tx| {
                Box::pin(async move {
                    ticket::Entity::update_many()
                        .filter(SimpleExpr::from(selector))
                        .col_expr(ticket::Column::State, Expr::value(new_state as u8))
                        .exec(tx.as_ref())
                        .await?;
                    Ok::<_, DbError>(())
                })
            })
            .await
    }

    async fn get_ticket_statistics<'a>(&'a self, tx: OptTx<'a>) -> Result<AllTicketStatistics> {
        self.nest_transaction_in_db(tx, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let stats = TicketStatistics::find_by_id(SINGULAR_TABLE_FIXED_ID)
                        .one(tx.as_ref())
                        .await?
                        .ok_or(DbError::MissingFixedTableEntry("ticket_statistics".into()))?;

                    let (unredeemed_tickets, unredeemed_value) = Ticket::find()
                        .stream(tx.as_ref())
                        .await?
                        .try_fold((0_u64, U256::zero()), |(count, amount), x| async move {
                            Ok((count + 1, amount + U256::from_be_bytes(x.amount)))
                        })
                        .await?;

                    Ok::<AllTicketStatistics, DbError>(AllTicketStatistics {
                        last_updated: chrono::DateTime::<chrono::Utc>::from_str(&stats.last_updated)
                            .map_err(|_| DbError::DecodingError)?
                            .into(),
                        losing_tickets: stats.losing_tickets as u64,
                        neglected_tickets: stats.neglected_tickets as u64,
                        neglected_value: BalanceType::HOPR.balance_bytes(stats.neglected_value),
                        redeemed_tickets: stats.redeemed_tickets as u64,
                        redeemed_value: BalanceType::HOPR.balance_bytes(stats.redeemed_value),
                        unredeemed_tickets,
                        unredeemed_value: BalanceType::HOPR.balance(unredeemed_value),
                        rejected_tickets: stats.rejected_tickets as u64,
                        rejected_value: BalanceType::HOPR.balance_bytes(stats.rejected_value),
                    })
                })
            })
            .await
    }

    async fn get_tickets_value<'a>(&'a self, tx: OptTx<'a>, selector: TicketSelector) -> Result<(usize, Balance)> {
        Ok(self
            .nest_transaction_in_db(tx, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    ticket::Entity::find()
                        .filter(SimpleExpr::from(selector))
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

    async fn compare_and_set_ticket_index(&self, channel_id: Hash, index: u64) -> Result<u64> {
        let old_value = self
            .get_ticket_index(channel_id)
            .await?
            .fetch_max(index, Ordering::SeqCst);

        // TODO: should we hint the persisting mechanism to trigger the flush?
        // if old_value < index {}

        Ok(old_value)
    }

    async fn increment_ticket_index(&self, channel_id: Hash) -> Result<u64> {
        let old_value = self.get_ticket_index(channel_id).await?.fetch_add(1, Ordering::SeqCst);

        // TODO: should we hint the persisting mechanism to trigger the flush?

        Ok(old_value)
    }

    async fn reset_ticket_index(&self, channel_id: Hash) -> Result<u64> {
        let old_value = self.get_ticket_index(channel_id).await?.swap(0, Ordering::SeqCst);

        // TODO: should we hint the persisting mechanism to trigger the flush?
        // if old_value > 0 { }

        Ok(old_value)
    }

    async fn persist_outgoing_ticket_indices(&self) -> Result<usize> {
        let outgoing_indices = outgoing_ticket_index::Entity::find().all(&self.tickets_db).await?;

        let mut updated = 0;
        for index_model in outgoing_indices {
            let channel_id = Hash::from_hex(&index_model.channel_id)?;
            let db_index = U256::from_be_bytes(&index_model.index).as_u64();
            if let Some(cached_index) = self.ticket_index.get(&channel_id).await {
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
                // channel, so the value  has not been yet fetched.
                debug!("channel {channel_id} is in the DB but not yet in the cache.");
            }
        }

        Ok::<_, DbError>(updated)
    }

    async fn get_ticket_index(&self, channel_id: Hash) -> Result<Arc<AtomicU64>> {
        let tkt_manager = self.ticket_manager.clone();

        self.ticket_index
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

    #[instrument(level = "trace", skip(self))]
    async fn handle_acknowledgement(&self, ack: Acknowledgement, me: ChainKeypair) -> Result<AckResult> {
        let myself = self.clone();
        let me_onchain = me.public().to_address();

        let result = self
            .begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    match myself
                        .unacked_tickets
                        .remove(&ack.ack_challenge())
                        .await
                        .ok_or_else(|| {
                            crate::errors::DbError::AcknowledgementValidationError(format!(
                                "received unexpected acknowledgement for half key challenge {} - half key {}",
                                ack.ack_challenge().to_hex(),
                                ack.ack_key_share.to_hex()
                            ))
                        })? {
                        PendingAcknowledgement::WaitingAsSender => {
                            trace!("received acknowledgement as sender: first relayer has processed the packet.");

                            Ok(AckResult::Sender(ack.ack_challenge()))
                        }

                        PendingAcknowledgement::WaitingAsRelayer(unacknowledged) => {
                            // Try to unlock the incentive
                            unacknowledged.verify_challenge(&ack.ack_key_share).map_err(|e| {
                                crate::errors::DbError::AcknowledgementValidationError(format!(
                                    "the acknowledgement is not sufficient to solve the embedded challenge, {e}"
                                ))
                            })?;

                            if myself
                                .get_channel_by_id(Some(tx), generate_channel_id(&unacknowledged.signer, &me_onchain))
                                .await?
                                .is_some_and(|c| c.channel_epoch.as_u32() != unacknowledged.ticket.channel_epoch)
                            {
                                return Err(crate::errors::DbError::LogicalError(format!(
                                    "no channel found for  address '{}'",
                                    unacknowledged.signer
                                )));
                            }

                            let domain_separator =
                                myself.get_indexer_data(Some(tx)).await?.channels_dst.ok_or_else(|| {
                                    crate::errors::DbError::LogicalError("domain separator missing".into())
                                })?;

                            let ack_ticket = unacknowledged.acknowledge(&ack.ack_key_share, &me, &domain_separator)?;

                            if ack_ticket.is_winning_ticket(&domain_separator) {
                                debug!(ticket = tracing::field::display(&ack_ticket), "winning ticket");
                                Ok(AckResult::RelayerWinning(ack_ticket))
                            } else {
                                trace!(ticket = tracing::field::display(&ack_ticket), "losing ticket");
                                Ok(AckResult::RelayerLosing)
                            }
                        }
                    }
                })
            })
            .await?;

        if let AckResult::RelayerWinning(ack_ticket) = &result {
            self.ticket_manager.insert_ticket(ack_ticket.clone())?;
        }

        Ok(result)
    }

    #[instrument(level = "trace", skip(self))]
    async fn to_send(
        &self,
        data: Box<[u8]>,
        me: ChainKeypair,
        path: Vec<OffchainPublicKey>,
    ) -> Result<TransportPacketWithChainData> {
        let myself = self.clone();

        let components = self
            .begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let next_peer = myself.resolve_chain_key(&path[0]).await?.ok_or_else(|| {
                        crate::errors::DbError::LogicalError(format!(
                            "failed to find channel key for packet key {} on previous hop",
                            path[0].to_peerid_str()
                        ))
                    })?;

                    let domain_separator = myself.get_indexer_data(Some(tx)).await?.channels_dst.ok_or_else(|| {
                        crate::errors::DbError::LogicalError("failed to fetch the domain separator".into())
                    })?;

                    // Decide whether to create 0-hop or multihop ticket
                    let next_ticket = if path.len() == 1 {
                        hopr_internal_types::channels::Ticket::new_zero_hop(&next_peer, &me, &domain_separator).map_err(
                            |e| {
                                crate::errors::DbError::LogicalError(format!("failed to construct a 0 hop ticket: {e}"))
                            },
                        )
                    } else {
                        myself
                            .create_multihop_ticket(Some(tx), me.public().to_address(), next_peer, path.len() as u8)
                            .await
                    }?;

                    ChainPacketComponents::into_outgoing(&data, &path, &me, next_ticket, &domain_separator).map_err(
                        |e| {
                            crate::errors::DbError::LogicalError(format!(
                                "failed to construct chain components for a packet: {e}"
                            ))
                        },
                    )
                })
            })
            .await?;

        match components {
            ChainPacketComponents::Final { .. } | ChainPacketComponents::Forwarded { .. } => Err(
                crate::errors::DbError::LogicalError("Must contain an outgoing packet type".into()),
            ),
            ChainPacketComponents::Outgoing {
                packet,
                ticket,
                next_hop,
                ack_challenge,
            } => {
                self.unacked_tickets
                    .insert(ack_challenge, PendingAcknowledgement::WaitingAsSender)
                    .await;

                let mut payload = Vec::with_capacity(ChainPacketComponents::SIZE);
                payload.extend_from_slice(packet.as_ref());
                payload.extend_from_slice(&ticket.to_bytes());

                Ok(TransportPacketWithChainData::Outgoing {
                    next_hop,
                    ack_challenge,
                    data: payload.into_boxed_slice(),
                })
            }
        }
    }

    #[instrument(level = "trace", skip(self))]
    async fn from_recv(
        &self,
        data: Box<[u8]>,
        me: ChainKeypair,
        pkt_keypair: &OffchainKeypair,
        sender: OffchainPublicKey,
    ) -> Result<TransportPacketWithChainData> {
        let me_onchain = me.public().to_address();
        match ChainPacketComponents::from_incoming(&data, pkt_keypair, sender)
            .map_err(|e| crate::errors::DbError::LogicalError(format!("failed to construct an incoming packet: {e}")))?
        {
            ChainPacketComponents::Final {
                packet_tag,
                ack_key,
                previous_hop,
                plain_text,
                ..
            } => {
                let ack = Acknowledgement::new(ack_key, pkt_keypair);

                Ok(TransportPacketWithChainData::Final {
                    packet_tag,
                    previous_hop,
                    plain_text,
                    ack,
                })
            }
            ChainPacketComponents::Forwarded {
                packet,
                ticket,
                ack_challenge,
                packet_tag,
                ack_key,
                previous_hop,
                own_key,
                next_hop,
                next_challenge,
                path_pos,
            } => {
                let myself = self.clone();

                let t = self
                    .begin_transaction()
                    .await?
                    .perform(|tx| {
                        Box::pin(async move {
                            let chain_data = myself.get_indexer_data(Some(tx)).await?;

                            let domain_separator = chain_data.channels_dst.ok_or_else(|| {
                                crate::errors::DbError::LogicalError("failed to fetch the domain separator".into())
                            })?;
                            let ticket_price = chain_data.ticket_price.ok_or_else(|| {
                                crate::errors::DbError::LogicalError("failed to fetch the ticket price".into())
                            })?;

                            let previous_hop_addr =
                                myself.resolve_chain_key(&previous_hop).await?.ok_or_else(|| {
                                    crate::errors::DbError::LogicalError(format!(
                                        "failed to find channel key for packet key {} on previous hop",
                                        previous_hop.to_peerid_str()
                                    ))
                                })?;

                            let next_hop_addr = myself.resolve_chain_key(&next_hop).await?.ok_or_else(|| {
                                crate::errors::DbError::LogicalError(format!(
                                    "failed to find channel key for packet key {} on next hop",
                                    next_hop.to_peerid_str()
                                ))
                            })?;

                            let channel = myself
                                .get_channel_by_id(Some(tx), generate_channel_id(&previous_hop_addr, &me_onchain))
                                .await?
                                .ok_or_else(|| {
                                    crate::errors::DbError::LogicalError(format!(
                                        "no channel found for previous hop address '{previous_hop_addr}'"
                                    ))
                                })?;

                            let unrealized_balance = myself
                                .unrealized_value
                                .get(&channel.get_id())
                                .await
                                .map(|balance| balance.sub(channel.balance))
                                .unwrap_or(channel.balance);

                            if let Err(e) = validate_unacknowledged_ticket(
                                &ticket,
                                &channel,
                                &previous_hop_addr,
                                ticket_price,
                                TICKET_WIN_PROB,
                                Some(unrealized_balance),
                                &domain_separator,
                            )
                            .await
                            {
                                // TODO: move this outside to the from_recv caller

                                // #[cfg(all(feature = "prometheus", not(test)))]
                                // METRIC_REJECTED_TICKETS_COUNT.increment();

                                myself.unacked_tickets.remove(&ack_challenge).await;
                                return Err(crate::errors::DbError::TicketValidationError(e.to_string()));
                            }

                            myself.increment_ticket_index(channel.get_id()).await?;

                            myself
                                .unacked_tickets
                                .insert(
                                    ack_challenge,
                                    PendingAcknowledgement::WaitingAsRelayer(UnacknowledgedTicket::new(
                                        ticket.clone(),
                                        own_key.clone(),
                                        previous_hop_addr,
                                    )),
                                )
                                .await;

                            // Check that the calculated path position from the ticket matches value from the packet header
                            let ticket_path_pos = ticket.get_path_position(ticket_price.amount())?;
                            if !ticket_path_pos.eq(&path_pos) {
                                return Err(crate::errors::DbError::LogicalError(format!(
                                    "path position mismatch: from ticket {ticket_path_pos}, from packet {path_pos}"
                                )));
                            }

                            // Create next ticket for the packet
                            let mut ticket = if ticket_path_pos == 1 {
                                Ok(hopr_internal_types::channels::Ticket::new_zero_hop(
                                    &next_hop_addr,
                                    &me,
                                    &domain_separator,
                                )?)
                            } else {
                                myself
                                    .create_multihop_ticket(
                                        Some(tx),
                                        me.public().to_address(),
                                        next_hop_addr,
                                        ticket_path_pos,
                                    )
                                    .await
                            }?;

                            // forward packet
                            ticket.challenge = next_challenge.to_ethereum_challenge();
                            ticket.sign(&me, &domain_separator);

                            Ok(ticket)
                        })
                    })
                    .await?;

                let ack = Acknowledgement::new(ack_key, pkt_keypair);

                let mut payload = Vec::with_capacity(ChainPacketComponents::SIZE);
                payload.extend_from_slice(packet.as_ref());
                payload.extend_from_slice(&t.to_bytes());

                Ok(TransportPacketWithChainData::Forwarded {
                    packet_tag,
                    previous_hop,
                    next_hop,
                    data: payload.into_boxed_slice(),
                    ack,
                })
            }
            ChainPacketComponents::Outgoing { .. } => Err(crate::errors::DbError::LogicalError(
                "Cannot receive an outgoing packet".into(),
            )),
        }
    }
}

impl HoprDb {
    async fn create_multihop_ticket<'a>(
        &'a self,
        tx: OptTx<'a>,
        me_onchain: Address,
        destination: Address,
        path_pos: u8,
    ) -> Result<hopr_internal_types::channels::Ticket> {
        let myself = self.clone();
        let (channel, ticket_price): (ChannelEntry, U256) = self
            .nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Ok::<_, DbError>(
                        if let Some(model) = hopr_db_entity::channel::Entity::find()
                            .filter(hopr_db_entity::channel::Column::Destination.eq(destination.to_string()))
                            .one(tx.as_ref())
                            .await?
                        {
                            let channel_id = Hash::from_str(&model.channel_id)?;

                            let old_ticket_index = myself.increment_ticket_index(channel_id).await?;

                            let ticket_price = myself.get_indexer_data(Some(tx)).await?.ticket_price;

                            Some((
                                {
                                    let mut channel: ChannelEntry = model.try_into()?;
                                    channel.ticket_index = old_ticket_index.into();
                                    channel
                                },
                                ticket_price
                                    .ok_or(DbError::LogicalError("missing ticket price".into()))?
                                    .amount(),
                            ))
                        } else {
                            None
                        },
                    )
                })
            })
            .await?
            .ok_or(crate::errors::DbError::LogicalError(format!(
                "channel '{destination}' not found",
            )))?;

        let amount = Balance::new(
            ticket_price.div_f64(TICKET_WIN_PROB).map_err(|e| {
                crate::errors::DbError::LogicalError(format!(
                    "winning probability outside of the allowed interval (0.0, 1.0]: {e}"
                ))
            })? * U256::from(path_pos - 1),
            BalanceType::HOPR,
        );

        if channel.balance.lt(&amount) {
            return Err(crate::errors::DbError::LogicalError(format!(
                "out of funds: {} with counterparty {destination}",
                channel.get_id()
            )));
        }

        let ticket = hopr_internal_types::channels::Ticket::new_partial(
            &me_onchain,
            &destination,
            &amount,
            channel.ticket_index,
            U256::one(), // unaggregated always have index_offset == 1
            TICKET_WIN_PROB,
            channel.channel_epoch,
        )
        .map_err(|e| crate::errors::DbError::LogicalError(format!("failed to construct a ticket: {e}")))?;

        //         #[cfg(all(feature = "prometheus", not(test)))]
        //         METRIC_TICKETS_COUNT.increment();

        Ok(ticket)
    }

    /// Used only by non-SQLite code and tests.
    pub async fn upsert_ticket<'a>(&'a self, tx: OptTx<'a>, acknowledged_ticket: AcknowledgedTicket) -> Result<()> {
        self.nest_transaction_in_db(tx, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    // For purpose of upserting, we must select only by the triplet (channel id, epoch, index)
                    let selector = TicketSelector::new(
                        acknowledged_ticket.ticket.channel_id,
                        acknowledged_ticket.ticket.channel_epoch,
                    )
                    .with_index(acknowledged_ticket.ticket.index);

                    let mut model = ticket::ActiveModel::from(acknowledged_ticket);

                    if let Some(ticket) = ticket::Entity::find()
                        .filter(SimpleExpr::from(selector))
                        .one(tx.as_ref())
                        .await?
                    {
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
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, Set};
    use std::sync::atomic::Ordering;

    use crate::accounts::HoprDbAccountOperations;
    use crate::channels::HoprDbChannelOperations;
    use crate::db::HoprDb;
    use crate::errors::DbError;
    use crate::info::{DomainSeparator, HoprDbInfoOperations};
    use crate::tickets::{AggregationPrerequisites, HoprDbTicketOperations, TicketSelector};
    use crate::{HoprDbGeneralModelOperations, TargetDb};

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
            .map(|i| generate_random_ack_ticket(i as u32))
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

        let stats = db.get_ticket_statistics(None).await.unwrap();
        assert_eq!(
            COUNT_TICKETS, stats.unredeemed_tickets,
            "must have {COUNT_TICKETS} unredeemed"
        );
        assert_eq!(
            BalanceType::HOPR.balance(TICKET_VALUE * COUNT_TICKETS),
            stats.unredeemed_value,
            "unredeemed balance must match"
        );
        assert_eq!(0, stats.redeemed_tickets, "there must be no redeemed tickets");
        assert_eq!(
            BalanceType::HOPR.zero(),
            stats.redeemed_value,
            "there must be 0 redeemed value"
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

        let stats = db.get_ticket_statistics(None).await.unwrap();
        assert_eq!(
            COUNT_TICKETS - TO_REDEEM,
            stats.unredeemed_tickets,
            "must have {COUNT_TICKETS} unredeemed"
        );
        assert_eq!(
            BalanceType::HOPR.balance(TICKET_VALUE * (COUNT_TICKETS - TO_REDEEM)),
            stats.unredeemed_value,
            "unredeemed balance must match"
        );
        assert_eq!(
            TO_REDEEM, stats.redeemed_tickets,
            "there must be {TO_REDEEM} redeemed tickets"
        );
        assert_eq!(
            BalanceType::HOPR.balance(TICKET_VALUE * TO_REDEEM),
            stats.redeemed_value,
            "there must be a redeemed value"
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

        let stats = db.get_ticket_statistics(None).await.unwrap();
        assert_eq!(
            COUNT_TICKETS, stats.unredeemed_tickets,
            "must have {COUNT_TICKETS} unredeemed"
        );
        assert_eq!(
            BalanceType::HOPR.balance(TICKET_VALUE * COUNT_TICKETS),
            stats.unredeemed_value,
            "unredeemed balance must match"
        );
        assert_eq!(0, stats.neglected_tickets, "there must be no redeemed tickets");
        assert_eq!(
            BalanceType::HOPR.zero(),
            stats.neglected_value,
            "there must be 0 redeemed value"
        );

        db.mark_tickets_neglected((&channel).into())
            .await
            .expect("should mark as neglected");

        let stats = db.get_ticket_statistics(None).await.unwrap();
        assert_eq!(0, stats.unredeemed_tickets, "must have 0 unredeemed");
        assert_eq!(
            BalanceType::HOPR.zero(),
            stats.unredeemed_value,
            "unredeemed balance must be zero"
        );
        assert_eq!(
            COUNT_TICKETS, stats.neglected_tickets,
            "there must be no redeemed tickets"
        );
        assert_eq!(
            BalanceType::HOPR.balance(TICKET_VALUE * COUNT_TICKETS),
            stats.neglected_value,
            "there must be a neglected value"
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

        let idx = db.get_ticket_index(hash).await.unwrap();
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
    async fn test_ticket_index_compare_and_set_and_increment() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let hash = Hash::default();

        let old_idx = db.compare_and_set_ticket_index(hash, 1).await.unwrap();
        assert_eq!(0, old_idx, "old value must be 0");

        let new_idx = db.get_ticket_index(hash).await.unwrap().load(Ordering::SeqCst);
        assert_eq!(1, new_idx, "new value must be 1");

        let old_idx = db.increment_ticket_index(hash).await.unwrap();
        assert_eq!(1, old_idx, "old value must be 1");

        let new_idx = db.get_ticket_index(hash).await.unwrap().load(Ordering::SeqCst);
        assert_eq!(2, new_idx, "new value must be 2");
    }

    #[async_std::test]
    async fn test_ticket_index_compare_and_set_must_not_decrease() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let hash = Hash::default();

        let new_idx = db.get_ticket_index(hash).await.unwrap().load(Ordering::SeqCst);
        assert_eq!(0, new_idx, "value must be 0");

        db.compare_and_set_ticket_index(hash, 1).await.unwrap();

        let new_idx = db.get_ticket_index(hash).await.unwrap().load(Ordering::SeqCst);
        assert_eq!(1, new_idx, "new value must be 1");

        let old_idx = db.compare_and_set_ticket_index(hash, 0).await.unwrap();
        assert_eq!(1, old_idx, "old value must be 1");
        assert_eq!(1, new_idx, "new value must be 1");

        let old_idx = db.compare_and_set_ticket_index(hash, 1).await.unwrap();
        assert_eq!(1, old_idx, "old value must be 1");
        assert_eq!(1, new_idx, "new value must be 1");
    }

    #[async_std::test]
    async fn test_ticket_index_reset() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let hash = Hash::default();

        let new_idx = db.get_ticket_index(hash).await.unwrap().load(Ordering::SeqCst);
        assert_eq!(0, new_idx, "value must be 0");

        db.compare_and_set_ticket_index(hash, 1).await.unwrap();

        let new_idx = db.get_ticket_index(hash).await.unwrap().load(Ordering::SeqCst);
        assert_eq!(1, new_idx, "new value must be 1");

        let old_idx = db.reset_ticket_index(hash).await.unwrap();
        assert_eq!(1, old_idx, "old value must be 1");

        let new_idx = db.get_ticket_index(hash).await.unwrap().load(Ordering::SeqCst);
        assert_eq!(0, new_idx, "new value must be 0");
    }

    #[async_std::test]
    async fn test_persist_ticket_indices() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let hash_1 = Hash::default();
        let hash_2 = Hash::from(hopr_crypto_random::random_bytes());

        db.get_ticket_index(hash_1).await.unwrap();
        db.compare_and_set_ticket_index(hash_2, 10).await.unwrap();

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

        db.compare_and_set_ticket_index(hash_1, 3).await.unwrap();
        db.increment_ticket_index(hash_2).await.unwrap();

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

        assert_eq!(tickets.len(), COUNT_TICKETS as usize);

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
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, None)
            .await
            .is_err());

        Ok(())
    }

    #[async_std::test]
    async fn test_ticket_aggregation_prepare_request_with_0_tickets_should_return_empty_result(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        const COUNT_TICKETS: usize = 0;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        assert_eq!(tickets.len(), COUNT_TICKETS as usize);

        let existing_channel_with_0_tickets = channel.get_id();

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_0_tickets, None)
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

        assert_eq!(tickets.len(), COUNT_TICKETS as usize);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, None)
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

        assert_eq!(tickets.len(), COUNT_TICKETS as usize);

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
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, None)
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
    async fn test_ticket_aggregation_prepare_request_with_some_requirements_should_pass_with_requirements_within_limits(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        const COUNT_TICKETS: usize = 5;
        const COUNT_JUST_WANTED_TICKETS: usize = 4;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        assert_eq!(tickets.len(), COUNT_TICKETS as usize);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let constraints = Some(AggregationPrerequisites {
            min_ticket_count: COUNT_JUST_WANTED_TICKETS,
            min_unaggregated_ratio: 0.0,
        });
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
    async fn test_ticket_aggregation_prepare_request_with_some_requirements_should_pass_with_requirements_outside_limits_not_marking_anything(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        const COUNT_TICKETS: usize = 2;

        let (db, channel, tickets) = create_alice_db_with_tickets_from_bob(COUNT_TICKETS).await?;

        assert_eq!(tickets.len(), COUNT_TICKETS as usize);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let constraints = Some(AggregationPrerequisites {
            min_ticket_count: COUNT_TICKETS + 1,
            min_unaggregated_ratio: 0.0,
        });
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
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, None)
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

        assert_eq!(tickets.len(), COUNT_TICKETS as usize);

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
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, None)
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
    async fn test_ticket_aggregation_should_replace_the_tickes_with_a_correctly_aggregated_ticket(
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

        assert_eq!(tickets.len(), COUNT_TICKETS as usize);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, None)
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

        assert_eq!(tickets.len(), COUNT_TICKETS as usize);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, None)
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

        assert_eq!(tickets.len(), COUNT_TICKETS as usize);

        let existing_channel_with_multiple_tickets = channel.get_id();

        let actual = db
            .prepare_aggregation_in_channel(&existing_channel_with_multiple_tickets, None)
            .await?;

        assert_eq!(actual, Some((BOB_OFFCHAIN.public().clone(), tickets)));

        assert!(db
            .process_received_aggregated_ticket(aggregated_ticket, &ALICE)
            .await
            .is_err());

        Ok(())
    }

    #[async_std::test]
    async fn test_aggregate_tickets() {
        //let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        // TODO
    }

    // TODO: think about incorporating these tests
    // #[async_std::test]
    // async fn test_ticket_workflow() {
    //     let mut db = CoreEthereumDb::new(
    //         DB::new(CurrentDbShim::new_in_memory().await),
    //         SENDER_PRIV_KEY.public().to_address(),
    //     );

    //     let hkc = HalfKeyChallenge::new(&random_bytes::<{ HalfKeyChallenge::SIZE }>());
    //     let unack = UnacknowledgedTicket::new(
    //         create_valid_ticket(),
    //         HalfKey::new(&random_bytes::<{ HalfKey::SIZE }>()),
    //         SENDER_PRIV_KEY.public().to_address(),
    //     );

    //     db.store_pending_acknowledgment(hkc, PendingAcknowledgement::WaitingAsRelayer(unack))
    //         .await
    //         .unwrap();
    //     let num_tickets = db.get_tickets(None).await.unwrap();
    //     assert_eq!(1, num_tickets.len(), "db should find one ticket");

    //     let pending = db
    //         .get_pending_acknowledgement(&hkc)
    //         .await
    //         .unwrap()
    //         .expect("db should contain pending ack");
    //     match pending {
    //         PendingAcknowledgement::WaitingAsSender => panic!("must not be pending as sender"),
    //         PendingAcknowledgement::WaitingAsRelayer(ticket) => {
    //             let ack = ticket
    //                 .acknowledge(&HalfKey::default(), &TARGET_PRIV_KEY, &Hash::default())
    //                 .unwrap();
    //             db.replace_unack_with_ack(&hkc, ack).await.unwrap();

    //             let num_tickets = db.get_tickets(None).await.unwrap().len();
    //             let num_unack = db.get_unacknowledged_tickets(None).await.unwrap().len();
    //             let num_ack = db.get_acknowledged_tickets(None).await.unwrap().len();
    //             assert_eq!(1, num_tickets, "db should find one ticket");
    //             assert_eq!(0, num_unack, "db should not contain any unacknowledged tickets");
    //             assert_eq!(1, num_ack, "db should contain exactly one acknowledged ticket");
    //         }
    //     }
    // }

    // #[async_std::test]
    // async fn test_db_should_store_ticket_index() {
    //     let mut db = CoreEthereumDb::new(
    //         DB::new(CurrentDbShim::new_in_memory().await),
    //         SENDER_PRIV_KEY.public().to_address(),
    //     );

    //     let dummy_channel = Hash::new(&[0xffu8; Hash::SIZE]);
    //     let dummy_index = U256::one();

    //     db.set_current_ticket_index(&dummy_channel, dummy_index).await.unwrap();
    //     let idx = db
    //         .get_current_ticket_index(&dummy_channel)
    //         .await
    //         .unwrap()
    //         .expect("db must contain ticket index");

    //     assert_eq!(dummy_index, idx, "ticket index mismatch");
    // }

    // #[async_std::test]
    // async fn test_db_should_increase_ticket_index() {
    //     let mut db = CoreEthereumDb::new(
    //         DB::new(CurrentDbShim::new_in_memory().await),
    //         SENDER_PRIV_KEY.public().to_address(),
    //     );

    //     let dummy_channel = Hash::new(&[0xffu8; Hash::SIZE]);

    //     // increase current ticket index of a non-existing channel, the result should be 1
    //     db.increase_current_ticket_index(&dummy_channel).await.unwrap();
    //     let idx = db
    //         .get_current_ticket_index(&dummy_channel)
    //         .await
    //         .unwrap()
    //         .expect("db must contain ticket index");
    //     assert_eq!(idx, U256::one(), "ticket index mismatch. Expecting 1");

    //     // increase current ticket index of an existing channel where previous value is 1, the result should be 2
    //     db.increase_current_ticket_index(&dummy_channel).await.unwrap();
    //     let idx = db
    //         .get_current_ticket_index(&dummy_channel)
    //         .await
    //         .unwrap()
    //         .expect("db must contain ticket index");
    //     assert_eq!(idx, 2_u32.into(), "ticket index mismatch. Expecting 2");
    // }

    // #[async_std::test]
    // async fn test_db_should_ensure_ticket_index_not_smaller_than_given_index() {
    //     let mut db = CoreEthereumDb::new(
    //         DB::new(CurrentDbShim::new_in_memory().await),
    //         SENDER_PRIV_KEY.public().to_address(),
    //     );

    //     let dummy_channel = Hash::new(&[0xffu8; Hash::SIZE]);
    //     let dummy_index = 123_u32.into();

    //     // the ticket index should be equal or greater than the given dummy index
    //     db.ensure_current_ticket_index_gte(&dummy_channel, dummy_index)
    //         .await
    //         .unwrap();
    //     let idx = db
    //         .get_current_ticket_index(&dummy_channel)
    //         .await
    //         .unwrap()
    //         .expect("db must contain ticket index");
    //     assert_eq!(idx, dummy_index, "ticket index mismatch. Expecting 2");
    // }
}
