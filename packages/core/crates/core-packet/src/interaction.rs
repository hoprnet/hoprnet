use crate::errors::PacketError::{
    AcknowledgementValidation, InvalidPacketState, OutOfFunds, TagReplay, TicketValidation,
};
use crate::errors::Result;
use crate::packet::{Packet, PacketState};
use crate::path::Path;
use core_crypto::types::{HalfKeyChallenge, Hash, PublicKey};
use core_db::traits::HoprCoreDbActions;
use core_types::acknowledgement::{
    AcknowledgedTicket, Acknowledgement, AcknowledgementChallenge, PendingAcknowledgement, UnacknowledgedTicket,
};
use core_types::channels::{ChannelEntry, ChannelStatus, Ticket};
use libp2p_identity::PeerId;
use std::cell::RefCell;
use std::ops::Mul;
use std::sync::Arc;
use utils_log::{debug, info};
use utils_metrics::metrics::SimpleCounter;
use utils_types::primitives::{Balance, BalanceType, U256};
use utils_types::traits::{BinarySerializable, PeerIdLike, ToHex};

pub const PRICE_PER_PACKET: &str = "10000000000000000";
pub const INVERSE_TICKET_WIN_PROB: &str = "1";
const PREIMAGE_PLACE_HOLDER: [u8; Hash::SIZE] = [0xffu8; Hash::SIZE];

struct AckMetrics {
    received_successful_acks: SimpleCounter,
    received_failed_acks: SimpleCounter,
    sent_acks: SimpleCounter,
    acked_tickets: SimpleCounter,
}

impl Default for AckMetrics {
    fn default() -> Self {
        Self {
            received_successful_acks: SimpleCounter::new(
                "core_counter_received_successful_acks",
                "Number of received successful message acknowledgements",
            )
            .unwrap(),
            received_failed_acks: SimpleCounter::new(
                "core_counter_received_failed_acks",
                "Number of received failed message acknowledgements",
            )
            .unwrap(),
            sent_acks: SimpleCounter::new("core_counter_sent_acks", "Number of sent message acknowledgements").unwrap(),
            acked_tickets: SimpleCounter::new("core_counter_acked_tickets", "Number of acknowledged tickets").unwrap(),
        }
    }
}

pub struct AcknowledgementInteraction<T: HoprCoreDbActions> {
    db: RefCell<T>,
    on_acknowledgement: fn(HalfKeyChallenge),
    on_acknowledged_ticket: fn(AcknowledgedTicket),
    public_key: PublicKey,
    metrics: AckMetrics,
}

impl<T: HoprCoreDbActions> AcknowledgementInteraction<T> {
    pub fn send_acknowledgement(&self, acknowledgement: Acknowledgement, destination: PeerId) {
        // TODO: serialize and push to queue
        self.metrics.sent_acks.increment();
        todo!()
    }

    pub async fn handle_acknowledgement(&self, mut ack: Acknowledgement, remote_peer: PeerId) -> Result<()> {
        if !ack.validate(&self.public_key, &PublicKey::from_peerid(&remote_peer)?) {
            return Err(AcknowledgementValidation(
                "could not validate the acknowledgement".to_string(),
            ));
        }

        /*
         There are three cases:
          1. There is an unacknowledged ticket and we are
             awaiting a half key.
          2. We were the creator of the packet, hence we
             do not wait for any half key
          3. The acknowledgement is unexpected and stems from
             a protocol bug or an attacker
        */

        let pending = self
            .db
            .borrow()
            .get_pending_acknowledgement(&ack.ack_challenge())
            .await?
            .ok_or_else(|| {
                self.metrics.received_failed_acks.increment();
                return AcknowledgementValidation(format!(
                    "received unexpected acknowledgement for half key challenge {} - half key {}",
                    ack.ack_challenge().to_hex(),
                    ack.ack_key_share.to_hex()
                ));
            })?;

        match pending {
            PendingAcknowledgement::WaitingAsSender => {
                // No pending ticket, nothing to do.
                debug!("Received acknowledgement as sender. First relayer has processed the packet.");
                (self.on_acknowledgement)(ack.ack_challenge());
                self.metrics.received_successful_acks.increment();
            }

            PendingAcknowledgement::WaitingAsRelayer(unackowledged) => {
                // Try to unlock our incentive
                unackowledged.verify_challenge(&ack.ack_key_share).map_err(|e| {
                    self.metrics.received_failed_acks.increment();
                    return AcknowledgementValidation(format!(
                        "the acknowledgement is not sufficient to solve the embedded challenge, {e}"
                    ));
                })?;

                self.db
                    .borrow()
                    .get_channel_from(&unackowledged.signer)
                    .await
                    .map_err(|e| {
                        self.metrics.received_failed_acks.increment();
                        return AcknowledgementValidation(format!(
                            "acknowledgement received for channel that does not exist, {e}"
                        ));
                    })?;
                let response = unackowledged.get_response(&ack.ack_key_share)?;
                debug!("Acknowledging ticket. Using response {}", response.to_hex());

                let ack_ticket = AcknowledgedTicket::new(
                    unackowledged.ticket,
                    response,
                    Hash::new(&PREIMAGE_PLACE_HOLDER),
                    unackowledged.signer,
                );

                // replace the un-acked ticket with acked ticket.
                self.db
                    .borrow_mut()
                    .replace_unack_with_ack(&ack.ack_challenge(), ack_ticket.clone())
                    .await?;
                self.metrics.acked_tickets.increment();
                (self.on_acknowledged_ticket)(ack_ticket);
            }
        }
        Ok(())
    }
}

struct PacketMetrics {
    fwd_message_count: SimpleCounter,
    recv_message_count: SimpleCounter,
    tickets_count: SimpleCounter,
    packets_count: SimpleCounter,
}

impl Default for PacketMetrics {
    fn default() -> Self {
        Self {
            fwd_message_count: SimpleCounter::new("core_counter_forwarded_messages", "Number of forwarded messages")
                .unwrap(),
            recv_message_count: SimpleCounter::new("core_counter_received_messages", "Number of received messages")
                .unwrap(),
            tickets_count: SimpleCounter::new("core_counter_created_tickets", "Number of created tickets").unwrap(),
            packets_count: SimpleCounter::new("core_counter_packets", "Number of created packets").unwrap(),
        }
    }
}

pub struct PacketInteraction<T: HoprCoreDbActions> {
    db: RefCell<T>,
    ack_interaction: Arc<AcknowledgementInteraction<T>>,
    message_emitter: fn(&[u8]),
    check_unrealized_balance: bool,
    private_key: Box<[u8]>,
    metrics: PacketMetrics,
}

impl<T: HoprCoreDbActions> PacketInteraction<T> {
    async fn check_packet_tag(&self, packet: &Packet) -> Result<()> {
        match packet.state() {
            PacketState::Final { packet_tag, .. } | PacketState::Forwarded { packet_tag, .. } => {
                if self.db.borrow_mut().check_and_set_packet_tag(packet_tag).await? {
                    Ok(())
                } else {
                    Err(TagReplay)
                }
            }
            PacketState::Outgoing { .. } => Err(InvalidPacketState),
        }
    }

    async fn interact(&self, counterparty: PeerId, serialized_packet: &[u8]) -> Result<()> {
        todo!("send message")
    }

    async fn validate_unacknowledged_ticket(
        &self,
        sender: &PublicKey,
        min_ticket_amount: Balance,
        req_inverse_ticket_win_prob: U256,
        ticket: &Ticket,
        channel: &ChannelEntry,
        check_unrealized_balance: bool,
    ) -> Result<()> {
        let required_win_prob = U256::from_inverse_probability(req_inverse_ticket_win_prob)?;

        // ticket signer MUST be the sender
        ticket
            .verify(sender)
            .map_err(|e| TicketValidation(format!("ticket signer does not match the sender: {e}")))?;

        // ticket amount MUST be greater or equal to minTicketAmount
        if !ticket.amount.gte(&min_ticket_amount) {
            return Err(TicketValidation(format!(
                "ticket amount {} in not at least {min_ticket_amount}",
                ticket.amount
            )));
        }

        // ticket MUST have match X winning probability
        if !ticket.win_prob.eq(&required_win_prob) {
            return Err(TicketValidation(format!(
                "ticket winning probability {} is not equal to {required_win_prob}",
                ticket.win_prob
            )));
        }

        // channel MUST be open or pending to close
        if channel.status == ChannelStatus::Closed {
            return Err(TicketValidation(format!(
                "payment channel with {sender} is not opened or pending to close"
            )));
        }

        // ticket's epoch MUST match our channel's epoch
        if !ticket.epoch.eq(&channel.ticket_epoch) {
            return Err(TicketValidation(format!(
                "ticket epoch {} does not match our account epoch {} of channel {}",
                ticket.epoch,
                channel.ticket_epoch,
                channel.get_id()
            )));
        }

        // ticket's channelEpoch MUST match the current channel's epoch
        if !ticket.channel_epoch.eq(&channel.channel_epoch) {
            return Err(TicketValidation(format!(
                "ticket was created for a different channel iteration {} != {} of channel {}",
                ticket.channel_epoch,
                channel.channel_epoch,
                channel.get_id()
            )));
        }

        if check_unrealized_balance {
            info!("checking unrealized balances for channel {}", channel.get_id());

            let unrealized_balance = self
                .db
                .borrow()
                .get_tickets(sender)
                .await?                         // all tickets from sender
                .into_iter()
                .filter(|t| t.epoch.eq(&channel.ticket_epoch) && t.channel_epoch.eq(&channel.channel_epoch))
                .fold(channel.balance, |result, t| result.sub(&t.amount));

            // ensure sender has enough funds
            if ticket.amount.gt(&unrealized_balance) {
                return Err(OutOfFunds(channel.get_id().to_string()));
            }
        }

        Ok(())
    }

    async fn bump_ticket_index(&self, channel_id: &Hash) -> Result<U256> {
        let current_ticket_index = self
            .db
            .borrow()
            .get_current_ticket_index(channel_id)
            .await?
            .unwrap_or(U256::one());

        self.db
            .borrow_mut()
            .set_current_ticket_index(channel_id, current_ticket_index.addn(1))
            .await?;

        Ok(current_ticket_index)
    }

    async fn create_multihop_ticket(&self, destination: PublicKey, path_pos: u8) -> Result<Ticket> {
        let channel = self.db.borrow().get_channel_to(&destination).await?;
        let channel_id = channel.get_id();
        let current_index = self.bump_ticket_index(&channel_id).await?;
        let amount = Balance::new(
            U256::new(PRICE_PER_PACKET)
                .mul(U256::new(INVERSE_TICKET_WIN_PROB))
                .muln(path_pos as u32 - 1),
            BalanceType::HOPR,
        );

        let outstanding_balance = self
            .db
            .borrow()
            .get_pending_balance_to(&destination.to_address())
            .await?;

        let channel_balance = channel.balance.sub(&outstanding_balance);

        info!(
            "balances {} - {outstanding_balance} = {channel_balance} should >= {amount} in channel open to {}",
            channel.balance, channel.destination
        );

        if channel_balance.lt(&amount) {
            return Err(OutOfFunds(format!("{channel_id} with counterparty {destination}")));
        }

        let ticket = Ticket::new(
            destination.to_address(),
            None,
            channel.ticket_epoch,
            current_index,
            amount,
            U256::from_inverse_probability(U256::new(INVERSE_TICKET_WIN_PROB))?,
            channel.channel_epoch,
            &self.private_key,
        );

        self.db.borrow_mut().mark_pending(&ticket).await?;

        debug!(
            "Creating ticket in channel {channel_id}. Ticket data: {}",
            ticket.to_hex()
        );
        self.metrics.tickets_count.increment();
        Ok(ticket)
    }

    pub async fn send_outgoing_packet(&self, msg: &[u8], complete_valid_path: Path) -> Result<HalfKeyChallenge> {
        let next_peer = PublicKey::from_peerid(&complete_valid_path.hops()[0])?;

        let next_ticket = if complete_valid_path.length() == 1 {
            Ticket::new_zero_hop(next_peer, None, &self.private_key)
        } else {
            self.create_multihop_ticket(next_peer, complete_valid_path.length() as u8)
                .await?
        };

        self.metrics.packets_count.increment();

        let packet = Packet::new(msg, &complete_valid_path.hops(), &self.private_key, next_ticket)?;
        match packet.state() {
            PacketState::Outgoing { ack_challenge, .. } => {
                self.db
                    .borrow_mut()
                    .store_pending_acknowledgment(ack_challenge.clone(), PendingAcknowledgement::WaitingAsSender)
                    .await?;

                self.interact(complete_valid_path.hops()[0].clone(), &packet.to_bytes())
                    .await?;

                Ok(ack_challenge.clone())
            }
            _ => panic!("invalid packet state"),
        }
    }

    pub async fn handle_mixed_packet(&self, mut packet: Packet) -> Result<()> {
        self.check_packet_tag(&packet).await?;

        let next_ticket;
        let previous_peer;
        let next_peer;

        match packet.state() {
            PacketState::Outgoing { .. } => return Err(InvalidPacketState),

            PacketState::Final {
                plain_text,
                previous_hop,
                ..
            } => {
                // We're the destination of the packet, so emit the packet contents
                (self.message_emitter)(plain_text.as_ref());

                // And create acknowledgement
                let ack = packet.create_acknowledgement(&self.private_key).unwrap();
                self.ack_interaction.send_acknowledgement(ack, previous_hop.to_peerid());
                self.metrics.recv_message_count.increment();
                return Ok(());
            }

            PacketState::Forwarded {
                ack_challenge,
                previous_hop,
                own_key,
                next_hop,
                ..
            } => {
                let inverse_win_prob = U256::new(INVERSE_TICKET_WIN_PROB);

                // Validate the ticket first
                let channel = self.db.borrow().get_channel_from(&previous_hop).await?;
                if let Err(e) = self
                    .validate_unacknowledged_ticket(
                        &previous_hop,
                        Balance::from_str(PRICE_PER_PACKET, BalanceType::HOPR),
                        inverse_win_prob,
                        &packet.ticket,
                        &channel,
                        self.check_unrealized_balance,
                    )
                    .await
                {
                    // Mark as reject and passthrough the error
                    self.db.borrow_mut().mark_rejected(&packet.ticket).await?;
                    return Err(e);
                }

                self.db
                    .borrow_mut()
                    .set_current_ticket_index(&channel.get_id().hash(), packet.ticket.index)
                    .await?;

                // Store the unacknowledged ticket
                self.db
                    .borrow_mut()
                    .store_pending_acknowledgment(
                        ack_challenge.clone(),
                        PendingAcknowledgement::WaitingAsRelayer(UnacknowledgedTicket::new(
                            packet.ticket.clone(),
                            own_key.clone(),
                            previous_hop.clone(),
                        )),
                    )
                    .await?;

                let path_pos = packet
                    .ticket
                    .get_path_position(U256::new(PRICE_PER_PACKET), inverse_win_prob);

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
        self.interact(next_peer, &packet.to_bytes()).await?;

        // Acknowledge to the previous hop that we forwarded the packet
        let ack = packet.create_acknowledgement(&self.private_key).unwrap();
        self.ack_interaction.send_acknowledgement(ack, previous_peer);
        self.metrics.fwd_message_count.increment();

        Ok(())
    }
}
