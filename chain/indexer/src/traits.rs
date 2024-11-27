use async_trait::async_trait;
use ethers::types::TxHash;
use std::sync::Arc;

use chain_types::chain_events::SignificantChainEvent;
use chain_types::ContractAddresses;
use hopr_chain_rpc::BlockWithLogs;
use hopr_primitive_types::prelude::*;

use crate::errors::Result;

#[async_trait]
pub trait ChainLogHandler {
    fn contract_addresses(&self) -> Vec<Address>;

    fn contract_addresses_map(&self) -> Arc<ContractAddresses>;

    fn contract_address_topics(&self, contract: Address) -> Vec<TxHash>;

    fn safe_address(&self) -> Address;

    async fn collect_block_events(&self, block_with_logs: BlockWithLogs) -> Result<Vec<SignificantChainEvent>>;
}

#[cfg(test)]
use mockall::mock;

#[cfg(test)]
mock! {
    /// Mock implementation of ChainLogHandler for testing.
    ///
    /// # Example
    /// ```
    /// use mockall::predicate::*;
    /// let mut mock = MockChainLogHandler::new();
    /// mock.expect_collect_block_events()
    ///     .returning(|_| Ok(vec![]));
    /// ```
    pub ChainLogHandler {}

    impl Clone for ChainLogHandler {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl ChainLogHandler for ChainLogHandler {
        fn contract_addresses(&self) -> Vec<Address>;
        fn contract_addresses_map(&self) -> Arc<ContractAddresses>;
        fn contract_address_topics(&self, contract: Address) -> Vec<TxHash>;
        fn safe_address(&self) -> Address;
        async fn collect_block_events(&self, block_with_logs: BlockWithLogs) -> Result<Vec<SignificantChainEvent>>;
    }
}
