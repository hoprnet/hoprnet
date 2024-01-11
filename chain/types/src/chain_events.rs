use hopr_crypto::types::Hash;
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::channels::ChannelEntry;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;
use utils_types::primitives::{Address, Balance};

/// Contains TX hash along with the Chain Event data.
/// This could be used to pair up some events with `Action`
#[derive(Debug, Clone, PartialEq)]
pub struct SignificantChainEvent {
    /// TX hash
    pub tx_hash: Hash,
    /// Chain event of interest
    pub event_type: ChainEventType,
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
    /// If the channel is own, also contains the ticket that has been redeemed.
    TicketRedeemed(ChannelEntry, Option<AcknowledgedTicket>),
    /// Safe has been registered with the node.
    NodeSafeRegistered(Address),
    /// Network registry update for a node.
    NetworkRegistryUpdate(Address, NetworkRegistryStatus),
}
