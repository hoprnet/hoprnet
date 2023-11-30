use core_crypto::types::Hash;
use core_types::channels::ChannelEntry;
use std::fmt::{Display, Formatter};
use utils_types::primitives::Address;

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
        write!(f, "{:?} in tx {}", self.event_type, self.tx_hash)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChainEventType {
    Announcement(String, Address, Vec<String>), // peer, address, multiaddresses
    ChannelUpdate(ChannelEntry),
    TicketRedeem(ChannelEntry),
    NetworkRegistryUpdate(Address, bool),
}
