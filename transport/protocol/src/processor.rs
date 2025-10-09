use std::ops::{Mul, Sub};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;
use futures::{Sink, SinkExt};
use moka::future::Cache;
use tracing::instrument;
use hopr_api::{
    chain::{ChainKeyOperations, ChainReadChannelOperations, ChainValues},
};
use hopr_api::chain::KeyIdMapper;
use hopr_api::db::{HoprDbTicketOperations, TicketSelector};
pub use hopr_crypto_packet::errors::PacketError;
use hopr_crypto_packet::errors::PacketError::TransportError;
use hopr_crypto_packet::prelude::{validate_unacknowledged_ticket, HoprForwardedPacket, HoprPacket, HoprSenderId, PacketRouting};
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::ResolvedTransportRouting;
use hopr_parallelize::cpu::spawn_fifo_blocking;
use hopr_path::{Path, ValidatedPath};
use hopr_primitive_types::prelude::*;
use hopr_protocol_app::prelude::*;

use crate::traits::{PacketUnwrapping, PacketWrapping, SurbStore};
use crate::errors::{IncomingPacketError, PacketProcessorError};
use crate::types::{AuxiliaryPacketInfo, IncomingAcknowledgementPacket, IncomingFinalPacket, IncomingForwardedPacket, IncomingPacket, OutgoingPacket};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_INCOMING_WIN_PROB: hopr_metrics::SimpleHistogram =
        hopr_metrics::SimpleHistogram::new(
            "hopr_tickets_incoming_win_probability",
            "Observes the winning probabilities on incoming tickets",
            vec![0.0, 0.0001, 0.001, 0.01, 0.05, 0.1, 0.15, 0.25, 0.3, 0.5],
        ).unwrap();
    pub(crate) static ref METRIC_RECEIVED_ACKS: hopr_metrics::MultiCounter = hopr_metrics::MultiCounter::new(
        "hopr_received_ack_count",
        "Number of received acknowledgements",
        &["valid"]
    )
    .unwrap();
    pub(crate) static ref METRIC_SENT_ACKS: hopr_metrics::SimpleCounter =
        hopr_metrics::SimpleCounter::new("hopr_sent_acks_count", "Number of sent message acknowledgements").unwrap();

    pub(crate) static ref METRIC_TICKETS_COUNT: hopr_metrics::MultiCounter =
        hopr_metrics::MultiCounter::new("hopr_tickets_count", "Number of winning tickets", &["type"]).unwrap();
}

const SLOW_OP: Duration = Duration::from_millis(150);

/// Implements protocol acknowledgement logic for msg packets
#[derive(Debug, Clone)]
pub struct PacketProcessor<Db, R, S> {
    db: Db,
    resolver: R,
    surb_store: S,
    unacked_tickets: Cache<HalfKeyChallenge, PendingAcknowledgement>,
    chain_key: ChainKeypair,
    packet_key: OffchainKeypair,
    cfg: PacketInteractionConfig,
}

#[async_trait::async_trait]
impl<Db, R, S> PacketWrapping for PacketProcessor<Db, R, S>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone,
    R: ChainReadChannelOperations + ChainKeyOperations + ChainValues + Send + Sync,
    S: SurbStore + Send + Sync + Clone,
{
    type Input = ApplicationDataOut;
    type Error = PacketProcessorError<Db::Error, R::Error>;

    #[tracing::instrument(level = "trace", skip(self, data), ret(Debug), err)]
    async fn send_data(
        &self,
        data: ApplicationDataOut,
        routing: ResolvedTransportRouting,
    ) -> Result<OutgoingPacket, Self::Error> {
        // Get necessary packet routing values
        let (next_peer, num_hops, pseudonym, routing) = match routing {
            ResolvedTransportRouting::Forward {
                pseudonym,
                forward_path,
                return_paths,
            } => (
                forward_path[0],
                forward_path.num_hops(),
                pseudonym,
                PacketRouting::ForwardPath {
                    forward_path,
                    return_paths,
                },
            ),
            ResolvedTransportRouting::Return(sender_id, surb) => {
                let next = self
                    .resolver
                    .key_id_mapper_ref()
                    .map_id_to_public(&surb.first_relayer)
                    .ok_or(PacketProcessorError::KeyNotFound)?;

                (
                    next,
                    surb.additional_data_receiver.proof_of_relay_values().chain_length() as usize,
                    sender_id.pseudonym(),
                    PacketRouting::Surb(sender_id.surb_id(), surb),
                )
            }
        };

        let next_peer = self
            .resolver
            .packet_key_to_chain_key(&next_peer)
            .await?
            .ok_or(PacketProcessorError::KeyNotFound)?;

        // Decide whether to create a multi-hop or a zero-hop ticket
        let next_ticket = if num_hops > 1 {
            let channel = self
                .resolver
                .channel_by_parties(self.chain_key.as_ref(), &next_peer)
                .await?
                .ok_or_else(|| PacketProcessorError::ChannelNotFound(*self.chain_key.as_ref(), next_peer.clone()))?;

            let (outgoing_ticket_win_prob, outgoing_ticket_price) =
                self.determine_network_config().await?;

            self.create_multihop_ticket(
                channel,
                next_peer,
                num_hops as u8,
                outgoing_ticket_win_prob,
                outgoing_ticket_price,
            )
                .await?
        } else {
            TicketBuilder::zero_hop().direction(self.chain_key.as_ref(), &next_peer)
        };

        let domain_separator = self
            .resolver
            .domain_separators()
            .await?
            .channel;

        // Construct the outgoing packet
        let chain_key = self.chain_key.clone();
        let mapper = self.resolver.key_id_mapper_ref().clone();
        let (packet, openers) = spawn_fifo_blocking(move || {
            HoprPacket::into_outgoing(
                data.data.to_bytes().as_ref(),
                &pseudonym,
                routing,
                &chain_key,
                next_ticket,
                &mapper,
                &domain_separator,
                data.packet_info.unwrap_or_default().signals_to_destination,
            )
        })
        .await?;

        // Store the reply openers under the given SenderId
        // This is a no-op for reply packets
        openers.into_iter().for_each(|(surb_id, opener)| {
            self.surb_store.insert_reply_opener(HoprSenderId::from_pseudonym_and_id(&pseudonym, surb_id), opener);
        });

        let out = packet
            .try_as_outgoing()
            .ok_or(PacketProcessorError::InvalidState("cannot send out packet that is not outgoing"))?;

        self.unacked_tickets
            .insert(out.ack_challenge, PendingAcknowledgement::WaitingAsSender)
            .await;

        let mut transport_payload = Vec::with_capacity(HoprPacket::SIZE);
        transport_payload.extend_from_slice(out.packet.as_ref());
        transport_payload.extend_from_slice(&out.ticket.into_encoded());

        Ok(OutgoingPacket {
            next_hop: out.next_hop,
            ack_challenge: out.ack_challenge,
            data: transport_payload.into_boxed_slice(),
        })
    }

    #[tracing::instrument(level = "trace", skip(self, ack, destination), ret(Debug), err)]
    async fn send_ack(&self, ack: Acknowledgement, destination: &OffchainPublicKey) -> Result<OutgoingPacket, Self::Error> {
        let next_peer = self
            .resolver
            .packet_key_to_chain_key(&destination)
            .await?
            .ok_or(PacketProcessorError::KeyNotFound)?;

        // No-ack packets are always sent as zero-hops with a random pseudonym
        let pseudonym = HoprPseudonym::random();
        let next_ticket = TicketBuilder::zero_hop().direction(self.chain_key.as_ref(), &next_peer);
        let domain_separator = self
            .resolver
            .domain_separators()
            .await?
            .channel;

        // Construct the outgoing packet
        let chain_key = self.chain_key.clone();
        let destination = *destination;
        let mapper = self.resolver.key_id_mapper_ref().clone();
        let (packet, _) = spawn_fifo_blocking(move || {
            HoprPacket::into_outgoing(
                ack.as_ref(),
                &pseudonym,
                PacketRouting::NoAck::<ValidatedPath>(destination),
                &chain_key,
                next_ticket,
                &mapper,
                &domain_separator,
                None, // NoAck messages currently do not have signals
            )
        })
        .await?;

        let out = packet
            .try_as_outgoing()
            .ok_or(PacketProcessorError::InvalidState("cannot send out packet that is not outgoing"))?;

        let mut transport_payload = Vec::with_capacity(HoprPacket::SIZE);
        transport_payload.extend_from_slice(out.packet.as_ref());
        transport_payload.extend_from_slice(&out.ticket.into_encoded());

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_SENT_ACKS.increment();

        Ok(OutgoingPacket {
            next_hop: out.next_hop,
            ack_challenge: out.ack_challenge,
            data: transport_payload.into_boxed_slice(),
        })
    }
}

pub enum ResolvedAcknowledgement {
    Sending(Box<VerifiedAcknowledgement>),
    RelayingWin(Box<AcknowledgedTicket>),
    RelayingLoss(Hash),
}

#[async_trait::async_trait]
impl<Db, R, S> PacketUnwrapping for PacketProcessor<Db, R, S>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone,
    R: ChainReadChannelOperations + ChainKeyOperations + ChainValues + Send + Sync,
    S: SurbStore + Send + Sync + Clone,
{
    type Packet = IncomingPacket;
    type Error = Db::Error;

    #[tracing::instrument(level = "trace", skip(self, data))]
    async fn recv_data(
        &self,
        previous_hop: OffchainPublicKey,
        data: Box<[u8]>,
    ) -> Result<Self::Packet, IncomingPacketError<Self::Error>> {
        let offchain_keypair = self.packet_key.clone();
        let surb_store = self.surb_store.clone();
        let start = std::time::Instant::now();
        let mapper = self.resolver.key_id_mapper_ref().clone();
        let packet = spawn_fifo_blocking(move || {
            HoprPacket::from_incoming(&data, &offchain_keypair, previous_hop, &mapper, |p| {
                surb_store.find_reply_opener(p)
            })
        })
        .await?;
        if start.elapsed() > SLOW_OP {
            tracing::warn!(
                elapsed = ?start.elapsed(),
                peer = previous_hop.to_peerid_str(),
                "from_incoming took too long",
            );
        }

        match packet {
            HoprPacket::Final(incoming) => {
                // Extract additional information from the packet that will be passed upwards
                let info = AuxiliaryPacketInfo {
                    packet_signals: incoming.signals,
                    num_surbs: incoming.surbs.len(),
                };

                // Store all incoming SURBs if any
                if !incoming.surbs.is_empty() {
                    self.surb_store.insert_surbs(incoming.sender, incoming.surbs).await;
                    tracing::trace!(pseudonym = %incoming.sender, num_surbs = info.num_surbs, "stored incoming surbs for pseudonym");
                }

                Ok(match incoming.ack_key {
                    None => {
                        // The contained payload represents an Acknowledgement
                        IncomingPacket::Acknowledgement(IncomingAcknowledgementPacket {
                            packet_tag: incoming.packet_tag,
                            previous_hop: incoming.previous_hop,
                            ack: incoming
                                .plain_text
                                .as_ref()
                                .try_into()?,
                        }.into())
                    }
                    Some(ack_key) => IncomingPacket::Final(IncomingFinalPacket {
                        packet_tag: incoming.packet_tag,
                        previous_hop: incoming.previous_hop,
                        sender: incoming.sender,
                        plain_text: incoming.plain_text,
                        ack_key,
                        info,
                    }.into()),
                })
            }
            HoprPacket::Forwarded(fwd) => {
                let start = std::time::Instant::now();
                let validation_res = self
                    .validate_and_replace_ticket(*fwd)
                    .await;
                if start.elapsed() > SLOW_OP {
                    tracing::warn!(
                        elapsed = ?start.elapsed(),
                        peer = previous_hop.to_peerid_str(),
                        "validate_and_replace_ticket took too long",
                    );
                }
                match validation_res {
                    Ok(fwd) => {
                        let mut payload = Vec::with_capacity(HoprPacket::SIZE);
                        payload.extend_from_slice(fwd.outgoing.packet.as_ref());
                        payload.extend_from_slice(&fwd.outgoing.ticket.into_encoded());

                        Ok(IncomingPacket::Forwarded(IncomingForwardedPacket {
                            packet_tag: fwd.packet_tag,
                            previous_hop: fwd.previous_hop,
                            next_hop: fwd.outgoing.next_hop,
                            data: payload.into_boxed_slice(),
                            ack_key: fwd.ack_key,
                        }.into()))
                    }
                    Err(PacketProcessorError::TicketValidationError(boxed_error)) => {
                        let (rejected_ticket, error) = *boxed_error;
                        let rejected_value = rejected_ticket.amount;
                        tracing::warn!(?rejected_ticket, %rejected_value, %error, "failure to validate during forwarding");

                        self.db.mark_unsaved_ticket_rejected(&rejected_ticket)
                            .await
                            .map_err(|e| {
                                PacketProcessorError::TicketValidationError(Box::new((
                                    rejected_ticket.clone(),
                                    format!("during validation error '{error}' update another error occurred: {e}"),
                                )))
                            })
                            .map_err(IncomingPacketError::ProcessingError)?;

                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_TICKETS_COUNT.increment(&["rejected"]);

                        Err(IncomingPacketError::ProcessingError(
                            PacketProcessorError::TicketValidationError(Box::new((rejected_ticket, error))),
                        ))
                    }
                    Err(e) => Err(IncomingPacketError::ProcessingError(e)),
                }
            }
            HoprPacket::Outgoing(_) => Err(IncomingPacketError::ProcessingError(PacketProcessorError::InvalidState)),
        }
    }

    async fn recv_ack(&self, peer: OffchainPublicKey, ack: Acknowledgement) -> Result<(), Self::Error> {
        let verified_ack = hopr_parallelize::cpu::spawn_blocking(move || {
            ack.verify(&peer)
        }).await?;

        let result = self.find_ticket_to_acknowledge(&verified_ack).await?;
        match &result {
            ResolvedAcknowledgement::RelayingWin(ack_ticket) => {
                // If the ticket was a win, store it
                self.db.insert_received_ticket(ack_ticket.as_ref().clone()).await?;

                #[cfg(all(feature = "prometheus", not(test)))]
                {
                    METRIC_RECEIVED_ACKS.increment(&["true"]);
                    METRIC_TICKETS_COUNT.increment(&["winning"]);
                }
            }
            ResolvedAcknowledgement::RelayingLoss(_channel) => {
                #[cfg(all(feature = "prometheus", not(test)))]
                {
                    METRIC_RECEIVED_ACKS.increment(&["true"]);
                    METRIC_TICKETS_COUNT.increment(&["losing"]);
                }
            }
            ResolvedAcknowledgement::Sending(_) => {
                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_RECEIVED_ACKS.increment(&["true"]);
            }
        };

        Ok(())
    }
}

impl<Db, R, S> PacketProcessor<Db, R, S>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone,
    R: ChainReadChannelOperations + ChainKeyOperations + ChainValues + Send + Sync,
    S: SurbStore + Send + Sync + Clone,
{
    /// Creates a new instance given the DB and configuration.
    pub fn new(
        db: Db,
        resolver: R,
        surb_store: S,
        chain_key: ChainKeypair,
        packet_key: OffchainKeypair,
        cfg: PacketInteractionConfig
    ) -> Self {
        #[cfg(all(feature = "prometheus", not(test)))]
        {
            lazy_static::initialize(&METRIC_RECEIVED_ACKS);
            lazy_static::initialize(&METRIC_SENT_ACKS);
            lazy_static::initialize(&METRIC_TICKETS_COUNT);
        }

        Self {
            db, resolver, surb_store,
            unacked_tickets: Cache::builder()
                .time_to_live(cfg.unack_ticket_timeout)
                .max_capacity(cfg.max_unack_tickets as u64)
                .build(),
            chain_key,
            packet_key,
            cfg
        }
    }

    #[tracing::instrument(level = "trace", skip(self, channel))]
    async fn create_multihop_ticket(
        &self,
        channel: ChannelEntry,
        destination: Address,
        current_path_pos: u8,
        winning_prob: WinningProbability,
        ticket_price: HoprBalance,
    ) -> Result<TicketBuilder, <Self as PacketWrapping>::Error> {
        // The next ticket is worth: price * remaining hop count / winning probability
        let amount = HoprBalance::from(
            ticket_price
                .amount()
                .mul(U256::from(current_path_pos - 1))
                .div_f64(winning_prob.into())?
        );

        if channel.balance.lt(&amount) {
            return Err(PacketProcessorError::OutOfFunds(destination, amount));
        }

        let ticket_builder = TicketBuilder::default()
            .direction(self.chain_key.as_ref(), &destination)
            .balance(amount)
            .index(self.db.increment_outgoing_ticket_index(channel.get_id()).await?)
            .index_offset(1) // unaggregated always have index_offset == 1
            .win_prob(winning_prob)
            .channel_epoch(channel.channel_epoch.as_u32());

        Ok(ticket_builder)
    }

    /// Validates a ticket from a forwarded packet and replaces it with a new ticket for the next hop.
    #[instrument(level = "trace", skip_all, err)]
    async fn validate_and_replace_ticket(
        &self,
        mut fwd: HoprForwardedPacket,
    ) -> Result<HoprForwardedPacket, <Self as PacketUnwrapping>::Error>
    where
        R: ChainReadChannelOperations + ChainKeyOperations + ChainValues + Send + Sync,
    {
        let previous_hop_addr = self
            .resolver
            .packet_key_to_chain_key(&fwd.previous_hop)
            .await?
            .ok_or(PacketProcessorError::KeyNotFound)?;

        let next_hop_addr = self
            .resolver
            .packet_key_to_chain_key(&fwd.outgoing.next_hop)
            .await?
            .ok_or(PacketProcessorError::KeyNotFound)?;

        let incoming_channel = self
            .resolver
            .channel_by_parties(&previous_hop_addr, self.chain_key.as_ref())
            .await?
            .ok_or_else(|| PacketProcessorError::ChannelNotFound(previous_hop_addr, *self.chain_key.as_ref()))?;

        // Check: is the ticket in the packet really for the given channel?
        if !fwd.outgoing.ticket.channel_id.eq(&incoming_channel.get_id()) {
            return Err(PacketProcessorError::InvalidState("ticket channel id does not match channel id of incoming packet"));
        }

        let domain_separator = self
            .resolver
            .domain_separators()
            .await?
            .channel;

        // The ticket price from the oracle times my node's position on the
        // path is the acceptable minimum
        let minimum_ticket_price = self
            .resolver
            .minimum_ticket_price()
            .await?
            .mul(U256::from(fwd.path_pos));

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_INCOMING_WIN_PROB.observe(fwd.outgoing.ticket.win_prob().as_f64());

        let remaining_balance = incoming_channel
            .balance
            .sub(self.db.unrealized_value(&incoming_channel).await?);

        // Here also the signature on the ticket gets validated,
        // so afterward we are sure the source of the `channel`
        // (which is equal to `previous_hop_addr`) has issued this
        // ticket.
        let start = std::time::Instant::now();
        let win_prob = self
            .resolver
            .minimum_incoming_ticket_win_prob()
            .await?;

        let verified_incoming_ticket = spawn_fifo_blocking(move || {
            validate_unacknowledged_ticket(
                fwd.outgoing.ticket,
                &incoming_channel,
                minimum_ticket_price,
                win_prob,
                remaining_balance,
                &domain_separator,
            )
        })
            .await?;

        if start.elapsed() > SLOW_OP {
            tracing::warn!(elapsed = ?start.elapsed(), "validate_unacknowledged_ticket took too long");
        }

        // The ticket is now validated, let's place it into the acknowledgement waiting queue
        self.unacked_tickets
            .insert(
                fwd.outgoing.ack_challenge,
                PendingAcknowledgement::WaitingAsRelayer(
                    verified_incoming_ticket.into_unacknowledged(fwd.own_key).into(),
                ),
            )
            .await;

        // NOTE: that the path position according to the ticket value
        // may no longer match the path position from the packet header,
        // because the ticket issuer may set the price of the ticket higher.

        // Create the new ticket for the new packet
        let ticket_builder = if fwd.path_pos > 1 {
            // There must be a channel to the next node if it's not the final hop
            let outgoing_channel = self
                .resolver
                .channel_by_parties(self.chain_key.as_ref(), &next_hop_addr)
                .await?
                .ok_or_else(|| PacketProcessorError::ChannelNotFound(*self.chain_key.as_ref(), next_hop_addr))?;

            let (outgoing_ticket_win_prob, outgoing_ticket_price) =
                self.determine_network_config()
                    .await
                    .map_err(IncomingPacketError::ProcessingError)?;

            // We currently take the maximum of the win prob from the incoming ticket
            // and the one configured on this node.
            // Therefore, the winning probability can only increase along the path.
            let outgoing_ticket_win_prob = outgoing_ticket_win_prob.max(&verified_incoming_ticket.win_prob());

            self.create_multihop_ticket(
                outgoing_channel,
                next_hop_addr,
                fwd.path_pos,
                outgoing_ticket_win_prob,
                outgoing_ticket_price,
            )
                .await?
        } else {
            TicketBuilder::zero_hop().direction(self.chain_key.as_ref(), &next_hop_addr)
        };

        // Finally, replace the ticket in the outgoing packet with a new one
        let ticket_builder = ticket_builder.eth_challenge(fwd.next_challenge);
        let me_on_chain = self.chain_key.clone();
        fwd.outgoing.ticket = spawn_fifo_blocking(move || ticket_builder.build_signed(&me_on_chain, &domain_separator))
            .await?
            .leak();

        Ok(fwd)
    }

    async fn determine_network_config(&self) -> Result<(WinningProbability, HoprBalance), PacketProcessorError<Db, R>> {
        // This operation hits the cache unless the new value is fetched for the first time
        // NOTE: as opposed to the winning probability, the ticket price does not have
        // a reasonable default, and therefore the operation fails
        let network_ticket_price = self.resolver
            .minimum_ticket_price()
            .await?;

        let outgoing_ticket_price = self.cfg.outgoing_ticket_price.unwrap_or(network_ticket_price);

        // This operation hits the cache unless the new value is fetched for the first time
        let network_win_prob = self
            .resolver
            .minimum_incoming_ticket_win_prob()
            .await
            .inspect_err(|error| tracing::error!(%error, "failed to determine current network winning probability"))
            .ok();

        // If no explicit winning probability is configured, use the network value
        // or 1 if the network value was not determined.
        // This code does not take the max from those, as it is the upper layer's responsibility
        // to ensure the configured value is not smaller than the network value.
        let outgoing_ticket_win_prob = self.cfg.outgoing_ticket_win_prob.or(network_win_prob).unwrap_or_default(); // Absolute default WinningProbability is 1.0

        Ok((outgoing_ticket_win_prob, outgoing_ticket_price))
    }

    #[instrument(level = "trace", skip(self, ack), err)]
    async fn find_ticket_to_acknowledge(
        &self,
        ack: &VerifiedAcknowledgement,
    ) -> Result<ResolvedAcknowledgement, <Self as PacketUnwrapping>::Error> {
        let ack_half_key = *ack.ack_key_share();
        let challenge = hopr_parallelize::cpu::spawn_blocking(move || ack_half_key.to_challenge()).await?;

        let pending_ack = self
            .unacked_tickets
            .remove(&challenge)
            .await
            .ok_or_else(|| PacketProcessorError::UnacknowledgedTicketNotFound(challenge))?;

        match pending_ack {
            PendingAcknowledgement::WaitingAsSender => {
                tracing::trace!("received acknowledgement as sender: first relayer has processed the packet");
                Ok(ResolvedAcknowledgement::Sending(Box::new(*ack)))
            }

            PendingAcknowledgement::WaitingAsRelayer(unacknowledged) => {
                let maybe_channel_with_issuer = self
                    .resolver
                    .channel_by_parties(unacknowledged.ticket.verified_issuer(), self.chain_key.as_ref())
                    .await?;

                // Issuer's channel must have an epoch matching with the unacknowledged ticket
                if maybe_channel_with_issuer
                    .is_some_and(|c| c.channel_epoch.as_u32() == unacknowledged.verified_ticket().channel_epoch)
                {
                    let domain_separator = self
                        .resolver
                        .domain_separators()
                        .await?
                        .channel;

                    let chain_key = self.chain_key.clone();
                    hopr_parallelize::cpu::spawn_blocking(move || {
                        // This explicitly checks whether the acknowledgement
                        // solves the challenge on the ticket. It must be done before we
                        // check that the ticket is winning, which is a lengthy operation
                        // and should not be done for bogus unacknowledged tickets
                        let ack_ticket = unacknowledged.acknowledge(&ack_half_key)?;

                        if ack_ticket.is_winning(&chain_key, &domain_separator) {
                            tracing::trace!("Found a winning ticket");
                            Ok(ResolvedAcknowledgement::RelayingWin(Box::new(ack_ticket)))
                        } else {
                            tracing::trace!("Found a losing ticket");
                            Ok(ResolvedAcknowledgement::RelayingLoss(
                                ack_ticket.ticket.verified_ticket().channel_id,
                            ))
                        }
                    })
                        .await
                } else {
                    Err(PacketProcessorError::ChannelNotFound(
                        *unacknowledged.ticket.verified_issuer(),
                        *self.chain_key.as_ref()
                    ))
                }
            }
        }
    }
}

/// Configuration parameters for the packet interaction.
#[derive(Clone, Debug, smart_default::SmartDefault)]
pub struct PacketInteractionConfig {
    pub outgoing_ticket_win_prob: Option<WinningProbability>,
    pub outgoing_ticket_price: Option<HoprBalance>,
    #[default(1_000_000_000)]
    pub max_unack_tickets: usize,
    #[default(Duration::from_secs(30))]
    pub unack_ticket_timeout: Duration,
}
