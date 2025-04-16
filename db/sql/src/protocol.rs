use async_trait::async_trait;
use hopr_crypto_packet::prelude::*;
use hopr_crypto_packet::HoprPseudonym;
use hopr_crypto_types::prelude::*;
use hopr_db_api::errors::Result;
use hopr_db_api::protocol::{
    AckResult, HoprDbProtocolOperations, ResolvedAcknowledgement, TransportPacketWithChainData,
};
use hopr_db_api::resolver::HoprDbResolverOperations;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use std::ops::{Mul, Sub};
use tracing::{instrument, trace, warn};

use crate::channels::HoprDbChannelOperations;
use crate::db::HoprDb;
use crate::errors::DbSqlError;
use crate::info::HoprDbInfoOperations;
use crate::prelude::HoprDbTicketOperations;
use hopr_parallelize::cpu::spawn_fifo_blocking;
use hopr_path::errors::PathError;
use hopr_path::{TransportKeyResolver, ValidatedPath};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_INCOMING_WIN_PROB: hopr_metrics::SimpleHistogram =
        hopr_metrics::SimpleHistogram::new(
            "hopr_tickets_incoming_win_probability",
            "Observes the winning probabilities on incoming tickets",
            vec![0.0, 0.0001, 0.001, 0.01, 0.05, 0.1, 0.15, 0.25, 0.3, 0.5],
        ).unwrap();
}

impl HoprDb {
    /// Validates a ticket from a forwarded packet and replaces it with a new ticket for the next hop.
    async fn validate_and_replace_ticket(
        &self,
        mut fwd: HoprForwardedPacket,
        me: &ChainKeypair,
        outgoing_ticket_win_prob: f64,
        outgoing_ticket_price: Balance,
    ) -> std::result::Result<HoprForwardedPacket, DbSqlError> {
        let previous_hop_addr = self.resolve_chain_key(&fwd.previous_hop).await?.ok_or_else(|| {
            DbSqlError::LogicalError(format!(
                "failed to find channel key for packet key {} on previous hop",
                fwd.previous_hop.to_peerid_str()
            ))
        })?;

        let next_hop_addr = self.resolve_chain_key(&fwd.outgoing.next_hop).await?.ok_or_else(|| {
            DbSqlError::LogicalError(format!(
                "failed to find channel key for packet key {} on next hop",
                fwd.outgoing.next_hop.to_peerid_str()
            ))
        })?;

        let incoming_channel = self
            .get_channel_by_parties(None, &previous_hop_addr, &self.me_onchain, true)
            .await?
            .ok_or_else(|| {
                DbSqlError::LogicalError(format!(
                    "no channel found for previous hop address '{previous_hop_addr}'"
                ))
            })?;

        // Check: is the ticket in the packet really for the given channel?
        if !fwd.outgoing.ticket.channel_id.eq(&incoming_channel.get_id()) {
            return Err(DbSqlError::LogicalError("invalid ticket for channel".into()).into());
        }

        let chain_data = self.get_indexer_data(None).await?;
        let domain_separator = chain_data
            .channels_dst
            .ok_or_else(|| DbSqlError::LogicalError("failed to fetch the domain separator".into()))?;

        // The ticket price from the oracle times my node's position on the
        // path is the acceptable minimum
        let minimum_ticket_price = chain_data
            .ticket_price
            .ok_or_else(|| DbSqlError::LogicalError("failed to fetch the ticket price".into()))?
            .mul(U256::from(fwd.path_pos));

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INCOMING_WIN_PROB.observe(fwd.outgoing.ticket.win_prob());

        let remaining_balance = incoming_channel
            .balance
            .sub(self.ticket_manager.unrealized_value((&incoming_channel).into()).await?);

        // Here also the signature on the ticket gets validated,
        // so afterward we are sure the source of the `channel`
        // (which is equal to `previous_hop_addr`) has issued this
        // ticket.
        let verified_incoming_ticket = spawn_fifo_blocking(move || {
            validate_unacknowledged_ticket(
                fwd.outgoing.ticket,
                &incoming_channel,
                minimum_ticket_price,
                chain_data.minimum_incoming_ticket_winning_prob,
                remaining_balance,
                &domain_separator,
            )
        })
        .await?;

        // We currently take the maximum of the win prob from the incoming ticket
        // and the one configured on this node.
        // Therefore, the winning probability can only increase along the path.
        let outgoing_ticket_win_prob = outgoing_ticket_win_prob.max(verified_incoming_ticket.win_prob());

        // The ticket is now validated, let's place it into acknowledgement waiting queue
        self.caches
            .unacked_tickets
            .insert(
                fwd.outgoing.ack_challenge,
                PendingAcknowledgement::WaitingAsRelayer(verified_incoming_ticket.into_unacknowledged(fwd.own_key)),
            )
            .await;

        // NOTE: that the path position according to the ticket value
        // may no longer match the path position from the packet header,
        // because the price of the ticket may be set higher be the ticket
        // issuer.

        // Create the new ticket for the new packet
        let ticket_builder = if fwd.path_pos > 1 {
            // There must be a channel to the next node if it's not the final hop
            let outgoing_channel = self
                .get_channel_by_parties(None, &self.me_onchain, &next_hop_addr, true)
                .await?
                .ok_or(DbSqlError::LogicalError(format!(
                    "channel to '{next_hop_addr}' not found",
                )))?;

            self.create_multihop_ticket(
                outgoing_channel,
                self.me_onchain,
                next_hop_addr,
                fwd.path_pos,
                outgoing_ticket_win_prob,
                outgoing_ticket_price,
            )
            .await?
        } else {
            TicketBuilder::zero_hop().direction(&self.me_onchain, &next_hop_addr)
        };

        // Finally, replace the ticket in the outgoing packet with a new one
        fwd.outgoing.ticket = ticket_builder
            .challenge(fwd.next_challenge)
            .build_signed(me, &domain_separator)?
            .leak();

        Ok(fwd)
    }

    async fn validate_acknowledgement(
        &self,
        ack: &Acknowledgement,
    ) -> std::result::Result<ResolvedAcknowledgement, DbSqlError> {
        let pending_ack = self
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
            })?;

        match pending_ack {
            PendingAcknowledgement::WaitingAsSender => {
                trace!("received acknowledgement as sender: first relayer has processed the packet");
                Ok(ResolvedAcknowledgement::Sending(ack.ack_challenge()))
            }

            PendingAcknowledgement::WaitingAsRelayer(unacknowledged) => {
                let maybe_channel_with_issuer = self
                    .get_channel_by_parties(None, unacknowledged.ticket.verified_issuer(), &self.me_onchain, true)
                    .await?;

                // Issuer's channel must have an epoch matching with the unacknowledged ticket
                if maybe_channel_with_issuer
                    .is_some_and(|c| c.channel_epoch.as_u32() == unacknowledged.verified_ticket().channel_epoch)
                {
                    let domain_separator = self
                        .get_indexer_data(None)
                        .await?
                        .channels_dst
                        .ok_or_else(|| DbSqlError::LogicalError("domain separator missing".into()))?;

                    let myself = self.clone();
                    let ack = ack.clone();
                    hopr_parallelize::cpu::spawn_blocking(move || {
                        // This explicitly checks whether the acknowledgement
                        // solves the challenge on the ticket. It must be done before we
                        // check that the ticket is winning, which is a lengthy operation
                        // and should not be done for bogus unacknowledged tickets
                        let ack_ticket = unacknowledged.acknowledge(&ack.ack_key_share)?;

                        if ack_ticket.is_winning(&myself.chain_key, &domain_separator) {
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
                } else {
                    Err(DbSqlError::LogicalError(format!(
                        "no channel found for  address '{}' with a matching epoch",
                        unacknowledged.ticket.verified_issuer()
                    ))
                    .into())
                }
            }
        }
    }
}

#[async_trait]
impl HoprDbProtocolOperations for HoprDb {
    #[instrument(level = "trace", skip(self, ack))]
    async fn handle_acknowledgement(&self, ack: Acknowledgement) -> Result<AckResult> {
        let result = self.validate_acknowledgement(&ack).await?;
        match &result {
            ResolvedAcknowledgement::RelayingWin(ack_ticket) => {
                // If the ticket was a win, store it
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

    async fn get_network_winning_probability(&self) -> Result<f64> {
        Ok(self
            .get_indexer_data(None)
            .await
            .map(|data| data.minimum_incoming_ticket_winning_prob)?)
    }

    async fn get_network_ticket_price(&self) -> Result<Balance> {
        Ok(self.get_indexer_data(None).await.and_then(|data| {
            data.ticket_price
                .ok_or(DbSqlError::LogicalError("missing ticket price".into()))
        })?)
    }

    #[tracing::instrument(level = "trace", skip(self, data, forward_path, return_paths))]
    async fn to_send(
        &self,
        data: Box<[u8]>,
        pseudonym: Option<&HoprPseudonym>,
        forward_path: ValidatedPath,
        return_paths: Vec<ValidatedPath>,
        outgoing_ticket_win_prob: f64,
        outgoing_ticket_price: Balance,
    ) -> Result<TransportPacketWithChainData> {
        let next_peer = self.resolve_chain_key(&forward_path[0]).await?.ok_or_else(|| {
            DbSqlError::LogicalError(format!(
                "failed to find chain key for packet key {} on previous hop",
                forward_path[0].to_peerid_str()
            ))
        })?;

        let pseudonym = pseudonym.map(|p| *p).unwrap_or_else(|| HoprPseudonym::random());

        // Decide whether to create a multi-hop or a zero-hop ticket
        let next_ticket = if forward_path.length() > 1 {
            let channel = self
                .get_channel_by_parties(None, &self.me_onchain, &next_peer, true)
                .await?
                .ok_or(DbSqlError::LogicalError(format!("channel to '{next_peer}' not found")))?;

            self.create_multihop_ticket(
                channel,
                self.me_onchain,
                next_peer,
                forward_path.length() as u8,
                outgoing_ticket_win_prob,
                outgoing_ticket_price,
            )
            .await?
        } else {
            TicketBuilder::zero_hop().direction(&self.me_onchain, &next_peer)
        };

        let domain_separator = self
            .get_indexer_data(None)
            .await?
            .channels_dst
            .ok_or_else(|| DbSqlError::LogicalError("failed to fetch the domain separator".into()))?;

        // Construct the outgoing packet
        let myself = self.clone();
        let (packet, openers) = spawn_fifo_blocking(move || {
            HoprPacket::into_outgoing(
                &data,
                &pseudonym,
                PacketRouting::ForwardPath {
                    forward_path,
                    return_paths,
                },
                &myself.chain_key,
                next_ticket,
                &myself.caches.key_id_mapper,
                &domain_separator,
            )
            .map_err(|e| DbSqlError::LogicalError(format!("failed to construct chain components for a packet: {e}")))
        })
        .await?;

        // Store the reply openers under the given pseudonym
        openers
            .into_iter()
            .for_each(|p| self.caches.pseudonym_openers.insert(pseudonym, p));

        if let Some(out) = packet.try_as_outgoing() {
            self.caches
                .unacked_tickets
                .insert(out.ack_challenge, PendingAcknowledgement::WaitingAsSender)
                .await;

            let mut transport_payload = Vec::with_capacity(HoprPacket::SIZE);
            transport_payload.extend_from_slice(out.packet.as_ref());
            transport_payload.extend_from_slice(&out.ticket.into_encoded());

            Ok(TransportPacketWithChainData::Outgoing {
                next_hop: out.next_hop,
                ack_challenge: out.ack_challenge,
                data: transport_payload.into_boxed_slice(),
            })
        } else {
            Err(DbSqlError::LogicalError("must be an outgoing packet".into()).into())
        }
    }

    #[tracing::instrument(level = "trace", skip(self, data, pkt_keypair, sender), fields(sender = %sender))]
    async fn from_recv(
        &self,
        data: Box<[u8]>,
        pkt_keypair: &OffchainKeypair,
        sender: OffchainPublicKey,
        outgoing_ticket_win_prob: f64,
        outgoing_ticket_price: Balance,
    ) -> Result<TransportPacketWithChainData> {
        let offchain_keypair = pkt_keypair.clone();
        let myself = self.clone();

        let packet = spawn_fifo_blocking(move || {
            HoprPacket::from_incoming(&data, &offchain_keypair, sender, &myself.caches.key_id_mapper, |p| {
                myself.caches.pseudonym_openers.remove(p)
            })
            .map_err(|e| DbSqlError::LogicalError(format!("failed to construct an incoming packet: {e}")))
        })
        .await?;

        match packet {
            HoprPacket::Final(incoming) => Ok(TransportPacketWithChainData::Final {
                packet_tag: incoming.packet_tag,
                previous_hop: incoming.previous_hop,
                plain_text: incoming.plain_text,
                ack: Acknowledgement::new(incoming.ack_key, &pkt_keypair),
            }),
            HoprPacket::Forwarded(fwd) => {
                match self
                    .validate_and_replace_ticket(*fwd, &self.chain_key, outgoing_ticket_win_prob, outgoing_ticket_price)
                    .await
                {
                    Ok(fwd) => {
                        let mut payload = Vec::with_capacity(HoprPacket::SIZE);
                        payload.extend_from_slice(fwd.outgoing.packet.as_ref());
                        payload.extend_from_slice(&fwd.outgoing.ticket.into_encoded());

                        Ok(TransportPacketWithChainData::Forwarded {
                            packet_tag: fwd.packet_tag,
                            previous_hop: fwd.previous_hop,
                            next_hop: fwd.outgoing.next_hop,
                            data: payload.into_boxed_slice(),
                            ack: Acknowledgement::new(fwd.ack_key, &pkt_keypair),
                        })
                    }
                    Err(DbSqlError::TicketValidationError(boxed_error)) => {
                        let (rejected_ticket, error) = *boxed_error;
                        let rejected_value = rejected_ticket.amount;
                        warn!(?rejected_ticket, %rejected_value, erorr = ?error, "failure to validate during forwarding");

                        self.mark_unsaved_ticket_rejected(&rejected_ticket).await.map_err(|e| {
                            DbSqlError::TicketValidationError(Box::new((
                                rejected_ticket.clone(),
                                format!("during validation error '{error}' update another error occurred: {e}"),
                            )))
                        })?;

                        Err(DbSqlError::TicketValidationError(Box::new((rejected_ticket, error))).into())
                    }
                    Err(e) => Err(e.into()),
                }
            }
            HoprPacket::Outgoing(_) => Err(DbSqlError::LogicalError("cannot receive an outgoing packet".into()).into()),
        }
    }
}

impl HoprDb {
    async fn create_multihop_ticket(
        &self,
        channel: ChannelEntry,
        me_onchain: Address,
        destination: Address,
        current_path_pos: u8,
        winning_prob: f64,
        ticket_price: Balance,
    ) -> crate::errors::Result<TicketBuilder> {
        // The next ticket is worth: price * remaining hop count / winning probability
        let amount = Balance::new(
            ticket_price
                .amount()
                .mul(U256::from(current_path_pos - 1))
                .div_f64(winning_prob)
                .map_err(|e| {
                    DbSqlError::LogicalError(format!(
                        "winning probability outside of the allowed interval (0.0, 1.0]: {e}"
                    ))
                })?,
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

#[async_trait::async_trait]
impl TransportKeyResolver for HoprDb {
    async fn resolve_transport_key(
        &self,
        address: &Address,
    ) -> std::result::Result<Option<OffchainPublicKey>, PathError> {
        self.resolve_packet_key(address)
            .await
            .map_err(|e| PathError::InvalidPeer(format!("{address}: {e}")))
    }
}
