use std::ops::Mul;
use std::sync::Arc;
use libp2p_identity::PeerId;
use core_crypto::types::{Hash, PublicKey};
use core_db::traits::HoprCoreDbActions;
use core_types::acknowledgement::{Acknowledgement, PendingAcknowledgement, UnacknowledgedTicket};
use core_types::channels::{ChannelEntry, Ticket};
use utils_log::{error, info};
use utils_types::primitives::{Balance, BalanceType, U256};
use utils_types::traits::{PeerIdLike, ToHex};
use crate::errors::PacketError::{InvalidPacketState, OutOfFunds, TagReplay, TicketValidation};
use crate::packet::{Packet, PacketState};
use crate::errors::Result;

pub const PRICE_PER_PACKET: &str = "";
pub const INVERSE_TICKET_WIN_PROB: &str = "";

pub struct AcknowledgementInteraction {

}

impl AcknowledgementInteraction {
    pub fn send_acknowledgement(&self, acknowledgement: Acknowledgement, destination: PeerId) {
        // TODO: serialize and push to queue
        todo!()
    }
}

pub struct PacketInteraction<T: HoprCoreDbActions> {
    db: T,
    ack_interaction: Arc<AcknowledgementInteraction>,
    message_emitter: fn(&[u8]),
    check_unrealized_balance: bool,
    private_key: Box<[u8]>
}

impl<T: HoprCoreDbActions> PacketInteraction<T> {

    async fn check_packet_tag(&mut self, packet: &Packet) -> Result<()> {
        match packet.state() {
            PacketState::Final { packet_tag, .. } | PacketState::Forwarded { packet_tag, .. } => {
                if self.db.check_and_set_packet_tag(packet_tag)
                    .await? {
                    Ok(())
                } else {
                    Err(TagReplay)
                }
            }
            PacketState::Outgoing { .. } => Err(InvalidPacketState),
        }
    }

    async fn forward_packet_data(&self, counterparty: PeerId, serialized_packet: &[u8]) -> Result<()> {
        todo!("send message")
    }

    async fn validate_unacknowledged_ticket(&self,
                                            them: &PublicKey,
                                            min_ticket_amount: Balance,
                                            req_inverse_ticket_win_prob: U256,
                                            ticket: &Ticket,
                                            channel: &ChannelEntry,
                                            check_unrealized_balance: bool) -> Result<()> {
        todo!()

    }

    async fn bump_ticket_index(&self, channel_id: &Hash) -> Result<U256> {
        todo!()
    }

    async fn create_multihop_ticket(&mut self, destination: PublicKey, path_pos: u8) -> Result<Ticket> {
        let channel = self.db.get_channel_to(&destination).await?;
        let channel_id = channel.get_id();
        let current_index = self.bump_ticket_index(&channel_id).await?;
        let amount = Balance::new(U256::new(PRICE_PER_PACKET)
            .mul(U256::new(INVERSE_TICKET_WIN_PROB))
            .muln(path_pos as u32 - 1), BalanceType::HOPR);

        let outstanding_balance = self.db.get_pending_balance_to(&destination.to_address()).await?;
        let channel_balance = channel.balance.sub(&outstanding_balance);

        info!("balances {} - {} = {} should >= {} in channel open to {}",
            channel.balance.to_formatted_string(), outstanding_balance.to_formatted_string(),
            channel_balance.to_formatted_string(), amount.to_string(), channel.destination.to_hex(true)
        );

        if channel_balance.lt(&amount) {
            return Err(OutOfFunds(format!("{} with counterparty {}", channel_id.to_hex(), destination.to_hex(true))))
        }

        let ticket = Ticket::new(
            destination.to_address(),
            None,
            channel.ticket_epoch,
            current_index,
            amount,
        U256::from_inverse_probability(U256::new(INVERSE_TICKET_WIN_PROB))?,
            channel.channel_epoch,
            &self.private_key);

        self.db.mark_pending(&ticket).await?;
        info!("Creating ticket in channel {}. Ticket data: {}", channel_id.to_hex(), ticket.to_hex());
        // TODO: metric_ticketCounter.increment()

        Ok(ticket)
    }

    async fn handle_mixed_packet(&mut self, mut packet: Packet) -> Result<()> {
        self.check_packet_tag(&packet).await?;

        let next_ticket;
        let previous_peer;
        let next_peer;

        match packet.state() {
            PacketState::Outgoing { .. } => return Err(InvalidPacketState),

            PacketState::Final { plain_text, previous_hop, .. } => {
                // We're the destination of the packet, so emit the packet contents
                (self.message_emitter)(plain_text.as_ref());

                // And create acknowledgement
                let ack = packet.create_acknowledgement(&self.private_key).unwrap();
                self.ack_interaction.send_acknowledgement(ack, previous_hop.to_peerid());
                return Ok(())
            },

            PacketState::Forwarded { ack_challenge, previous_hop, own_key, next_hop, .. } => {
                let inverse_win_prob = U256::new(INVERSE_TICKET_WIN_PROB);

                // Validate the ticket first
                let channel = self.db.get_channel_from(&previous_hop).await?;
                if let Err(e) = self.validate_unacknowledged_ticket(
                    &previous_hop,
                    Balance::from_str(PRICE_PER_PACKET, BalanceType::HOPR),
                    inverse_win_prob,
                    &packet.ticket,
                    &channel,
                    self.check_unrealized_balance
                ).await {
                    // Mark as reject and passthrough the error
                    self.db.mark_rejected(&packet.ticket).await?;
                    return Err(e);
                }

                // TODO: await self.db.set_current_ticket_index(channel.get_id().hash(), packet_ticket.index);

                // Store the unacknowledged ticket
                self.db.store_pending_acknowledgment(
                    ack_challenge.clone(),
                    PendingAcknowledgement::WaitingAsRelayer(UnacknowledgedTicket::new(
                        packet.ticket.clone(),
                        own_key.clone(),
                        previous_hop.clone()
                    ))
                ).await?;

                let path_pos = packet.ticket.get_path_position(
                    U256::new(PRICE_PER_PACKET),
                    inverse_win_prob
                );

                // Create next ticket for the packet
                next_ticket = if path_pos == 1 {
                    Ticket::new_zero_hop(next_hop.clone(), None, &self.private_key)
                } else {
                    self.create_multihop_ticket(next_hop.clone(), path_pos).await?
                };
                previous_peer = previous_hop.to_peerid();
                next_peer = next_hop.to_peerid();
            }
        }

        // Transform the packet for forwarding using the next ticket
        packet.forward(&self.private_key, next_ticket)?;

        // Forward the packet to the next hop
        self.forward_packet_data(next_peer, &packet.to_bytes()).await?;

        // Acknowledge to the previous hop that we forwarded the packet
        let ack = packet.create_acknowledgement(&self.private_key).unwrap();
        self.ack_interaction.send_acknowledgement(ack, previous_peer);

        Ok(())
    }
}