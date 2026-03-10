use std::{
    ops::{Mul, Sub},
    time::Duration,
};

use hopr_api::{
    chain::*,
    types::{crypto::prelude::*, internal::prelude::*, primitive::prelude::*},
};
use hopr_crypto_packet::prelude::*;
use hopr_platform::trace_timed;

use crate::{
    AuxiliaryPacketInfo, HoprCodecConfig, IncomingAcknowledgementPacket, IncomingFinalPacket, IncomingForwardedPacket,
    IncomingPacket, IncomingPacketError, PacketDecoder, SurbStore, TicketCreationError, TicketTracker,
    errors::HoprProtocolError, tbf::TagBloomFilter,
};

/// Default [decoder](PacketDecoder) implementation for HOPR packets.
pub struct HoprDecoder<Chain, S, T> {
    chain_api: Chain,
    surb_store: std::sync::Arc<S>,
    tracker: T,
    packet_key: OffchainKeypair,
    chain_key: ChainKeypair,
    channels_dst: Hash,
    cfg: HoprCodecConfig,
    tbf: parking_lot::Mutex<TagBloomFilter>,
    peer_id_cache: moka::sync::Cache<PeerId, OffchainPublicKey>,
}

impl<Chain, S, T> HoprDecoder<Chain, S, T>
where
    Chain: ChainReadChannelOperations + ChainKeyOperations + ChainReadTicketOperations + ChainValues + Send + Sync,
    S: SurbStore + Send + Sync,
    T: TicketTracker + Send + Sync,
{
    /// Creates a new instance of the decoder.
    pub fn new(
        (packet_key, chain_key): (OffchainKeypair, ChainKeypair),
        chain_api: Chain,
        surb_store: S,
        tracker: T,
        channels_dst: Hash,
        cfg: HoprCodecConfig,
    ) -> Self {
        Self {
            chain_api,
            surb_store: std::sync::Arc::new(surb_store),
            packet_key,
            chain_key,
            channels_dst,
            cfg,
            tracker,
            tbf: parking_lot::Mutex::new(Default::default()),
            peer_id_cache: moka::sync::Cache::builder()
                .time_to_idle(Duration::from_secs(600))
                .max_capacity(100_000)
                .build(),
        }
    }

    #[tracing::instrument(skip(self, fwd), level = "debug", fields(path_pos = fwd.path_pos))]
    fn validate_and_replace_ticket(
        &self,
        mut fwd: HoprForwardedPacket,
    ) -> Result<(HoprForwardedPacket, UnacknowledgedTicket), HoprProtocolError> {
        let previous_hop_addr = trace_timed!("previous_hop_addr lookup", {
            self.chain_api
                .packet_key_to_chain_key(&fwd.previous_hop)
                .map_err(HoprProtocolError::resolver)?
                .ok_or(HoprProtocolError::KeyNotFound)?
        });

        let next_hop_addr = trace_timed!("next_hop_addr lookup", {
            self.chain_api
                .packet_key_to_chain_key(&fwd.outgoing.next_hop)
                .map_err(HoprProtocolError::resolver)?
                .ok_or(HoprProtocolError::KeyNotFound)?
        });

        let incoming_channel = trace_timed!("incoming_channel lookup", {
            self.chain_api
                .channel_by_parties(&previous_hop_addr, self.chain_key.as_ref())
                .map_err(HoprProtocolError::resolver)?
                .ok_or_else(|| HoprProtocolError::ChannelNotFound(previous_hop_addr, *self.chain_key.as_ref()))?
        });

        // The ticket price from the oracle times my node's position on the
        // path is the acceptable minimum
        let (win_prob, minimum_ticket_price) = self
            .chain_api
            .incoming_ticket_values()
            .map_err(HoprProtocolError::resolver)?;

        let minimum_ticket_price = minimum_ticket_price
            .mul(U256::from(fwd.path_pos))
            .max(self.cfg.min_incoming_ticket_price.unwrap_or_default());

        let remaining_balance = trace_timed!("unrealized_balance lookup", {
            incoming_channel.balance.sub(
                self.tracker
                    .incoming_channel_unrealized_balance(
                        incoming_channel.get_id(),
                        incoming_channel.channel_epoch,
                        incoming_channel.ticket_index,
                    )
                    .map_err(HoprProtocolError::ticket_tracker)?,
            )
        });

        // Here also the signature on the ticket gets validated,
        // so afterward we are sure the source of the `channel`
        // (which is equal to `previous_hop_addr`) has issued this
        // ticket.

        let verified_incoming_ticket = trace_timed!("ticket_signature_verification", {
            validate_unacknowledged_ticket(
                fwd.outgoing.ticket,
                &incoming_channel,
                minimum_ticket_price,
                win_prob,
                remaining_balance,
                &self.channels_dst,
            )
        })?;

        // The ticket is now validated:
        tracing::trace!(%verified_incoming_ticket, "successfully verified incoming ticket");

        // NOTE: that the path position according to the ticket value
        // may no longer match the path position from the packet header,
        // because the ticket issuer may set the price of the ticket higher.

        // Create the new ticket for the new packet
        let ticket_builder = if fwd.path_pos > 1 {
            // There must be a channel to the next node if it's not the final hop.
            // If the channel does not exist, the ticket we extracted before cannot be saved,
            // as there would be no way to acknowledge it without the channel.
            let outgoing_channel = self
                .chain_api
                .channel_by_parties(self.chain_key.as_ref(), &next_hop_addr)
                .map_err(HoprProtocolError::resolver)?
                .ok_or_else(|| HoprProtocolError::ChannelNotFound(*self.chain_key.as_ref(), next_hop_addr))?;

            let (outgoing_ticket_win_prob, outgoing_ticket_price) = self
                .chain_api
                .outgoing_ticket_values(self.cfg.outgoing_win_prob, self.cfg.outgoing_ticket_price)
                .map_err(HoprProtocolError::resolver)?;

            // We currently take the maximum of the win prob from the incoming ticket
            // and the one configured on this node.
            // Therefore, the winning probability can only increase along the path.
            let outgoing_ticket_win_prob = outgoing_ticket_win_prob.max(&verified_incoming_ticket.win_prob());

            // The following operation fails if there's not enough balance on the channel to the next hop.
            // Again, in this case, we cannot save the ticket we previously extracted because there is no way it gets
            // acknowledged without enough balance.
            self.tracker
                .create_multihop_ticket(
                    &outgoing_channel,
                    fwd.path_pos,
                    outgoing_ticket_win_prob,
                    outgoing_ticket_price,
                )
                .map_err(|e| match e {
                    TicketCreationError::OutOfFunds(id, a) => HoprProtocolError::OutOfFunds(id, a),
                    e => HoprProtocolError::TicketTrackerError(e.into()),
                })?
        } else {
            TicketBuilder::zero_hop().counterparty(next_hop_addr)
        };

        // Finally, replace the ticket in the outgoing packet with a new one
        let ticket_builder = ticket_builder.eth_challenge(fwd.next_challenge);
        fwd.outgoing.ticket = trace_timed!("ticket_signing", {
            ticket_builder.build_signed(&self.chain_key, &self.channels_dst)?.leak()
        });

        let unack_ticket = verified_incoming_ticket.into_unacknowledged(fwd.own_key);
        Ok((fwd, unack_ticket))
    }
}

impl<Chain, S, T> PacketDecoder for HoprDecoder<Chain, S, T>
where
    Chain: ChainReadChannelOperations + ChainKeyOperations + ChainReadTicketOperations + ChainValues + Send + Sync,
    S: SurbStore + Send + Sync + 'static,
    T: TicketTracker + Send + Sync,
{
    type Error = HoprProtocolError;

    #[tracing::instrument(skip(self, sender, data), level = "trace", fields(%sender))]
    fn decode(&self, sender: PeerId, data: Box<[u8]>) -> Result<IncomingPacket, IncomingPacketError<Self::Error>> {
        #[cfg(feature = "trace-timing")]
        let decode_start = std::time::Instant::now();
        tracing::trace!(data_len = data.len(), "decoding packet");

        // Phase 1: Peer ID conversion
        // Try to retrieve the peer's public key from the cache or compute it if it does not exist yet.
        // The async block ensures the Rayon task is only submitted on cache miss.
        let previous_hop = trace_timed!("peer_id_conversion complete", {
            match self
                .peer_id_cache
                .try_get_with_by_ref(&sender, || OffchainPublicKey::from_peerid(&sender))
            {
                Ok(peer) => Ok(peer),
                Err(error) => {
                    tracing::error!(%sender, %error, "dropping packet - cannot convert peer id");
                    Err(IncomingPacketError::Undecodable(HoprProtocolError::InvalidSender))
                }
            }
        })?;

        // Phase 2: Sphinx packet decoding

        // If the following operation fails, it means that the packet is not a valid Hopr packet,
        // and as such should not be acknowledged later.
        let packet = trace_timed!("sphinx_decode complete", {
            HoprPacket::from_incoming(
                &data,
                &self.packet_key,
                previous_hop,
                self.chain_api.key_id_mapper_ref(),
                |p| self.surb_store.find_reply_opener(p),
            )
        })
        .map_err(IncomingPacketError::undecodable)?;

        // This is checked on both Final and Forwarded packets,
        // Outgoing packets are not allowed to pass and are later reported as invalid state.
        if let Some(tag) = packet.packet_tag() {
            // This operation has run-time of ~10 nanoseconds,
            // and therefore does not need to be invoked via spawn_blocking
            if self.tbf.lock().check_and_set(tag) {
                return Err(IncomingPacketError::ProcessingError(
                    previous_hop.into(),
                    HoprProtocolError::Replay,
                ));
            }
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
                    self.surb_store.insert_surbs(incoming.sender, incoming.surbs);
                    tracing::trace!(pseudonym = %incoming.sender, num_surbs = info.num_surbs, packet_type = "final", "stored incoming surbs for pseudonym");
                }

                let result = match incoming.ack_key {
                    None => {
                        if incoming.plain_text.len() < size_of::<u16>() {
                            return Err(IncomingPacketError::Undecodable(
                                GeneralError::ParseError("invalid acknowledgement packet size".into()).into(),
                            ));
                        }

                        let num_acks =
                            u16::from_be_bytes(incoming.plain_text[..size_of::<u16>()].try_into().map_err(|_| {
                                IncomingPacketError::Undecodable(
                                    GeneralError::ParseError("invalid num acks".into()).into(),
                                )
                            })?);

                        if incoming.plain_text.len() < size_of::<u16>() + (num_acks as usize) * Acknowledgement::SIZE {
                            return Err(IncomingPacketError::Undecodable(
                                GeneralError::ParseError("invalid number of acknowledgements in packet".into()).into(),
                            ));
                        }
                        tracing::trace!(num_acks, packet_type = "final", "received acknowledgement packet");

                        // The contained payload represents an Acknowledgement
                        IncomingPacket::Acknowledgement(
                            IncomingAcknowledgementPacket {
                                packet_tag: incoming.packet_tag,
                                previous_hop: incoming.previous_hop,
                                received_acks: incoming.plain_text
                                    [size_of::<u16>()..size_of::<u16>() + num_acks as usize * Acknowledgement::SIZE]
                                    .chunks_exact(Acknowledgement::SIZE)
                                    .map(Acknowledgement::try_from)
                                    .collect::<Result<Vec<_>, _>>()
                                    .map_err(|e: GeneralError| IncomingPacketError::Undecodable(e.into()))?,
                            }
                            .into(),
                        )
                    }
                    Some(ack_key) => IncomingPacket::Final(
                        IncomingFinalPacket {
                            packet_tag: incoming.packet_tag,
                            previous_hop: incoming.previous_hop,
                            sender: incoming.sender,
                            plain_text: incoming.plain_text,
                            ack_key,
                            info,
                        }
                        .into(),
                    ),
                };
                #[cfg(feature = "trace-timing")]
                tracing::trace!(
                    total_ms = decode_start.elapsed().as_millis() as u64,
                    packet_type = "final",
                    "decode complete"
                );
                Ok(result)
            }
            HoprPacket::Forwarded(fwd) => {
                // Phase 3: Ticket validation and replacement for forwarded packets
                // Transform the ticket so it can be sent to the next hop
                let (fwd, verified_unack_ticket) = trace_timed!("ticket_validation complete", {
                    self.validate_and_replace_ticket(*fwd).map_err(|error| match error {
                        // Distinguish ticket validation errors so that they can get extra treatment later
                        HoprProtocolError::TicketValidationError(e) => {
                            IncomingPacketError::InvalidTicket(previous_hop.into(), e)
                        }
                        e => IncomingPacketError::ProcessingError(previous_hop.into(), e),
                    })?
                });

                let mut payload = Vec::with_capacity(HoprPacket::SIZE);
                payload.extend_from_slice(fwd.outgoing.packet.as_ref());
                payload.extend_from_slice(&fwd.outgoing.ticket.into_encoded());

                #[cfg(feature = "trace-timing")]
                tracing::trace!(
                    total_ms = decode_start.elapsed().as_millis() as u64,
                    packet_type = "forwarded",
                    "decode complete"
                );
                Ok(IncomingPacket::Forwarded(
                    IncomingForwardedPacket {
                        packet_tag: fwd.packet_tag,
                        previous_hop: fwd.previous_hop,
                        next_hop: fwd.outgoing.next_hop,
                        data: payload.into_boxed_slice(),
                        ack_challenge: fwd.outgoing.ack_challenge,
                        received_ticket: verified_unack_ticket,
                        ack_key_prev_hop: fwd.ack_key,
                    }
                    .into(),
                ))
            }
            HoprPacket::Outgoing(_) => {
                #[cfg(feature = "trace-timing")]
                tracing::trace!(
                    total_ms = decode_start.elapsed().as_millis() as u64,
                    packet_type = "outgoing",
                    "decode complete"
                );
                Err(IncomingPacketError::ProcessingError(
                    previous_hop.into(),
                    HoprProtocolError::InvalidState("cannot be outgoing packet"),
                ))
            }
        }
    }
}
