use std::sync::Arc;
use ringbuffer::RingBuffer;
use hopr_crypto_packet::{HoprSurb, ReplyOpener};
use hopr_crypto_packet::prelude::{HoprSenderId, HoprSurbId};
use hopr_crypto_types::prelude::{HalfKeyChallenge, OffchainPublicKey};
use hopr_internal_types::prelude::{Acknowledgement, HoprPseudonym};
use hopr_network_types::prelude::{ResolvedTransportRouting, SurbMatcher};
use hopr_protocol_app::prelude::ApplicationDataOut;
use crate::errors::IncomingPacketError;
use crate::types::{FoundSurb, OutgoingPacket};

#[async_trait::async_trait]
pub trait SurbStore {
    async fn find_surb(&self, matcher: SurbMatcher) -> Option<FoundSurb>;

    async fn insert_surbs(&self, pseudonym: HoprPseudonym, surbs: Vec<(HoprSurbId, HoprSurb)>) -> usize;

    fn insert_reply_opener(&self, sender_id: HoprSenderId, opener: ReplyOpener);

    fn find_reply_opener(&self, sender_id: &HoprSenderId) -> Option<ReplyOpener>;
}

#[async_trait::async_trait]
pub trait PacketWrapping {
    type Input;
    type Error;

    async fn send_data(
        &self,
        data: ApplicationDataOut,
        routing: ResolvedTransportRouting,
    ) -> Result<OutgoingPacket, Self::Error>;

    async fn send_ack(
        &self,
        ack: Acknowledgement,
        destination: &OffchainPublicKey,
    ) -> Result<OutgoingPacket, Self::Error>;
}

#[async_trait::async_trait]
pub trait PacketUnwrapping {
    type Packet;

    type Error;

    async fn recv_data(
        &self,
        peer: OffchainPublicKey,
        data: Box<[u8]>,
    ) -> Result<Self::Packet, IncomingPacketError<Self::Error>>;

    async fn recv_ack(
        &self,
        peer: OffchainPublicKey,
        ack: Acknowledgement,
    ) -> Result<(), Self::Error>;
}