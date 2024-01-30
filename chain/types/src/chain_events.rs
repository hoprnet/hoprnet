use std::fmt::{Display, Formatter};
use hopr_crypto_types::types::Hash;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;

/// Contains TX hash along with the Chain Event data.
/// This could be used to pair up some events with `Action`
#[derive(Debug, Clone, PartialEq)]
pub struct SignificantChainEvent {
    /// TX hash
    pub tx_hash: Hash,
    /// Chain event of interest
    pub event_type: ChainEventType,
}

impl Display for SignificantChainEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} in tx {}", self.tx_hash, self.event_type)
    }
}

/// Status of a node in network registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkRegistryStatus {
    /// Connections to the node are allowed.
    Allowed,
    /// Connections to the node are not allowed.
    Denied,
}

/// Enumeration of HOPR chain events.
#[allow(clippy::large_enum_variant)] // TODO: Refactor the large enum variant
#[derive(Debug, Clone, PartialEq)]
pub enum ChainEventType {
    /// Peer on-chain announcement event.
    Announcement {
        /// Announced peer id
        peer: PeerId,
        /// Announced on-chain address
        address: Address,
        /// Multiaddresses
        multiaddresses: Vec<Multiaddr>,
    },
    /// New channel has been opened
    ChannelOpened(ChannelEntry),
    /// Channel closure has been initiated.
    ChannelClosureInitiated(ChannelEntry),
    /// Channel closure has been finalized.
    ChannelClosed(ChannelEntry),
    /// Channel balance has increased by an amount.
    ChannelBalanceIncreased(ChannelEntry, Balance),
    /// Channel balance has decreased by an amount.
    ChannelBalanceDecreased(ChannelEntry, Balance),
    /// Ticket has been redeemed on a channel.
    /// If the channel is node's own, also contains the ticket that has been redeemed.
    TicketRedeemed(ChannelEntry, Option<AcknowledgedTicket>),
    /// Safe has been registered with the node.
    NodeSafeRegistered(Address),
    /// Network registry update for a node.
    NetworkRegistryUpdate(Address, NetworkRegistryStatus),
}

impl Display for ChainEventType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainEventType::Announcement { peer, address, multiaddresses } => write!(f, "announcement event of {peer} ({address}): {:?}", multiaddresses),
            ChainEventType::ChannelOpened(c) => write!(f, "open channel event {}", c.get_id()),
            ChainEventType::ChannelClosureInitiated(c) => write!(f, "close channel initiation event {}", c.get_id()),
            ChainEventType::ChannelClosed(c) => write!(f, "close channel event {}", c.get_id()),
            ChainEventType::ChannelBalanceIncreased(c, _) => write!(f, "channel increase balance event {}", c.get_id()),
            ChainEventType::ChannelBalanceDecreased(c, _) => write!(f, "channel decrease balance event {}", c.get_id()),
            ChainEventType::TicketRedeemed(c, _) => write!(f, "ticket redeem event in channel {}", c.get_id()),
            ChainEventType::NodeSafeRegistered(s) => write!(f, "safe registered event {s}"),
            ChainEventType::NetworkRegistryUpdate(a, s) => write!(f, "network registry update event {a}: {:?}", s),
        }
    }
}