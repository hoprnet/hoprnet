use std::ops::Mul;

use bytes::{BufMut, Bytes, BytesMut};
use hopr_api::{
    chain::*,
    types::{crypto::prelude::*, internal::prelude::*, primitive::prelude::*},
};
use hopr_crypto_packet::prelude::*;
use hopr_utils::trace_timed;

use crate::{
    AuxiliaryPacketInfo, HoprCodecConfig, IncomingAcknowledgementPacket, IncomingFinalPacket, IncomingForwardedPacket,
    IncomingPacket, IncomingPacketError, PacketDecoder, SurbStore, errors::HoprProtocolError,
    replay_filter::TagReplayFilter,
};

/// Default [decoder](PacketDecoder) implementation for HOPR packets.
pub struct HoprDecoder<Chain, S, T> {
    chain_api: Chain,
    surb_store: std::sync::Arc<S>,
    ticket_factory: T,
    packet_key: OffchainKeypair,
    chain_key: ChainKeypair,
    channels_dst: Hash,
    cfg: HoprCodecConfig,
    replay_filter: TagReplayFilter,
}

impl<Chain, S, T> HoprDecoder<Chain, S, T>
where
    Chain: ChainReadChannelOperations + ChainKeyOperations + ChainReadTicketOperations + ChainValues + Send + Sync,
    S: SurbStore + Send + Sync,
    T: hopr_api::tickets::TicketFactory + Send + Sync,
{
    /// Creates a new instance of the decoder.
    pub fn new(
        (packet_key, chain_key): (OffchainKeypair, ChainKeypair),
        chain_api: Chain,
        surb_store: S,
        ticket_factory: T,
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
            ticket_factory,
            replay_filter: TagReplayFilter::new(&cfg.replay_filter),
        }
    }

    /// The replay filter, for tests and telemetry.
    pub fn replay_filter(&self) -> &TagReplayFilter {
        &self.replay_filter
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
            self.ticket_factory
                .remaining_incoming_channel_stake(&incoming_channel)
                .map_err(HoprProtocolError::ticket_factory)?
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
            self.ticket_factory
                .new_multihop_ticket(
                    &outgoing_channel,
                    fwd.path_pos.try_into().expect("path position is always > 1"),
                    outgoing_ticket_win_prob,
                    outgoing_ticket_price,
                )
                .map_err(HoprProtocolError::ticket_factory)?
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
    T: hopr_api::tickets::TicketFactory + Send + Sync,
{
    type Error = HoprProtocolError;

    #[tracing::instrument(skip(self, sender, data), level = "trace", fields(%sender))]
    fn decode(&self, sender: PeerId, data: Bytes) -> Result<IncomingPacket, IncomingPacketError<Self::Error>> {
        #[cfg(feature = "trace-timing")]
        let decode_start = std::time::Instant::now();
        tracing::trace!(data_len = data.len(), "decoding packet");

        // Phase 1: Peer ID conversion
        // Reads the key straight out of the peer id's multihash, so there is nothing to cache.
        let previous_hop = trace_timed!("peer_id_conversion complete", {
            OffchainPublicKey::from_peerid(&sender).map_err(|error| {
                tracing::error!(%sender, %error, "dropping packet - cannot convert peer id");
                IncomingPacketError::Undecodable(HoprProtocolError::InvalidSender)
            })
        })?;

        // Phase 2: Sphinx decoding, with the replay filter consulted on the packet tag.
        //
        // The tag is derived from the shared secret, so it becomes available right after the elliptic-curve
        // multiplication - before the header MAC, the payload permutation and the message parse. The filter is
        // asked from inside the decode, which is what lets a replay skip that remaining work. The check runs
        // first and records nothing; the commit runs once the header MAC has verified.
        //
        // Anybody can *compute* a tag, from the node's public key and an alpha of their choosing, so a tag is not
        // by itself proof of anything. What costs a well-formed packet is getting one *recorded*: only a packet
        // whose header MAC verifies reaches the commit. A packet that authenticates and then fails on our side is
        // recorded too, which costs one slot.

        // If the following operation fails, it means that the packet is not a valid Hopr packet,
        // and as such should not be acknowledged later.
        // Which of our SURBs this packet came back on, if any. Captured here because resolving the
        // opener is the only point that knows it; by the time the decoded packet exists the sender
        // id is gone.
        // The opener is *peeked* here and removed below only once the reply has been delivered, so that a copy
        // which loses a concurrent race cannot take the winner's decryption key with it.
        let replied_on_surb = std::cell::Cell::new(None);
        let consumed_opener = std::cell::Cell::new(None);
        let packet = trace_timed!("sphinx_decode complete", {
            HoprPacket::from_incoming_checked(
                &data,
                &self.packet_key,
                previous_hop,
                self.chain_api.key_id_mapper_ref(),
                |p| {
                    let opener = self.surb_store.find_reply_opener(p);
                    if opener.is_some() {
                        replied_on_surb.set(Some(p.surb_id()));
                        consumed_opener.set(Some(*p));
                    }
                    opener
                },
                // Checked once the secret exists, so a replay skips the header MAC, the payload permutation and
                // the message parse.
                |tag| {
                    let is_replay = self.replay_filter.contains(tag);
                    if is_replay {
                        tracing::debug!(%sender, "dropping packet - replay detected after the shared secret");
                    }
                    is_replay
                },
                // Committed once the header MAC and payload permutation have succeeded - the message/ticket parse
                // follows - so a forged alpha costs a well-formed packet rather than just the node's public key.
                // Reporting whether the tag was already present closes the race with a concurrent decode of the
                // same packet.
                |tag| {
                    let is_replay = self.replay_filter.commit_from(&previous_hop, tag);
                    if is_replay {
                        tracing::debug!(%sender, "dropping packet - replay detected while committing");
                    }
                    is_replay
                },
            )
        })
        .map_err(|error| match error {
            hopr_crypto_packet::errors::PacketError::PacketReplay => IncomingPacketError::Replay(previous_hop.into()),
            error => IncomingPacketError::undecodable(HoprProtocolError::from(error)),
        })?;

        // The reply decoded, so its opener has done its job. Removing it only now means a concurrent copy that
        // lost the tag race still found - and did not consume - the opener, so its own failure costs nothing.
        if let Some(sender_id) = consumed_opener.get() {
            self.surb_store.remove_reply_opener(&sender_id);
        }

        match packet {
            HoprPacket::Final(incoming) => {
                // Extract additional information from the packet that will be passed upwards
                let mut info = AuxiliaryPacketInfo {
                    packet_signals: incoming.signals,
                    num_surbs: incoming.surbs.len(),
                    num_evicted_surbs: 0,
                    // Taken before the SURBs move into the store. A malformed chain length of 0
                    // saturates to a direct return path, the reading that refuses rather than admits.
                    min_return_path_hops: incoming
                        .surbs
                        .iter()
                        .map(|(_, surb)| {
                            surb.additional_data_receiver
                                .proof_of_relay_values()
                                .chain_length()
                                .saturating_sub(1)
                        })
                        .min(),
                };

                // Store all incoming SURBs if any
                if !incoming.surbs.is_empty() {
                    let outcome = self.surb_store.insert_surbs(incoming.sender, incoming.surbs);
                    info.num_evicted_surbs = outcome.evicted;

                    // The tier split costs a second lookup and a lock, so take it only if the line below is
                    // going to be emitted.
                    let tiers = tracing::enabled!(tracing::Level::TRACE)
                        .then(|| self.surb_store.tier_lens(&incoming.sender))
                        .flatten();
                    tracing::trace!(
                        pseudonym = %incoming.sender,
                        num_surbs = info.num_surbs,
                        retained = outcome.retained,
                        retained_shares = tiers.map(|(shares, _)| shares),
                        retained_plain = tiers.map(|(_, plain)| plain),
                        packet_type = "final",
                        "stored incoming surbs for pseudonym"
                    );

                    // Debug rather than warn: the store summarises overflows once per report interval and per
                    // tier (`EvictionStats`), and that summary is what warns, on any lost PIX share and on
                    // dropped share-less SURBs only above a threshold. A sender that bypasses the balancer gate
                    // keeps the share-less tier full, so every one of its packets evicts SURBs although no
                    // share is lost, and a warning here would fire for each of them. This line is for chasing
                    // a single overflow, and `evicted_shares` is what it cost.
                    if outcome.evicted > 0 {
                        tracing::debug!(
                            pseudonym = %incoming.sender,
                            evicted = outcome.evicted,
                            evicted_shares = outcome.evicted_shares,
                            retained = outcome.retained,
                            "SURB buffer full; dropped SURBs to make room (share-less first; \
                             `evicted_shares` of them carried a PIX share, which is lost)"
                        );
                    }
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
                            replied_on_surb: replied_on_surb.get(),
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

                let mut payload = BytesMut::with_capacity(HoprPacket::SIZE);
                payload.put_slice(fwd.outgoing.packet.as_ref());
                payload.put_slice(&fwd.outgoing.ticket.into_encoded());

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
                        data: payload.freeze(),
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
