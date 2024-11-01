use async_trait::async_trait;
use ethers::types::TxHash;

use chain_types::chain_events::SignificantChainEvent;
use hopr_chain_rpc::BlockWithLogs;
use hopr_primitive_types::prelude::*;

use crate::errors::Result;

#[async_trait]
pub trait ChainLogHandler {
    fn contract_addresses(&self) -> Vec<Address>;

    fn contract_address_topics(&self, contract: Address) -> Vec<TxHash>;

    async fn collect_block_events(&self, block_with_logs: BlockWithLogs) -> Result<Vec<SignificantChainEvent>>;
}

#[cfg(test)]
/// Mock implementation of ChainLogHandler for testing.
/// 
/// # Example
/// ```
/// use mockall::predicate::*;
/// let mut mock = MockChainLogHandler::new();
/// mock.expect_collect_block_events()
///     .returning(|_| Ok(vec![]));
/// ```
mock! {
    pub ChainLogHandler {}

    impl Clone for ChainLogHandler {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl ChainLogHandler for ChainLogHandler {
        fn contract_addresses(&self) -> Vec<Address>;
        fn contract_address_topics(&self, contract: Address) -> Vec<TxHash>;
        async fn collect_block_events(&self, block_with_logs: BlockWithLogs) -> Result<Vec<SignificantChainEvent>>;
    }
}
