use hopr_api::types::{crypto::prelude::*, internal::prelude::*};
use hopr_crypto_packet::prelude::*;

pub use crate::{
    errors::IncomingPacketError,
    types::{FoundSurb, IncomingPacket, OutgoingPacket, ResolvedAcknowledgement},
};

/// A trait defining the operations required to store and retrieve SURBs (Single Use Reply Blocks) and their reply
/// openers.
///
/// The sending side stores the reply openers, whereas the SURBs are stored by the replying side
/// of the communication.
// TODO: refactor this trait to be sync (see https://github.com/hoprnet/hoprnet/pull/7915)
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
// TODO: refactor this trait to be sync (see https://github.com/hoprnet/hoprnet/pull/7915)
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
        routing: ResolvedTransportRouting<HoprSurb>,
        signals: S,
    ) -> Result<OutgoingPacket, Self::Error>;

    /// Encodes the given vector of [`VerifiedAcknowledgements`](VerifiedAcknowledgement) as an outgoing packet to be
    /// sent to the given [`destination`](OffchainPublicKey).
    async fn encode_acknowledgements(
        &self,
        acks: &[VerifiedAcknowledgement],
        destination: &OffchainPublicKey,
    ) -> Result<OutgoingPacket, Self::Error>;
}

/// Trait defining decoder HOPR packets.
///
/// This operation is done directly by the packet processing pipeline after
/// the underlying p2p transport hands over incoming data packets.
// TODO: refactor this trait to be sync (see https://github.com/hoprnet/hoprnet/pull/7915)
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait PacketDecoder {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Decodes the `data` received from the given [`sender`](PeerId)
    /// returns the corresponding [`IncomingPacket`] if the decoding into a HOPR packet was successful.
    async fn decode(&self, sender: PeerId, data: Box<[u8]>)
    -> Result<IncomingPacket, IncomingPacketError<Self::Error>>;
}

/// Defines errors returned by `UnacknowledgedTicketProcessor::acknowledge_ticket`.
#[derive(Debug, thiserror::Error)]
pub enum TicketAcknowledgementError<E> {
    /// An acknowledgement from a peer was not expected.
    #[error("acknowledgement from the peer was not expected")]
    UnexpectedAcknowledgement,
    /// An error occurred while processing the acknowledgement.
    #[error(transparent)]
    Inner(E),
}

impl<E> TicketAcknowledgementError<E> {
    pub fn inner<F: Into<E>>(e: F) -> Self {
        Self::Inner(e.into())
    }
}

/// Performs necessary processing of unacknowledged tickets in the HOPR packet processing pipeline.
// TODO: refactor this trait to be sync (see https://github.com/hoprnet/hoprnet/pull/7915)
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait UnacknowledgedTicketProcessor {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Inserts a verified unacknowledged ticket from a delivered packet into the internal storage.
    ///
    /// The [`ticket`](UnacknowledgedTicket) corresponds to the given [`challenge`](HalfKeyChallenge)
    /// and awaits to be [acknowledged](UnacknowledgedTicketProcessor::acknowledge_tickets)
    /// once an [`Acknowledgement`] is received from the `next_hop`.
    async fn insert_unacknowledged_ticket(
        &self,
        next_hop: &OffchainPublicKey,
        challenge: HalfKeyChallenge,
        ticket: UnacknowledgedTicket,
    ) -> Result<(), Self::Error>;

    /// Finds and acknowledges previously inserted tickets, using incoming [`Acknowledgements`](Acknowledgement) from
    /// the upstream [`peer`](OffchainPublicKey).
    ///
    /// Function should first check if any acknowledgements are expected from the given `peer`.
    ///
    /// Furthermore, the function must verify each given acknowledgement and find if it evaluates to any solutions
    /// to challenges of previously [inserted tickets](UnacknowledgedTicketProcessor::insert_unacknowledged_ticket).
    ///
    /// On success, the [resolutions](ResolvedAcknowledgement) contain decisions whether the previously
    /// stored ticket with a matching challenge was found, and whether it is winning (and thus also redeemable) or
    /// losing.
    /// Challenges for which tickets were not found are skipped.
    ///
    /// Must return [`TicketAcknowledgementError::UnexpectedAcknowledgement`] if no `Acknowledgements` from the given
    /// `peer` was expected.
    async fn acknowledge_tickets(
        &self,
        peer: OffchainPublicKey,
        acks: Vec<Acknowledgement>,
    ) -> Result<Vec<ResolvedAcknowledgement>, TicketAcknowledgementError<Self::Error>>;
}
