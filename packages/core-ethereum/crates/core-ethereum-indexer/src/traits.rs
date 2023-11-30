use async_trait::async_trait;

use ethers::abi::RawLog;
use core_ethereum_types::chain_events::ChainEventType;
use utils_types::primitives::{Address, Snapshot};

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
    ) -> crate::errors::Result<Option<ChainEventType>>;
}
