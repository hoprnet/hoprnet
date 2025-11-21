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

#[async_trait::async_trait]
pub trait SurbStore {
    async fn find_surb(&self, matcher: SurbMatcher) -> Option<FoundSurb>;

    async fn insert_surbs(&self, pseudonym: HoprPseudonym, surbs: Vec<(HoprSurbId, HoprSurb)>) -> usize;

    fn insert_reply_opener(&self, sender_id: HoprSenderId, opener: ReplyOpener);

    fn find_reply_opener(&self, sender_id: &HoprSenderId) -> Option<ReplyOpener>;
}

#[async_trait::async_trait]
pub trait PacketEncoder {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn encode_packet<T: AsRef<[u8]> + Send + 'static, S: Into<PacketSignals> + Send + 'static>(
        &self,
        data: T,
        routing: ResolvedTransportRouting,
        signals: S,
    ) -> Result<OutgoingPacket, Self::Error>;

    async fn encode_acknowledgement(
        &self,
        ack: VerifiedAcknowledgement,
        peer: &OffchainPublicKey,
    ) -> Result<OutgoingPacket, Self::Error>;
}

#[async_trait::async_trait]
pub trait PacketDecoder {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn decode(&self, sender: PeerId, data: Box<[u8]>)
    -> Result<IncomingPacket, IncomingPacketError<Self::Error>>;
}

/// Performs necessary processing of unacknowledged tickets.
#[async_trait::async_trait]
pub trait UnacknowledgedTicketProcessor {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn insert_unacknowledged_ticket(
        &self,
        challenge: HalfKeyChallenge,
        ticket: UnacknowledgedTicket,
    ) -> Result<(), Self::Error>;

    // Finds and acknowledges previously inserted ticket
    async fn acknowledge_ticket(
        &self,
        peer: OffchainPublicKey,
        ack: Acknowledgement,
    ) -> Result<ResolvedAcknowledgement, Self::Error>;
}

/// Allows tracking ticket indices of outgoing channels and
/// unrealized balances of incoming channels.
#[async_trait::async_trait]
pub trait TicketTracker {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Gets the next ticket index for an outgoing ticket for the given channel.
    async fn next_outgoing_ticket_index(&self, channel_id: &ChannelId) -> Result<u64, Self::Error>;

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
            .channel_id(*channel.get_id())
            .balance(amount)
            .index(
                self.next_outgoing_ticket_index(channel.get_id())
                    .await
                    .map_err(TicketCreationError::Other)?,
            )
            .win_prob(winning_prob)
            .channel_epoch(channel.channel_epoch.as_u32());

        Ok(ticket_builder)
    }
}
