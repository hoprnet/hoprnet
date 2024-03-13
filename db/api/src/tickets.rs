use async_stream::stream;
use std::ops::Add;
use std::ops::Sub;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicUsize, Arc};
use std::time::SystemTime;
use tracing::error;

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::TryStreamExt;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Set, Value};
use sea_query::{Expr, SimpleExpr};
use tracing::{debug, instrument, trace};

use hopr_crypto_packet::{chain::ChainPacketComponents, validation::validate_unacknowledged_ticket};
use hopr_crypto_types::prelude::*;
use hopr_db_entity::conversions::tickets::model_to_acknowledged_ticket;
use hopr_db_entity::prelude::{Ticket, TicketStatistics};
use hopr_db_entity::ticket;
use hopr_db_entity::ticket_statistics;
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

impl From<&AcknowledgedTicket> for TicketSelector {
    fn from(value: &AcknowledgedTicket) -> Self {
        Self {
            channel_id: value.ticket.channel_id,
            epoch: value.ticket.channel_epoch.into(),
            index: Some(value.ticket.index.into()),
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

pub struct RunningAggregation {}

#[async_trait]
pub trait HoprDbTicketOperations {
    async fn get_ticket<'a>(
        &'a self,
        tx: OptTx<'a>,
        selector: TicketSelector,
        // To be removed with https://github.com/hoprnet/hoprnet/pull/6018
        chain_keypair: &ChainKeypair,
    ) -> Result<Option<AcknowledgedTicket>>;

    async fn mark_tickets_redeemed(&self, selector: TicketSelector) -> Result<usize>;

    async fn mark_tickets_neglected(&self, selector: TicketSelector) -> Result<usize>;

    // Remove ChainKeypair once https://github.com/hoprnet/hoprnet/pull/6018 is merged
    async fn update_ticket_states<'a>(
        &'a self,
        selector: TicketSelector,
        new_state: AcknowledgedTicketStatus,
        chain_keypair: &'a ChainKeypair,
    ) -> Result<BoxStream<'a, AcknowledgedTicket>>;

    async fn get_ticket_statistics<'a>(&'a self, tx: OptTx<'a>) -> Result<AllTicketStatistics>;

    async fn get_tickets_value<'a>(&'a self, tx: OptTx<'a>, selector: TicketSelector) -> Result<(usize, Balance)>;

    async fn invalidate_cached_ticket_index(&self, channel_id: &Hash);

    async fn get_cached_ticket_index(&self, channel_id: &Hash) -> Option<Arc<AtomicUsize>>;

    async fn calculate_aggregatable_tickets_in_channel(
        &self,
        channel: &Hash,
        chain_keypair: &ChainKeypair,
    ) -> Result<(u32, Balance)>;

    async fn prepare_aggregation_in_channel(
        &self,
        channel: &Hash,
        chain_keypair: &ChainKeypair,
    ) -> Result<(OffchainPublicKey, Vec<AcknowledgedTicket>)>;

    async fn process_received_aggregated_ticket(
        &self,
        aggregated_ticket: hopr_internal_types::channels::Ticket,
        chain_keypair: &ChainKeypair,
    ) -> Result<AcknowledgedTicket>;

    async fn aggregate_tickets(
        &mut self,
        destination: OffchainPublicKey,
        mut acked_tickets: Vec<AcknowledgedTicket>,
    ) -> Result<Ticket>;

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
    async fn get_ticket<'a>(
        &'a self,
        tx: OptTx<'a>,
        selector: TicketSelector,
        chain_keypair: &ChainKeypair,
    ) -> Result<Option<AcknowledgedTicket>> {
        assert!(
            selector.index.is_some(),
            "ticket index must be specified in the selector to fetch a single ticket"
        );

        let channel_dst = self
            .get_chain_data(tx)
            .await?
            .channels_dst
            .ok_or(LogicalError("missing channel dst".into()))?;

        let ticket = self
            .nest_transaction_in_db(tx, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Ok::<_, DbError>(
                        ticket::Entity::find()
                            .filter(SimpleExpr::from(selector))
                            .one(tx.as_ref())
                            .await?,
                    )
                })
            })
            .await?;

        match ticket {
            None => Ok(None),
            Some(ticket_model) => Ok(Some(model_to_acknowledged_ticket(
                &ticket_model,
                channel_dst,
                chain_keypair,
            )?)),
        }
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
        &mut self,
        destination: OffchainPublicKey,
        mut acked_tickets: Vec<AcknowledgedTicket>,
    ) -> Result<Ticket> {
        // let domain_separator = self
        //     .db
        //     .read()
        //     .await
        //     .get_channels_domain_separator()
        //     .await?
        //     .ok_or_else(|| {
        //         warn!("Missing domain separator");
        //         ProtocolTicketAggregation("Missing domain separator".into())
        //     })?;

        // let destination = self
        //     .db
        //     .read()
        //     .await
        //     .get_chain_key(
        //         &OffchainPublicKey::try_from(destination)
        //             .expect("Invalid PeerId. Could not convert to OffchainPublicKey"),
        //     )
        //     .await?
        //     .ok_or_else(|| {
        //         warn!("Could not find chain key for {}", destination);
        //         ProtocolTicketAggregation("Could not find chain key".into())
        //     })?;

        // let channel_id = generate_channel_id(&(&self.chain_key).into(), &destination);
        // let channel_entry = self
        //     .db
        //     .read()
        //     .await
        //     .get_channel(&channel_id)
        //     .await?
        //     .ok_or(ProtocolTicketAggregation(format!(
        //         "channel {channel_id} does not exist"
        //     )))?;
        // let channel_balance = channel_entry.balance;

        // acked_tickets.sort();
        // acked_tickets.dedup();

        // let channel_epoch = channel_entry.channel_epoch;

        // let mut final_value = Balance::zero(BalanceType::HOPR);

        // for (i, acked_ticket) in acked_tickets.iter().enumerate() {
        //     if channel_id != acked_ticket.ticket.channel_id {
        //         return Err(ProtocolTicketAggregation(format!(
        //             "aggregated ticket has an invalid channel id {}",
        //             acked_ticket.ticket.channel_id
        //         )));
        //     }

        //     if U256::from(acked_ticket.ticket.channel_epoch) != channel_epoch {
        //         return Err(ProtocolTicketAggregation("Channel epochs do not match".to_owned()));
        //     }

        //     if i + 1 < acked_tickets.len()
        //         && acked_ticket.ticket.index + acked_ticket.ticket.index_offset as u64
        //             > acked_tickets[i + 1].ticket.index
        //     {
        //         return Err(ProtocolTicketAggregation(
        //             "Tickets with overlapping index intervals".to_owned(),
        //         ));
        //     }

        //     if acked_ticket
        //         .verify(&(&self.chain_key).into(), &destination, &domain_separator)
        //         .is_err()
        //     {
        //         return Err(ProtocolTicketAggregation("Not a valid ticket".to_owned()));
        //     }

        //     if !acked_ticket.is_winning_ticket(&domain_separator) {
        //         return Err(ProtocolTicketAggregation("Not a winning ticket".to_owned()));
        //     }

        //     final_value = final_value.add(&acked_ticket.ticket.amount);
        //     if final_value.gt(&channel_balance) {
        //         return Err(ProtocolTicketAggregation(format!("ticket amount to aggregate {final_value} is greater than the balance {channel_balance} of channel {channel_id}")));
        //     }

        //     #[cfg(all(feature = "prometheus", not(test)))]
        //     METRIC_AGGREGATED_TICKETS.increment();
        // }

        // info!(
        //     "aggregated {} tickets in channel {channel_id} with total value {final_value}",
        //     acked_tickets.len()
        // );

        // let first_acked_ticket = acked_tickets.first().unwrap();
        // let last_acked_ticket = acked_tickets.last().unwrap();

        // #[cfg(all(feature = "prometheus", not(test)))]
        // METRIC_AGGREGATION_COUNT.increment();

        // trace!("after ticket aggregation, ensure the current ticket index is larger than the last index and the on-chain index");
        // // calculate the minimum current ticket index as the larger value from the acked ticket index and on-chain ticket_index from channel_entry
        // let current_ticket_index_from_acked_tickets = U256::from(last_acked_ticket.ticket.index).add(1);
        // let current_ticket_index_gte = current_ticket_index_from_acked_tickets.max(channel_entry.ticket_index);
        // {
        //     self.db
        //         .write()
        //         .await
        //         .ensure_current_ticket_index_gte(&channel_id, current_ticket_index_gte)
        //         .await?;
        // }

        // Ticket::new(
        //     &destination,
        //     &final_value,
        //     first_acked_ticket.ticket.index.into(),
        //     (last_acked_ticket.ticket.index - first_acked_ticket.ticket.index + 1).into(),
        //     1.0, // Aggregated tickets have always 100% winning probability
        //     channel_epoch,
        //     first_acked_ticket.ticket.challenge.clone(),
        //     &self.chain_key,
        //     &domain_separator,
        // )
        // .map_err(|e| e.into())
        Err(DbError::LogicalError("TODO".into()))
    }

    async fn process_received_aggregated_ticket(
        &self,
        aggregated_ticket: hopr_internal_types::channels::Ticket,
        chain_keypair: &ChainKeypair,
    ) -> Result<AcknowledgedTicket> {
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
                            .get_channel_by_id(Some(tx), channel_id.clone())
                            .await?
                            .ok_or(DbError::ChannelNotFound(channel_id))?;

                        if entry.status != ChannelStatus::Open {
                            return Err(DbError::LogicalError(format!("channel '{channel_id}' not open")));
                        }

                        // TODO: this should not be needed, because there are no tickets
                        // for channels not associated with us?
                        // // Perform sanity checks on the arguments
                        // assert_eq!(
                        //     ChannelDirection::Incoming,
                        //     channel.direction(&self.me).expect("must be own channel"),
                        //     "aggregation request can happen on incoming channels only"
                        // );

                        let domain_separator =
                            myself.get_chain_data(Some(tx)).await?.channels_dst.ok_or_else(|| {
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
            .map(|m| model_to_acknowledged_ticket(&m, domain_separator, &chain_keypair).map_err(DbError::from))
            .collect::<Result<Vec<AcknowledgedTicket>>>()?;

        // can be done, because the tickets collection is tested for emptiness before
        let first_stored_ticket = acknowledged_tickets.first().unwrap();

        // calculate the new current ticket index
        let current_ticket_index_from_aggregated_ticket =
            U256::from(aggregated_ticket.index).add(aggregated_ticket.index_offset);

        let acked_aggregated_ticket = AcknowledgedTicket::new(
            aggregated_ticket,
            first_stored_ticket.response.clone(),
            first_stored_ticket.signer,
            &chain_keypair,
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
                        .filter(
                            hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8),
                        )
                        .exec(tx.as_ref())
                        .await?;

                    // TODO: check that delete row count == length of current collection

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

    async fn calculate_aggregatable_tickets_in_channel(
        &self,
        channel: &Hash,
        chain_keypair: &ChainKeypair,
    ) -> Result<(u32, Balance)> {
        let myself = self.clone();
        let chain_keypair = chain_keypair.clone();

        let channel = *channel;

        let (channel_entry, ds) =
            self.nest_transaction_in_db(None, TargetDb::Index)
                .await?
                .perform(|tx| {
                    Box::pin(async move {
                        let entry = myself
                            .get_channel_by_id(Some(tx), channel.clone())
                            .await?
                            .ok_or(DbError::ChannelNotFound(channel))?;

                        if entry.status != ChannelStatus::Open {
                            return Err(DbError::LogicalError(format!("channel '{channel}' not open")));
                        }

                        // TODO: this should not be needed, because there are no tickets
                        // for channels not associated with us?
                        // // Perform sanity checks on the arguments
                        // assert_eq!(
                        //     ChannelDirection::Incoming,
                        //     channel.direction(&self.me).expect("must be own channel"),
                        //     "aggregation request can happen on incoming channels only"
                        // );

                        let domain_separator =
                            myself.get_chain_data(Some(tx)).await?.channels_dst.ok_or_else(|| {
                                crate::errors::DbError::LogicalError("domain separator missing".into())
                            })?;

                        Ok((entry, domain_separator))
                    })
                })
                .await?;

        let tickets = self
            .nest_transaction_in_db(None, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    // verify that no aggregation is in progress in the channel
                    if ticket::Entity::find()
                        .filter(hopr_db_entity::ticket::Column::ChannelId.eq(channel_entry.get_id().to_hex()))
                        .filter(
                            hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8),
                        )
                        .one(tx.as_ref())
                        .await?
                        .is_some()
                    {
                        return Ok(vec![]);
                    }

                    // find the index of the last ticket being redeemed
                    let idx_of_last_being_redeemed = ticket::Entity::find()
                        .filter(hopr_db_entity::ticket::Column::ChannelId.eq(channel_entry.get_id().to_hex()))
                        .filter(hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingRedeemed as u8))
                        .order_by_desc(hopr_db_entity::ticket::Column::Index)
                        .one(tx.as_ref())
                        .await?
                        .map(|m| m.id)
                        .unwrap_or(i32::MIN); // go from the lowest possible index of none is found

                    // get the list of all tickets to be aggregated
                    let to_be_aggregated = ticket::Entity::find()
                        .filter(hopr_db_entity::ticket::Column::Index.gt(idx_of_last_being_redeemed))
                        .filter(
                            hopr_db_entity::ticket::Column::State.ne(AcknowledgedTicketStatus::BeingAggregated as u8),
                        )
                        .all(tx.as_ref())
                        .await?;

                    to_be_aggregated
                        .into_iter()
                        .map(|m| model_to_acknowledged_ticket(&m, ds, &chain_keypair).map_err(DbError::from))
                        .collect::<Result<Vec<AcknowledgedTicket>>>()
                })
            })
            .await?;

        Ok(tickets
            .into_iter()
            .fold((0, Balance::zero(BalanceType::HOPR)), |acc, ack_tkt| {
                (acc.0 + 1, acc.1.add(ack_tkt.ticket.amount))
            }))
    }

    async fn prepare_aggregation_in_channel(
        &self,
        channel: &Hash,
        chain_keypair: &ChainKeypair,
    ) -> Result<(OffchainPublicKey, Vec<AcknowledgedTicket>)> {
        let myself = self.clone();
        let chain_keypair = chain_keypair.clone();

        let channel = *channel;

        let (channel_entry, peer, ds) =
            self.nest_transaction_in_db(None, TargetDb::Index)
                .await?
                .perform(|tx| {
                    Box::pin(async move {
                        let entry = myself
                            .get_channel_by_id(Some(tx), channel.clone())
                            .await?
                            .ok_or(DbError::ChannelNotFound(channel))?;

                        if entry.status != ChannelStatus::Open {
                            return Err(DbError::LogicalError(format!("channel '{channel}' not open")));
                        }

                        // TODO: this should not be needed, because there are no tickets
                        // for channels not associated with us?
                        // // Perform sanity checks on the arguments
                        // assert_eq!(
                        //     ChannelDirection::Incoming,
                        //     channel.direction(&self.me).expect("must be own channel"),
                        //     "aggregation request can happen on incoming channels only"
                        // );

                        let pk = myself
                            .resolve_packet_key(&entry.source)
                            .await?
                            .ok_or(DbError::LogicalError(format!(
                                "peer '{}' has no offchain key record",
                                entry.source
                            )))?;

                        let domain_separator =
                            myself.get_chain_data(Some(tx)).await?.channels_dst.ok_or_else(|| {
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
                    // verify that no aggregation is in progress in the channel
                    if ticket::Entity::find()
                        .filter(hopr_db_entity::ticket::Column::ChannelId.eq(channel_entry.get_id().to_hex()))
                        .filter(
                            hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingAggregated as u8),
                        )
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
                    let idx_of_last_being_redeemed = ticket::Entity::find()
                        .filter(hopr_db_entity::ticket::Column::ChannelId.eq(channel_entry.get_id().to_hex()))
                        .filter(hopr_db_entity::ticket::Column::State.eq(AcknowledgedTicketStatus::BeingRedeemed as u8))
                        .order_by_desc(hopr_db_entity::ticket::Column::Index)
                        .one(tx.as_ref())
                        .await?
                        .map(|m| m.id)
                        .unwrap_or(i32::MIN); // go from the lowest possible index of none is found

                    // get the list of all tickets to be aggregated
                    let to_be_aggregated = ticket::Entity::find()
                        .filter(hopr_db_entity::ticket::Column::Index.gt(idx_of_last_being_redeemed))
                        .filter(
                            hopr_db_entity::ticket::Column::State.ne(AcknowledgedTicketStatus::BeingAggregated as u8),
                        )
                        .all(tx.as_ref())
                        .await?;

                    // do a balance check to be sure not to aggregate more than current channel stake
                    let mut total_balance = Balance::zero(BalanceType::HOPR);
                    let mut last_idx_to_take = i32::MIN;

                    while let Some(m) = to_be_aggregated.iter().next() {
                        total_balance = total_balance + BalanceType::HOPR.balance_bytes(&m.amount);
                        if total_balance.gt(&channel_entry.balance) {
                            break;
                        } else {
                            last_idx_to_take = m.id;
                        }
                    }

                    let to_be_aggregated = to_be_aggregated
                        .into_iter()
                        .take_while(|m| m.id != last_idx_to_take)
                        .map(|m| model_to_acknowledged_ticket(&m, ds, &chain_keypair).map_err(DbError::from))
                        .collect::<Result<Vec<AcknowledgedTicket>>>()?;

                    // mark all tickets with appropriate characteristics as being aggregated
                    let to_be_aggregated_count = to_be_aggregated.len();
                    if to_be_aggregated_count > 0 {
                        let marked: sea_orm::UpdateResult = ticket::Entity::update_many()
                            .filter(hopr_db_entity::ticket::Column::Index.gt(idx_of_last_being_redeemed))
                            .filter(hopr_db_entity::ticket::Column::Index.lte(last_idx_to_take))
                            .filter(
                                hopr_db_entity::ticket::Column::State
                                    .ne(AcknowledgedTicketStatus::BeingAggregated as u8),
                            )
                            .col_expr(
                                hopr_db_entity::ticket::Column::State,
                                Expr::value(Value::Int(Some(AcknowledgedTicketStatus::BeingAggregated as u8 as i32))),
                            )
                            .exec(tx.as_ref())
                            .await?;

                        if marked.rows_affected as usize != to_be_aggregated_count {
                            return Err(DbError::LogicalError(format!(
                                "expected to mark {to_be_aggregated_count}, but was only able to mark {}",
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

        Ok((peer, tickets))
    }

    async fn update_ticket_states<'a>(
        &'a self,
        selector: TicketSelector,
        new_state: AcknowledgedTicketStatus,
        chain_keypair: &'a ChainKeypair,
    ) -> Result<BoxStream<'a, AcknowledgedTicket>> {
        let channel_dst = self
            .get_chain_data(None)
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
                                tracing::error!("failed to update ticket in the db: {e}");
                            }
                        }

                        match model_to_acknowledged_ticket(&ticket, channel_dst, &chain_keypair) {
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

    async fn invalidate_cached_ticket_index(&self, channel_id: &Hash) {
        self.ticket_index.invalidate(channel_id).await;
    }

    async fn get_cached_ticket_index(&self, channel_id: &Hash) -> Option<Arc<AtomicUsize>> {
        let db = self.clone();
        self.ticket_index
            .optionally_get_with_by_ref(channel_id, async move {
                db.get_channel_by_id(None, *channel_id)
                    .await
                    .ok() // TODO: log the error here
                    .flatten()
                    .map(|c| Arc::new(AtomicUsize::from(c.ticket_index.as_usize())))
            })
            .await
    }

    #[instrument(level = "trace", skip(self))]
    async fn handle_acknowledgement(&self, ack: Acknowledgement, me: ChainKeypair) -> Result<AckResult> {
        let myself = self.clone();

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
                                .get_channel_from(Some(tx), unacknowledged.signer)
                                .await?
                                .is_some_and(|c| c.channel_epoch.as_u32() != unacknowledged.ticket.channel_epoch)
                            {
                                return Err(crate::errors::DbError::LogicalError(format!(
                                    "no channel found for  address '{}'",
                                    unacknowledged.signer
                                )));
                            }

                            let domain_separator =
                                myself.get_chain_data(Some(tx)).await?.channels_dst.ok_or_else(|| {
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

                    let domain_separator = myself.get_chain_data(Some(tx)).await?.channels_dst.ok_or_else(|| {
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
                            let chain_data = myself.get_chain_data(Some(tx)).await?;

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

                            let channel =
                                myself
                                    .get_channel_from(Some(tx), previous_hop_addr)
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

                            let ticket_index = myself
                                .ticket_index
                                .get_with(channel.get_id(), async {
                                    let channel_ticket_index = channel.ticket_index.as_usize();
                                    Arc::new(AtomicUsize::from(channel_ticket_index))
                                })
                                .await;
                            ticket_index.fetch_add(1, Ordering::SeqCst);

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

                            let ticket_index = myself
                                .ticket_index
                                .get_with(channel_id, async {
                                    let channel_ticket_index =
                                        U256::from_be_bytes(model.ticket_index.clone()).as_usize();
                                    Arc::new(AtomicUsize::from(channel_ticket_index))
                                })
                                .await;

                            let old_ticket_index = ticket_index.fetch_add(1, Ordering::SeqCst) as u128;

                            let ticket_price = myself.get_chain_data(Some(tx)).await?.ticket_price;

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

    /// Used only by non-SQLite code
    #[allow(dead_code)]
    async fn insert_ticket<'a>(&'a self, tx: OptTx<'a>, acknowledged_ticket: AcknowledgedTicket) -> Result<()> {
        self.nest_transaction_in_db(tx, TargetDb::Tickets)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Ok::<_, DbError>(
                        ticket::ActiveModel::from(acknowledged_ticket)
                            .insert(tx.as_ref())
                            .await?,
                    )
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

    use crate::channels::HoprDbChannelOperations;
    use crate::db::HoprDb;
    use crate::errors::DbError;
    use crate::info::{DomainSeparator, HoprDbInfoOperations};
    use crate::tickets::{HoprDbTicketOperations, TicketSelector};
    use crate::{HoprDbGeneralModelOperations, TargetDb};

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();
    }

    const TICKET_VALUE: u64 = 100_000;

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
            BalanceType::HOPR.balance(100_u32),
            (count_tickets + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        db.insert_channel(None, channel).await.unwrap();

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
                        db_clone.insert_ticket(Some(tx), t).await?;
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
        let db = HoprDb::new_in_memory().await;
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
            .get_ticket(None, (&ack_ticket).into(), &ALICE)
            .await
            .expect("should get ticket")
            .expect("ticket should exist");

        assert_eq!(ack_ticket, db_ticket, "tickets must be equal");
    }

    #[async_std::test]
    async fn test_mark_redeemed() {
        let db = HoprDb::new_in_memory().await;
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
        let db = HoprDb::new_in_memory().await;

        let ticket = init_db_with_tickets(&db, 1).await.1.pop().unwrap();

        db.mark_tickets_redeemed((&ticket).into()).await.expect("must not fail");
        assert_eq!(
            0,
            db.mark_tickets_redeemed((&ticket).into()).await.expect("must not fail")
        );
    }

    #[async_std::test]
    async fn test_mark_redeem_should_redeem_all_tickets() {
        let db = HoprDb::new_in_memory().await;

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
        let db = HoprDb::new_in_memory().await;
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
    async fn test_update_tickets_state() {
        let db = HoprDb::new_in_memory().await;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await
            .unwrap();

        let channel = init_db_with_tickets(&db, 10).await.0;

        let mut selector: TicketSelector = (&channel).into();
        selector.index = Some(5);

        let v: Vec<AcknowledgedTicket> = db
            .update_ticket_states(selector, AcknowledgedTicketStatus::BeingRedeemed, &ALICE)
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

        let mut selector: TicketSelector = (&channel).into();
        selector.state = Some(AcknowledgedTicketStatus::Untouched);

        let v: Vec<AcknowledgedTicket> = db
            .update_ticket_states(selector, AcknowledgedTicketStatus::BeingRedeemed, &ALICE)
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
