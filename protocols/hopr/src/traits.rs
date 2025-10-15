use hopr_crypto_packet::{HoprSurb, ReplyOpener};
use hopr_crypto_packet::prelude::{HoprSenderId, HoprSurbId, PacketSignals};
use hopr_crypto_types::prelude::{HalfKeyChallenge, OffchainPublicKey};
use hopr_internal_types::prelude::{Acknowledgement, HoprPseudonym};
use hopr_network_types::prelude::{ResolvedTransportRouting, SurbMatcher};

pub use crate::errors::IncomingPacketError;
pub use crate::types::{FoundSurb, IncomingPacket, OutgoingPacket};

#[async_trait::async_trait]
pub trait SurbStore {
    async fn find_surb(&self, matcher: SurbMatcher) -> Option<FoundSurb>;

    async fn insert_surbs(&self, pseudonym: HoprPseudonym, surbs: Vec<(HoprSurbId, HoprSurb)>) -> usize;

    fn insert_reply_opener(&self, sender_id: HoprSenderId, opener: ReplyOpener);

    fn find_reply_opener(&self, sender_id: &HoprSenderId) -> Option<ReplyOpener>;
}

#[async_trait::async_trait]
pub trait PacketEncoder {
    type Error: std::error::Error;

    async fn encode<T: AsRef<[u8]> + Send, S: Into<PacketSignals> + Send>(
        &self,
        data: T,
        routing: ResolvedTransportRouting,
        signals: S,
    ) -> Result<OutgoingPacket, Self::Error>;
}

#[async_trait::async_trait]
pub trait PacketDecoder {
    type Error: std::error::Error;

    async fn decode_packet(
        &self,
        peer: OffchainPublicKey,
        data: Box<[u8]>,
    ) -> Result<IncomingPacket, IncomingPacketError<Self::Error>>;
}

#[async_trait::async_trait]
pub trait TicketProcessor {
    type Error: std::error::Error;

    async fn new_unacknowledged_ticket(
        &self,
    ) -> Result<(), Self::Error>;

    async fn acknowledge_ticket(
        &self,
        peer: OffchainPublicKey,
        ack: Acknowledgement,
    ) -> Result<(), Self::Error>;
}