use async_trait::async_trait;

use chain_types::chain_events::ChainEventType;
use ethers::abi::RawLog;
use hopr_primitive_types::prelude::*;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ChainLogHandler {
    fn contract_addresses(&self) -> Vec<Address>;

    async fn on_event(
        &self,
        address: Address,
        block_number: u32,
        log: RawLog,
    ) -> crate::errors::Result<Option<ChainEventType>>;
}
