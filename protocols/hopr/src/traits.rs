use std::ops::Mul;

use hopr_crypto_packet::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::TicketCreationError;
pub use crate::{
    errors::IncomingPacketError,
    types::{FoundSurb, IncomingPacket, OutgoingPacket, ResolvedAcknowledgement},
};

/// A trait defining the operations required to store and retrieve SURBs (Single Use Reply Blocks) and their reply
/// openers.
///
/// The sending side stores the reply openers, whereas the SURBs are stored by the replying side
/// of the communication.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait SurbStore {
    /// Tries to find SURB using the given [`matcher`](SurbMatcher).
    ///
    /// This is used by the replying side when it is about to send a reply packet back
    /// to the sender.
    async fn find_surb(&self, matcher: SurbMatcher) -> Option<FoundSurb>;

    /// Stores the `surbs` and associates them with the given [`pseudonym`](HoprPseudonym).
    ///
    /// This is used by the replying side when it receives packets containing SURBs from the sender
    /// with the given `pseudonym`.
    ///
    /// Returns the total number of SURBs available for that `pseudonym`, including the newly inserted
    /// ones.
    async fn insert_surbs(&self, pseudonym: HoprPseudonym, surbs: Vec<(HoprSurbId, HoprSurb)>) -> usize;

    /// Stores the given [`opener`](ReplyOpener) for the given [`sender_id`](HoprSenderId).
    ///
    /// This is done by the sending side, when it creates a packet containing a SURB to be delivered
    /// to the replying side.
    ///
    /// The operation should happen reasonably fast, as it is called from the packet processing code.
    fn insert_reply_opener(&self, sender_id: HoprSenderId, opener: ReplyOpener);

    /// Tries to find a [`ReplyOpener`] given the [`sender_id`](HoprSenderId).
    ///
    /// This is done by the sending side of the original packet when the reply to that
    /// packet is received and needs to be decrypted.
    ///
    /// The operation should happen reasonably fast, as it is called from the packet processing code.
    fn find_reply_opener(&self, sender_id: &HoprSenderId) -> Option<ReplyOpener>;
}

/// Trait defining encoder for [outgoing HOPR packets](OutgoingPacket).
///
/// These operations are done directly by the packet processing pipeline before
/// the outgoing packet is handled to the underlying p2p transport.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait PacketEncoder {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Encodes the given `data` and [`signals`](PacketSignals) for sending.
    ///
    /// The `data` MUST be already correctly sized for HOPR packets, otherwise the operation
    /// must fail.
    async fn encode_packet<T: AsRef<[u8]> + Send + 'static, S: Into<PacketSignals> + Send + 'static>(
        &self,
        data: T,
        routing: ResolvedTransportRouting,
        signals: S,
    ) -> Result<OutgoingPacket, Self::Error>;

    /// Encodes the given [`VerifiedAcknowledgement`] as an outgoing packet to be sent to the given
    /// [`peer`](OffchainPublicKey).
    async fn encode_acknowledgement(
        &self,
        ack: VerifiedAcknowledgement,
        peer: &OffchainPublicKey,
    ) -> Result<OutgoingPacket, Self::Error>;
}

/// Trait defining decoder HOPR packets.
///
/// This operation is done directly by the packet processing pipeline after
/// the underlying p2p transport hands over incoming data packets.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait PacketDecoder {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Decodes the `data` received from the given [`sender`](PeerId)
    /// returns the corresponding [`IncomingPacket`] if the decoding into a HOPR packet was successful.
    async fn decode(&self, sender: PeerId, data: Box<[u8]>)
    -> Result<IncomingPacket, IncomingPacketError<Self::Error>>;
}

/// Performs necessary processing of unacknowledged tickets in the HOPR packet processing pipeline.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait UnacknowledgedTicketProcessor {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Inserts a verified unacknowledged ticket from a delivered packet into the internal storage.
    ///
    /// The [`ticket`](UnacknowledgedTicket) corresponds to the given [`challenge`](HalfKeyChallenge)
    /// and awaits to be [acknowledged](UnacknowledgedTicketProcessor::acknowledge_ticket)
    /// once an [`Acknowledgement`] is received from the next hop.
    async fn insert_unacknowledged_ticket(
        &self,
        challenge: HalfKeyChallenge,
        ticket: UnacknowledgedTicket,
    ) -> Result<(), Self::Error>;

    /// Finds and acknowledges previously inserted ticket, using an [`Acknowledgement`] from the
    /// next [`peer`](OffchainPublicKey).
    ///
    /// This function must verify the given acknowledgement and find if it contains a solution
    /// to a challenge of a previously [inserted ticket](UnacknowledgedTicketProcessor::insert_unacknowledged_ticket).
    ///
    /// On success, the [resolution](ResolvedAcknowledgement) contains a decision whether the corresponding previously
    /// stored ticket was found, and whether it is winning (and thus also redeemable) or losing.
    ///
    /// Must return an error if no ticket with the challenge corresponding to the [`Acknowledgement`] was inserted.
    async fn acknowledge_ticket(
        &self,
        peer: OffchainPublicKey,
        ack: Acknowledgement,
    ) -> Result<ResolvedAcknowledgement, Self::Error>;
}

/// Allows tracking ticket indices of outgoing channels and
/// unrealized balances of incoming channels.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait TicketTracker {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Gets the next ticket index for an outgoing ticket for the given channel.
    async fn next_outgoing_ticket_index(&self, channel_id: &ChannelId, epoch: u32) -> Result<u64, Self::Error>;

    /// Retrieves the unrealized balance of the given channel.
    ///
    /// This allows guarding from situations where the ticket issuer issues more tickets
    /// than there's balance in the given channel.
    async fn incoming_channel_unrealized_balance(
        &self,
        channel_id: &ChannelId,
        epoch: u32,
    ) -> Result<HoprBalance, Self::Error>;

    /// Convenience function that allows creating multi-hop tickets.
    async fn create_multihop_ticket(
        &self,
        channel: &ChannelEntry,
        current_path_pos: u8,
        winning_prob: WinningProbability,
        ticket_price: HoprBalance,
    ) -> Result<TicketBuilder, TicketCreationError<Self::Error>> {
        // The next ticket is worth: price * remaining hop count / winning probability
        let amount = HoprBalance::from(
            ticket_price
                .amount()
                .mul(U256::from(current_path_pos - 1))
                .div_f64(winning_prob.into())
                .expect("winning probability is always less than or equal to 1"),
        );

        if channel.balance.lt(&amount) {
            return Err(TicketCreationError::OutOfFunds(*channel.get_id(), amount));
        }

        let ticket_builder = TicketBuilder::default()
            .counterparty(channel.destination)
            .balance(amount)
            .index(
                self.next_outgoing_ticket_index(channel.get_id(), channel.channel_epoch)
                    .await
                    .map_err(TicketCreationError::Other)?,
            )
            .win_prob(winning_prob)
            .channel_epoch(channel.channel_epoch);

        Ok(ticket_builder)
    }
}
