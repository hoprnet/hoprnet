use async_trait::async_trait;
use hopr_api_traits::chain::{ChainKeyOperations, ChainReadChannelOperations};
use hopr_crypto_packet::prelude::PacketSignals;
pub use hopr_crypto_packet::{HoprSurb, prelude::HoprSenderId};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::{ResolvedTransportRouting, SurbMatcher};
use hopr_primitive_types::balance::HoprBalance;

use crate::errors::Result;

/// Contains a SURB found in the SURB ring buffer via [`HoprDbProtocolOperations::find_surb`].
#[derive(Debug)]
pub struct FoundSurb {
    /// Complete sender ID of the SURB.
    pub sender_id: HoprSenderId,
    /// The SURB itself.
    pub surb: HoprSurb,
    /// Number of SURBs remaining in the ring buffer with the same pseudonym.
    pub remaining: usize,
}

/// Configuration for the SURB cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
pub struct SurbCacheConfig {
    /// Size of the SURB ring buffer per pseudonym.
    pub rb_capacity: usize,
    /// Threshold for the number of SURBs in the ring buffer, below which it is
    /// considered low ("SURB distress").
    pub distress_threshold: usize,
}

/// Trait defining all DB functionality needed by a packet/acknowledgement processing pipeline.
#[async_trait]
pub trait HoprDbProtocolOperations {
    /// Processes the acknowledgements for the pending tickets
    ///
    /// There are three cases:
    /// 1. There is an unacknowledged ticket and we are awaiting a half key.
    /// 2. We were the creator of the packet, hence we do not wait for any half key
    /// 3. The acknowledgement is unexpected and stems from a protocol bug or an attacker
    async fn handle_acknowledgement<R>(&self, ack: VerifiedAcknowledgement, chain_resolver: &R) -> Result<()>
    where R: ChainReadChannelOperations + Send + Sync + 'static;
    
    /// Attempts to find SURB and its ID given the [`SurbMatcher`].
    async fn find_surb(&self, matcher: SurbMatcher) -> Result<FoundSurb>;

    /// Returns the SURB cache configuration.
    fn get_surb_config(&self) -> SurbCacheConfig;

    /// Process the data into an outgoing packet that is not going to be acknowledged.
    async fn to_send_no_ack<R>(&self, data: Box<[u8]>, destination: OffchainPublicKey, resolver: &R) -> Result<OutgoingPacket>
    where R: ChainReadChannelOperations + ChainKeyOperations + Send + Sync + 'static;

    /// Process the data into an outgoing packet
    async fn to_send<R>(
        &self,
        data: Box<[u8]>,
        routing: ResolvedTransportRouting,
        outgoing_ticket_win_prob: WinningProbability,
        outgoing_ticket_price: HoprBalance,
        signals: PacketSignals,
        resolver: &R,
    ) -> Result<OutgoingPacket>
    where R: ChainReadChannelOperations + ChainKeyOperations + Send + Sync + 'static;

    /// Process the incoming packet into data
    #[allow(clippy::wrong_self_convention)]
    async fn from_recv<R>(
        &self,
        data: Box<[u8]>,
        pkt_keypair: &OffchainKeypair,
        sender: OffchainPublicKey,
        outgoing_ticket_win_prob: WinningProbability,
        outgoing_ticket_price: HoprBalance,
        resolver: &R,
    ) -> Result<IncomingPacket>
    where R: ChainReadChannelOperations + ChainKeyOperations + Send + Sync + 'static;
}

/// Contains some miscellaneous information about a received packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct AuxiliaryPacketInfo {
    /// Packet signals that the packet carried.
    ///
    /// Zero if no signal flags were specified.
    pub packet_signals: PacketSignals,
    /// Number of SURBs that the packet carried.
    pub num_surbs: usize,
}

#[allow(clippy::large_enum_variant)] // TODO: Uses too large objects
pub enum IncomingPacket {
    /// Packet is intended for us
    Final {
        packet_tag: PacketTag,
        previous_hop: OffchainPublicKey,
        sender: HoprPseudonym,
        plain_text: Box<[u8]>,
        ack_key: HalfKey,
        info: AuxiliaryPacketInfo,
    },
    /// Packet must be forwarded
    Forwarded {
        packet_tag: PacketTag,
        previous_hop: OffchainPublicKey,
        next_hop: OffchainPublicKey,
        data: Box<[u8]>,
        /// Acknowledgement payload to be sent to the previous hop
        ack_key: HalfKey,
    },
    /// The packet contains an acknowledgement of a delivered packet.
    Acknowledgement {
        packet_tag: PacketTag,
        previous_hop: OffchainPublicKey,
        ack: Acknowledgement,
    },
}

/// Packet that is being sent out by us
pub struct OutgoingPacket {
    pub next_hop: OffchainPublicKey,
    pub ack_challenge: HalfKeyChallenge,
    pub data: Box<[u8]>,
}

impl std::fmt::Debug for OutgoingPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutgoingPacket")
            .field("next_hop", &self.next_hop)
            .field("ack_challenge", &self.ack_challenge)
            .finish_non_exhaustive()
    }
}

#[allow(clippy::large_enum_variant)] // TODO: Uses too large objects
pub enum ResolvedAcknowledgement {
    Sending(VerifiedAcknowledgement),
    RelayingWin(AcknowledgedTicket),
    RelayingLoss(Hash),
}
