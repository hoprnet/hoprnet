use std::ops::{Mul, Sub};

use hopr_api::chain::*;
use hopr_crypto_packet::{errors::TicketValidationError, prelude::*};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::{
    AuxiliaryPacketInfo, IncomingAcknowledgementPacket, IncomingFinalPacket, IncomingForwardedPacket, IncomingPacket,
    IncomingPacketError, PacketDecoder, SurbStore, TicketTracker, errors::HoprProtocolError, tbf::TagBloomFilter,
};

#[derive(Clone, Debug, smart_default::SmartDefault)]
pub struct HoprDecoderConfig {
    pub outgoing_ticket_price: Option<HoprBalance>,
    #[default(Some(WinningProbability::ALWAYS))]
    pub outgoing_win_prob: Option<WinningProbability>,
    pub channels_dst: Hash,
}

pub struct HoprDecoder<R, S, T> {
    provider: R,
    surb_store: std::sync::Arc<S>,
    tracker: T,
    packet_key: OffchainKeypair,
    chain_key: ChainKeypair,
    cfg: HoprDecoderConfig,
    tbf: parking_lot::Mutex<TagBloomFilter>,
}

impl<R, S, T> HoprDecoder<R, S, T>
where
    R: ChainReadChannelOperations + ChainKeyOperations + ChainValues + Send + Sync,
    S: SurbStore + Send + Sync,
    T: TicketTracker + Send + Sync,
{
    pub fn new(
        provider: R,
        surb_store: S,
        tracker: T,
        (packet_key, chain_key): (OffchainKeypair, ChainKeypair),
        cfg: HoprDecoderConfig,
    ) -> Self {
        Self {
            provider,
            surb_store: std::sync::Arc::new(surb_store),
            packet_key,
            chain_key,
            cfg,
            tracker,
            tbf: parking_lot::Mutex::new(TagBloomFilter::default()),
        }
    }

    async fn validate_and_replace_ticket(
        &self,
        mut fwd: HoprForwardedPacket,
    ) -> Result<(HoprForwardedPacket, UnacknowledgedTicket), HoprProtocolError> {
        let previous_hop_addr = self
            .provider
            .packet_key_to_chain_key(&fwd.previous_hop)
            .await
            .map_err(|e| HoprProtocolError::ResolverError(e.into()))?
            .ok_or(HoprProtocolError::KeyNotFound)?;

        let next_hop_addr = self
            .provider
            .packet_key_to_chain_key(&fwd.outgoing.next_hop)
            .await
            .map_err(|e| HoprProtocolError::ResolverError(e.into()))?
            .ok_or(HoprProtocolError::KeyNotFound)?;

        let incoming_channel = self
            .provider
            .channel_by_parties(&previous_hop_addr, self.chain_key.as_ref())
            .await
            .map_err(|e| HoprProtocolError::ResolverError(e.into()))?
            .ok_or_else(|| HoprProtocolError::ChannelNotFound(previous_hop_addr, *self.chain_key.as_ref()))?;

        // Check: is the ticket in the packet really for the given channel?
        if !fwd.outgoing.ticket.channel_id.eq(incoming_channel.get_id()) {
            return Err(TicketValidationError {
                reason: "channel id on the ticket does not match the channel".into(),
                ticket: Box::new(fwd.outgoing.ticket),
            }
            .into());
        }

        // The ticket price from the oracle times my node's position on the
        // path is the acceptable minimum
        let minimum_ticket_price = self
            .provider
            .minimum_ticket_price()
            .await
            .map_err(|e| HoprProtocolError::ResolverError(e.into()))?
            .mul(U256::from(fwd.path_pos));

        let remaining_balance = incoming_channel.balance.sub(
            self.tracker
                .incoming_channel_unrealized_balance(incoming_channel.get_id()),
        );

        // Here also the signature on the ticket gets validated,
        // so afterward we are sure the source of the `channel`
        // (which is equal to `previous_hop_addr`) has issued this
        // ticket.
        let win_prob = self
            .provider
            .minimum_incoming_ticket_win_prob()
            .await
            .map_err(|e| HoprProtocolError::ResolverError(e.into()))?;
        let domain_separator = self.cfg.channels_dst;

        let verified_incoming_ticket = hopr_parallelize::cpu::spawn_fifo_blocking(move || {
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
                .provider
                .channel_by_parties(self.chain_key.as_ref(), &next_hop_addr)
                .await
                .map_err(|e| HoprProtocolError::ResolverError(e.into()))?
                .ok_or_else(|| HoprProtocolError::ChannelNotFound(*self.chain_key.as_ref(), next_hop_addr))?;

            let (outgoing_ticket_win_prob, outgoing_ticket_price) = self
                .provider
                .outgoing_ticket_values(self.cfg.outgoing_win_prob, self.cfg.outgoing_ticket_price)
                .await
                .map_err(|e| HoprProtocolError::ResolverError(e.into()))?;

            // We currently take the maximum of the win prob from the incoming ticket
            // and the one configured on this node.
            // Therefore, the winning probability can only increase along the path.
            let outgoing_ticket_win_prob = outgoing_ticket_win_prob.max(&verified_incoming_ticket.win_prob());

            // The following operation fails if there's not enough balance on the channel to the next hop.
            // Again, in this case, we cannot save the ticket we previously extracted because there is no way it gets
            // acknowledged without enough balance.
            self.tracker.create_multihop_ticket(
                &outgoing_channel,
                fwd.path_pos,
                outgoing_ticket_win_prob,
                outgoing_ticket_price,
            )?
        } else {
            TicketBuilder::zero_hop().direction(self.chain_key.as_ref(), &next_hop_addr)
        };

        // Finally, replace the ticket in the outgoing packet with a new one
        let ticket_builder = ticket_builder.eth_challenge(fwd.next_challenge);
        let me_on_chain = self.chain_key.clone();
        fwd.outgoing.ticket = hopr_parallelize::cpu::spawn_fifo_blocking(move || {
            ticket_builder.build_signed(&me_on_chain, &domain_separator)
        })
        .await?
        .leak();

        let unack_ticket = verified_incoming_ticket.into_unacknowledged(fwd.own_key);
        Ok((fwd, unack_ticket))
    }
}

#[async_trait::async_trait]
impl<R, S, T> PacketDecoder for HoprDecoder<R, S, T>
where
    R: ChainReadChannelOperations + ChainKeyOperations + ChainValues + Send + Sync,
    S: SurbStore + Send + Sync + 'static,
    T: TicketTracker + Send + Sync,
{
    type Error = HoprProtocolError;

    async fn decode(
        &self,
        previous_hop: OffchainPublicKey,
        data: Box<[u8]>,
    ) -> Result<IncomingPacket, IncomingPacketError<Self::Error>> {
        let offchain_keypair = self.packet_key.clone();
        let surb_store = self.surb_store.clone();
        let mapper = self.provider.key_id_mapper_ref().clone();

        // If the following operation fails, it means that the packet is not a valid Hopr packet,
        // and as such should not be acknowledged later.
        let packet = hopr_parallelize::cpu::spawn_fifo_blocking(move || {
            HoprPacket::from_incoming(&data, &offchain_keypair, previous_hop, &mapper, |p| {
                surb_store.find_reply_opener(p)
            })
        })
        .await
        .map_err(|e| IncomingPacketError::Undecodable(e.into()))?;

        // This is checked on both Final and Forwarded packets,
        // Outgoing packets are not allowed to pass and are later reported as invalid state.
        if let Some(tag) = packet.packet_tag() {
            // This operation has run-time of ~10 nanoseconds,
            // and therefore does not need to be invoked via spawn_blocking
            if self.tbf.lock().check_and_set(tag) {
                return Err(IncomingPacketError::ProcessingError(HoprProtocolError::Replay));
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
                    self.surb_store.insert_surbs(incoming.sender, incoming.surbs).await;
                    tracing::trace!(pseudonym = %incoming.sender, num_surbs = info.num_surbs, "stored incoming surbs for pseudonym");
                }

                Ok(match incoming.ack_key {
                    None => {
                        // The contained payload represents an Acknowledgement
                        IncomingPacket::Acknowledgement(
                            IncomingAcknowledgementPacket {
                                packet_tag: incoming.packet_tag,
                                previous_hop: incoming.previous_hop,
                                ack: incoming
                                    .plain_text
                                    .as_ref()
                                    .try_into()
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
                })
            }
            HoprPacket::Forwarded(fwd) => {
                // Transform the ticket so it can be sent to the next hop
                let (fwd, verified_unack_ticket) =
                    self.validate_and_replace_ticket(*fwd)
                        .await
                        .map_err(|error| match error {
                            // Distinguish ticket validation errors so that they can get extra treatment later
                            HoprProtocolError::TicketValidationError(e) => IncomingPacketError::InvalidTicket(e),
                            e => IncomingPacketError::ProcessingError(e),
                        })?;

                let mut payload = Vec::with_capacity(HoprPacket::SIZE);
                payload.extend_from_slice(fwd.outgoing.packet.as_ref());
                payload.extend_from_slice(&fwd.outgoing.ticket.into_encoded());

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
            HoprPacket::Outgoing(_) => Err(IncomingPacketError::ProcessingError(HoprProtocolError::InvalidState(
                "cannot be outgoing packet",
            ))),
        }
    }
}
