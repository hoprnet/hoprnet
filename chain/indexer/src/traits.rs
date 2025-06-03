use std::sync::Arc;

use alloy::primitives::B256;
use async_trait::async_trait;
use hopr_chain_types::{ContractAddresses, chain_events::SignificantChainEvent};
use hopr_primitive_types::prelude::*;

use crate::errors::Result;

#[async_trait]
pub trait ChainLogHandler {
    fn contract_addresses(&self) -> Vec<Address>;

    fn contract_addresses_map(&self) -> Arc<ContractAddresses>;

    fn contract_address_topics(&self, contract: Address) -> Vec<B256>;

    fn safe_address(&self) -> Address;

    async fn collect_log_event(&self, log: SerializableLog) -> Result<Option<SignificantChainEvent>>;
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
    /// mock.expect_collect_log_event()
    ///     .returning(|_| Ok(None));
    /// ```
    pub ChainLogHandler {}

    impl Clone for ChainLogHandler {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl ChainLogHandler for ChainLogHandler {
        fn contract_addresses(&self) -> Vec<Address>;
        fn contract_addresses_map(&self) -> Arc<ContractAddresses>;
        fn contract_address_topics(&self, contract: Address) -> Vec<B256>;
        fn safe_address(&self) -> Address;
        async fn collect_log_event(&self, log: SerializableLog) -> Result<Option<SignificantChainEvent>>;
    }
}
