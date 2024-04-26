use async_trait::async_trait;

use crate::errors::Result;
use chain_rpc::BlockWithLogs;
use chain_types::chain_events::SignificantChainEvent;
use hopr_primitive_types::prelude::*;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ChainLogHandler {
    fn contract_addresses(&self) -> Vec<Address>;

    async fn collect_block_events(&self, block_with_logs: BlockWithLogs) -> Result<Vec<SignificantChainEvent>>;
}
