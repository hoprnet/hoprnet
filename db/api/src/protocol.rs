use async_trait::async_trait;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::ResolvedTransportRouting;
use hopr_primitive_types::prelude::Balance;
use std::fmt::Debug;

use crate::errors::Result;

/// Trait defining all DB functionality needed by packet/acknowledgement processing pipeline.
#[async_trait]
pub trait HoprDbProtocolOperations {
    /// Processes the acknowledgements for the pending tickets
    ///
    /// There are three cases:
    /// 1. There is an unacknowledged ticket and we are awaiting a half key.
    /// 2. We were the creator of the packet, hence we do not wait for any half key
    /// 3. The acknowledgement is unexpected and stems from a protocol bug or an attacker
    async fn handle_acknowledgement(&self, ack: Acknowledgement) -> crate::errors::Result<AckResult>;

    /// Loads (presumably cached) value of the network's minimum winning probability from the DB.
    async fn get_network_winning_probability(&self) -> Result<f64>;

    /// Loads (presumably cached) value of the network's minimum ticket price from the DB.
    async fn get_network_ticket_price(&self) -> Result<Balance>;

    /// Process the data into an outgoing packet that is not going to be acknowledged.
    async fn to_send_no_ack(&self, data: Box<[u8]>, destination: OffchainPublicKey) -> Result<OutgoingPacket>;

    /// Process the data into an outgoing packet
    async fn to_send(
        &self,
        data: Box<[u8]>,
        routing: ResolvedTransportRouting,
        outgoing_ticket_win_prob: f64,
        outgoing_ticket_price: Balance,
    ) -> Result<OutgoingPacket>;

    /// Process the incoming packet into data
    #[allow(clippy::wrong_self_convention)]
    async fn from_recv(
        &self,
        data: Box<[u8]>,
        pkt_keypair: &OffchainKeypair,
        sender: OffchainPublicKey,
        outgoing_ticket_win_prob: f64,
        outgoing_ticket_price: Balance,
    ) -> Result<Option<IncomingPacket>>;
}

pub enum IncomingPacket {
    /// Packet is intended for us
    Final {
        packet_tag: PacketTag,
        previous_hop: OffchainPublicKey,
        plain_text: Box<[u8]>,
        ack_key: Option<HalfKey>,
    },
    /// Packet must be forwarded
    Forwarded {
        packet_tag: PacketTag,
        previous_hop: OffchainPublicKey,
        next_hop: OffchainPublicKey,
        data: Box<[u8]>,
        ack: Acknowledgement,
    },
}

/// Packet that is being sent out by us
pub struct OutgoingPacket {
    pub next_hop: OffchainPublicKey,
    pub ack_challenge: HalfKeyChallenge,
    pub data: Box<[u8]>,
}

#[allow(clippy::large_enum_variant)] // TODO: Uses too large objects
pub enum AckResult {
    Sender(Acknowledgement),
    RelayerWinning(AcknowledgedTicket),
    RelayerLosing,
}

impl Debug for AckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sender(_) => f.debug_tuple("Sender").finish(),
            Self::RelayerWinning(_) => f.debug_tuple("RelayerWinning").finish(),
            Self::RelayerLosing => write!(f, "RelayerLosing"),
        }
    }
}

#[allow(clippy::large_enum_variant)] // TODO: Uses too large objects
pub enum ResolvedAcknowledgement {
    Sending(Acknowledgement),
    RelayingWin(AcknowledgedTicket),
    RelayingLoss(Hash),
}

impl From<ResolvedAcknowledgement> for AckResult {
    fn from(value: ResolvedAcknowledgement) -> Self {
        match value {
            ResolvedAcknowledgement::Sending(ack) => AckResult::Sender(ack),
            ResolvedAcknowledgement::RelayingWin(ack_ticket) => AckResult::RelayerWinning(ack_ticket),
            ResolvedAcknowledgement::RelayingLoss(_) => AckResult::RelayerLosing,
        }
    }
}
