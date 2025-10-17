use hopr_crypto_packet::{
    HoprSurb, ReplyOpener,
    prelude::{HoprSenderId, HoprSurbId, PacketSignals},
};
use hopr_crypto_types::prelude::{HalfKeyChallenge, OffchainPublicKey};
use hopr_internal_types::{
    prelude::{Acknowledgement, HoprPseudonym, Ticket, UnacknowledgedTicket},
    protocol::VerifiedAcknowledgement,
};
use hopr_network_types::prelude::{ResolvedTransportRouting, SurbMatcher};

pub use crate::{
    errors::IncomingPacketError,
    types::{FoundSurb, IncomingPacket, OutgoingPacket},
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
    type Error: std::error::Error + Send + Sync;

    async fn encode_packet<T: AsRef<[u8]> + Send, S: Into<PacketSignals> + Send>(
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
    type Error: std::error::Error + Send + Sync;

    async fn decode(
        &self,
        peer: OffchainPublicKey,
        data: Box<[u8]>,
    ) -> Result<IncomingPacket, IncomingPacketError<Self::Error>>;
}

#[async_trait::async_trait]
pub trait TicketProcessor {
    type Error: std::error::Error + Send + Sync;

    async fn insert_unacknowledged_ticket(
        &self,
        challenge: HalfKeyChallenge,
        ticket: UnacknowledgedTicket,
    ) -> Result<(), Self::Error>;

    // Finds and acknowledges previously inserted ticket
    async fn acknowledge_ticket(&self, peer: OffchainPublicKey, ack: Acknowledgement) -> Result<(), Self::Error>;

    async fn reject_ticket(&self, peer: OffchainPublicKey, ticket: Ticket) -> Result<(), Self::Error>;
}
