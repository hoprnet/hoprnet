use async_trait::async_trait;
use hopr_crypto_packet::chain::ChainPacketComponents;
use hopr_crypto_packet::validation::validate_unacknowledged_ticket;
use hopr_crypto_types::prelude::*;
use hopr_db_api::errors::Result;
use hopr_db_api::protocol::{
    AckResult, HoprDbProtocolOperations, ResolvedAcknowledgement, TransportPacketWithChainData,
};
use hopr_db_api::resolver::HoprDbResolverOperations;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use std::ops::Sub;
use tracing::{instrument, trace, warn};

use crate::channels::HoprDbChannelOperations;
use crate::db::HoprDb;
use crate::errors::DbSqlError;
use crate::info::HoprDbInfoOperations;
use crate::prelude::HoprDbTicketOperations;
use crate::{HoprDbGeneralModelOperations, OptTx};

use hopr_parallelize::cpu::spawn_fifo_blocking;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_INCOMING_WIN_PROB: hopr_metrics::SimpleHistogram =
        hopr_metrics::SimpleHistogram::new(
            "hopr_tickets_incoming_win_probability",
            "Observes the winning probabilities on incoming tickets",
            vec![0.0, 0.0001, 0.001, 0.01, 0.05, 0.1, 0.15, 0.25, 0.3, 0.5],
        ).unwrap();
}

#[async_trait]
impl HoprDbProtocolOperations for HoprDb {
    #[instrument(level = "trace", skip(self, ack, me), ret)]
    async fn handle_acknowledgement(&self, ack: Acknowledgement, me: &ChainKeypair) -> Result<AckResult> {
        let myself = self.clone();
        let me_ckp = me.clone();

        let result = self
            .begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    match myself
                        .caches
                        .unacked_tickets
                        .remove(&ack.ack_challenge())
                        .await
                        .ok_or_else(|| {
                            DbSqlError::AcknowledgementValidationError(format!(
                                "received unexpected acknowledgement for half key challenge {} - half key {}",
                                ack.ack_challenge().to_hex(),
                                ack.ack_key_share.to_hex()
                            ))
                        })? {
                        PendingAcknowledgement::WaitingAsSender => {
                            trace!("received acknowledgement as sender: first relayer has processed the packet.");

                            Ok(ResolvedAcknowledgement::Sending(ack.ack_challenge()))
                        }

                        PendingAcknowledgement::WaitingAsRelayer(unacknowledged) => {
                            if myself
                                .get_channel_by_parties(
                                    Some(tx),
                                    unacknowledged.ticket.verified_issuer(),
                                    &myself.me_onchain,
                                    true,
                                )
                                .await?
                                .is_some_and(|c| {
                                    c.channel_epoch.as_u32() != unacknowledged.verified_ticket().channel_epoch
                                })
                            {
                                return Err(DbSqlError::LogicalError(format!(
                                    "no channel found for  address '{}'",
                                    unacknowledged.ticket.verified_issuer()
                                )));
                            }

                            let domain_separator = myself
                                .get_indexer_data(Some(tx))
                                .await?
                                .channels_dst
                                .ok_or_else(|| DbSqlError::LogicalError("domain separator missing".into()))?;

                            hopr_parallelize::cpu::spawn_blocking(move || {
                                // This explicitly checks whether the acknowledgement
                                // solves the challenge on the ticket. It must be done before we
                                // check that the ticket is winning, which is a lengthy operation
                                // and should not be done for bogus unacknowledged tickets
                                let ack_ticket = unacknowledged.acknowledge(&ack.ack_key_share)?;

                                if ack_ticket.is_winning(&me_ckp, &domain_separator) {
                                    trace!("Found a winning ticket");
                                    Ok(ResolvedAcknowledgement::RelayingWin(ack_ticket))
                                } else {
                                    trace!("Found a losing ticket");
                                    Ok(ResolvedAcknowledgement::RelayingLoss(
                                        ack_ticket.ticket.verified_ticket().channel_id,
                                    ))
                                }
                            })
                            .await
                        }
                    }
                })
            })
            .await?;

        match &result {
            ResolvedAcknowledgement::RelayingWin(ack_ticket) => {
                self.ticket_manager.insert_ticket(ack_ticket.clone()).await?;

                #[cfg(all(feature = "prometheus", not(test)))]
                {
                    let verified_ticket = ack_ticket.ticket.verified_ticket();
                    let channel = verified_ticket.channel_id.to_string();
                    crate::tickets::METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                        &[&channel, "unredeemed"],
                        self.ticket_manager
                            .unrealized_value(hopr_db_api::tickets::TicketSelector::new(
                                verified_ticket.channel_id,
                                verified_ticket.channel_epoch,
                            ))
                            .await?
                            .amount()
                            .as_u128() as f64,
                    );
                    crate::tickets::METRIC_HOPR_TICKETS_INCOMING_STATISTICS
                        .increment(&[&channel, "winning_count"], 1.0f64);
                }
            }
            ResolvedAcknowledgement::RelayingLoss(_channel) => {
                #[cfg(all(feature = "prometheus", not(test)))]
                {
                    crate::tickets::METRIC_HOPR_TICKETS_INCOMING_STATISTICS
                        .increment(&[&_channel.to_string(), "losing_count"], 1.0f64);
                }
            }
            _ => {}
        };

        Ok(result.into())
    }

    #[tracing::instrument(level = "trace", skip(self, data, me, path))]
    async fn to_send(
        &self,
        data: Box<[u8]>,
        me: ChainKeypair,
        path: Vec<OffchainPublicKey>,
        outgoing_ticket_win_prob: f64,
    ) -> Result<TransportPacketWithChainData> {
        let myself = self.clone();
        let next_peer = myself.resolve_chain_key(&path[0]).await?.ok_or_else(|| {
            DbSqlError::LogicalError(format!(
                "failed to find chain key for packet key {} on previous hop",
                path[0].to_peerid_str()
            ))
        })?;

        let components =
            self.begin_transaction()
                .await?
                .perform(|tx| {
                    Box::pin(async move {
                        let domain_separator =
                            myself.get_indexer_data(Some(tx)).await?.channels_dst.ok_or_else(|| {
                                DbSqlError::LogicalError("failed to fetch the domain separator".into())
                            })?;

                        // Decide whether to create 0-hop or multihop ticket
                        let next_ticket = if path.len() == 1 {
                            TicketBuilder::zero_hop().direction(&myself.me_onchain, &next_peer)
                        } else {
                            myself
                                .create_multihop_ticket(
                                    Some(tx),
                                    me.public().to_address(),
                                    next_peer,
                                    path.len() as u8,
                                    outgoing_ticket_win_prob,
                                )
                                .await?
                        };

                        spawn_fifo_blocking(move || {
                            ChainPacketComponents::into_outgoing(&data, &path, &me, next_ticket, &domain_separator)
                                .map_err(|e| {
                                    DbSqlError::LogicalError(format!(
                                        "failed to construct chain components for a packet: {e}"
                                    ))
                                })
                        })
                        .await
                    })
                })
                .await?;

        match components {
            ChainPacketComponents::Final { .. } | ChainPacketComponents::Forwarded { .. } => {
                Err(DbSqlError::LogicalError("Must contain an outgoing packet type".into()).into())
            }
            ChainPacketComponents::Outgoing {
                packet,
                ticket,
                next_hop,
                ack_challenge,
            } => {
                self.caches
                    .unacked_tickets
                    .insert(ack_challenge, PendingAcknowledgement::WaitingAsSender)
                    .await;

                let payload = spawn_fifo_blocking(move || {
                    let mut payload = Vec::with_capacity(ChainPacketComponents::SIZE);
                    payload.extend_from_slice(packet.as_ref());

                    let ticket_bytes: [u8; Ticket::SIZE] = ticket.into();
                    payload.extend_from_slice(ticket_bytes.as_ref());

                    payload.into_boxed_slice()
                })
                .await;

                Ok(TransportPacketWithChainData::Outgoing {
                    next_hop,
                    ack_challenge,
                    data: payload,
                })
            }
        }
    }

    #[tracing::instrument(level = "trace", skip(self, data, me, pkt_keypair, sender), fields(sender = %sender))]
    async fn from_recv(
        &self,
        data: Box<[u8]>,
        me: ChainKeypair,
        pkt_keypair: &OffchainKeypair,
        sender: OffchainPublicKey,
        outgoing_ticket_win_prob: f64,
    ) -> Result<TransportPacketWithChainData> {
        let offchain_keypair = pkt_keypair.clone();

        let packet = spawn_fifo_blocking(move || {
            ChainPacketComponents::from_incoming(&data, &offchain_keypair, sender)
                .map_err(|e| DbSqlError::LogicalError(format!("failed to construct an incoming packet: {e}")))
        })
        .await?;

        match packet {
            ChainPacketComponents::Final {
                packet_tag,
                ack_key,
                previous_hop,
                plain_text,
                ..
            } => {
                let offchain_keypair = pkt_keypair.clone();
                let ack = spawn_fifo_blocking(move || Acknowledgement::new(ack_key, &offchain_keypair)).await;

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
                let previous_hop_addr = myself.resolve_chain_key(&previous_hop).await?.ok_or_else(|| {
                    DbSqlError::LogicalError(format!(
                        "failed to find channel key for packet key {} on previous hop",
                        previous_hop.to_peerid_str()
                    ))
                })?;

                let next_hop_addr = myself.resolve_chain_key(&next_hop).await?.ok_or_else(|| {
                    DbSqlError::LogicalError(format!(
                        "failed to find channel key for packet key {} on next hop",
                        next_hop.to_peerid_str()
                    ))
                })?;

                let verified_ticket = match self
                    .begin_transaction()
                    .await?
                    .perform(|tx| {
                        Box::pin(async move {
                            let chain_data = myself.get_indexer_data(Some(tx)).await?;

                            let channel = myself
                                .get_channel_by_parties(Some(tx), &previous_hop_addr, &myself.me_onchain, true)
                                .await?
                                .ok_or_else(|| {
                                    DbSqlError::LogicalError(format!(
                                        "no channel found for previous hop address '{previous_hop_addr}'"
                                    ))
                                })?;

                            let remaining_balance = channel
                                .balance
                                .sub(myself.ticket_manager.unrealized_value((&channel).into()).await?);

                            let domain_separator = chain_data.channels_dst.ok_or_else(|| {
                                DbSqlError::LogicalError("failed to fetch the domain separator".into())
                            })?;

                            let ticket_price = chain_data
                                .ticket_price
                                .ok_or_else(|| DbSqlError::LogicalError("failed to fetch the ticket price".into()))?;

                            #[cfg(all(feature = "prometheus", not(test)))]
                            METRIC_INCOMING_WIN_PROB.observe(ticket.win_prob());

                            // Here also the signature on the ticket gets validated,
                            // so afterward we are sure the source of the `channel`
                            // (which is equal to `previous_hop_addr`) has issued this
                            // ticket.
                            let ticket = spawn_fifo_blocking(move || {
                                validate_unacknowledged_ticket(
                                    ticket,
                                    &channel,
                                    ticket_price,
                                    chain_data.minimum_incoming_ticket_winning_prob,
                                    remaining_balance,
                                    &domain_separator,
                                )
                            })
                            .await?;

                            myself.increment_outgoing_ticket_index(channel.get_id()).await?;

                            myself
                                .caches
                                .unacked_tickets
                                .insert(
                                    ack_challenge,
                                    PendingAcknowledgement::WaitingAsRelayer(
                                        ticket.clone().into_unacknowledged(own_key),
                                    ),
                                )
                                .await;

                            // Check that the calculated path position from the ticket matches the value from the packet header
                            let ticket_path_pos = ticket.get_path_position(ticket_price.amount())?;
                            if !ticket_path_pos.eq(&path_pos) {
                                return Err(DbSqlError::LogicalError(format!(
                                    "path position mismatch: from ticket {ticket_path_pos}, from packet {path_pos}"
                                )));
                            }

                            // Create the next ticket for the packet
                            let ticket_builder = if ticket_path_pos == 1 {
                                TicketBuilder::zero_hop().direction(&myself.me_onchain, &next_hop_addr)
                            } else {
                                // We currently take the maximum of the win prob on the ticket
                                // and the one configured on this node.
                                // Therefore, the winning probability can only increase on the path.
                                myself
                                    .create_multihop_ticket(
                                        Some(tx),
                                        myself.me_onchain,
                                        next_hop_addr,
                                        ticket_path_pos,
                                        outgoing_ticket_win_prob.max(ticket.win_prob()),
                                    )
                                    .await?
                            };

                            // TODO: benchmark this to confirm, offload a CPU intensive task off the async executor onto a parallelized thread pool
                            let ticket = spawn_fifo_blocking(move || {
                                ticket_builder
                                    .challenge(next_challenge.to_ethereum_challenge())
                                    .build_signed(&me, &domain_separator)
                            })
                            .await?;

                            // forward packet
                            Ok(ticket)
                        })
                    })
                    .await
                {
                    Ok(ticket) => Ok(ticket),
                    Err(DbSqlError::TicketValidationError(boxed_error)) => {
                        let (rejected_ticket, error) = *boxed_error;
                        let rejected_value = rejected_ticket.amount;
                        warn!("encountered validation error during forwarding for {rejected_ticket} with value: {rejected_value}: {error}");

                        self.mark_unsaved_ticket_rejected(&rejected_ticket).await.map_err(|e| {
                            DbSqlError::TicketValidationError(Box::new((
                                rejected_ticket.clone(),
                                format!("during validation error '{error}' update another error occurred: {e}"),
                            )))
                        })?;

                        Err(DbSqlError::TicketValidationError(Box::new((rejected_ticket, error))))
                    }
                    Err(e) => Err(e),
                }?;

                let offchain_keypair = pkt_keypair.clone();
                let (ack, payload) = spawn_fifo_blocking(move || {
                    let ack = Acknowledgement::new(ack_key, &offchain_keypair);

                    let mut payload = Vec::with_capacity(ChainPacketComponents::SIZE);
                    payload.extend_from_slice(packet.as_ref());

                    let ticket_bytes = verified_ticket.leak().into_encoded();
                    payload.extend_from_slice(ticket_bytes.as_ref());

                    (ack, payload.into_boxed_slice())
                })
                .await;

                Ok(TransportPacketWithChainData::Forwarded {
                    packet_tag,
                    previous_hop,
                    next_hop,
                    data: payload,
                    ack,
                })
            }
            ChainPacketComponents::Outgoing { .. } => {
                Err(DbSqlError::LogicalError("Cannot receive an outgoing packet".into()).into())
            }
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
        winning_prob: f64,
    ) -> crate::errors::Result<TicketBuilder> {
        let myself = self.clone();
        let (channel, ticket_price): (ChannelEntry, U256) = self
            .nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Ok::<_, DbSqlError>(
                        if let Some(channel) = myself
                            .get_channel_by_parties(Some(tx), &me_onchain, &destination, true)
                            .await?
                        {
                            let ticket_price = myself.get_indexer_data(Some(tx)).await?.ticket_price;

                            Some((
                                channel,
                                ticket_price
                                    .ok_or(DbSqlError::LogicalError("missing ticket price".into()))?
                                    .amount(),
                            ))
                        } else {
                            None
                        },
                    )
                })
            })
            .await?
            .ok_or(DbSqlError::LogicalError(format!(
                "channel to '{destination}' not found",
            )))?;

        let amount = Balance::new(
            ticket_price.div_f64(winning_prob).map_err(|e| {
                DbSqlError::LogicalError(format!(
                    "winning probability outside of the allowed interval (0.0, 1.0]: {e}"
                ))
            })? * U256::from(path_pos - 1),
            BalanceType::HOPR,
        );

        if channel.balance.lt(&amount) {
            return Err(DbSqlError::LogicalError(format!(
                "out of funds: {} with counterparty {destination} has balance {} < {amount}",
                channel.get_id(),
                channel.balance
            )));
        }

        let ticket_builder = TicketBuilder::default()
            .direction(&me_onchain, &destination)
            .balance(amount)
            .index(self.increment_outgoing_ticket_index(channel.get_id()).await?)
            .index_offset(1) // unaggregated always have index_offset == 1
            .win_prob(winning_prob)
            .channel_epoch(channel.channel_epoch.as_u32());

        Ok(ticket_builder)
    }
}
