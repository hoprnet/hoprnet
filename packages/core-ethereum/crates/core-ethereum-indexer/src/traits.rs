use async_trait::async_trait;

use core_types::channels::ChannelEntry;
use ethers::abi::RawLog;
use utils_types::primitives::{Address, Balance, Snapshot};

#[derive(Debug, Clone, PartialEq)]
pub enum SignificantChainEvent {
    Announcement(String, Address, Vec<String>), // peer, address, multiaddresses
    ChannelUpdate(ChannelEntry),
    TicketRedeem(ChannelEntry, Balance),
    NetworkRegistryUpdate(Address, bool),
}

#[cfg_attr(test, mockall::automock)]
#[async_trait(? Send)]
pub trait ChainLogHandler {
    fn contract_addresses(&self) -> Vec<Address>;

    async fn on_event(
        &self,
        address: Address,
        block_number: u32,
        log: RawLog,
        snapshot: Snapshot,
    ) -> crate::errors::Result<Option<SignificantChainEvent>>;
}
