use async_trait::async_trait;
use ethers_providers::{JsonRpcClient, Middleware, Provider};
use serde::{Deserialize, Serialize};
use validator::Validate;
use core_crypto::types::Hash;
use utils_types::primitives::{Address, Balance, U256};
use crate::errors::Result;

pub mod http_provider;
pub mod errors;

#[async_trait]
pub trait HoprRpcOperations {

    async fn genesis_block(&self) -> Result<u64>;

    async fn block_number(&self) -> Result<u64>;

    async fn get_native_balance(&self) -> Result<Balance>;

    async fn get_token_balance(&self) -> Result<Balance>;

    async fn get_transactions_in_block(&self) -> Result<Vec<Hash>>;

    async fn get_node_management_module_target_info(&self) -> Result<U256>;

    async fn get_safe_from_node_safe_registry(&self) -> Result<Address>;

    async fn get_module_target_address(&self) -> Result<Address>;
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct RpcOperationsConfig {
    indexer_start_block_number: u64
}

pub struct RpcOperations<P: JsonRpcClient> {
    provider: Provider<P>,
    cfg: RpcOperationsConfig,
}

#[async_trait]
impl<P: JsonRpcClient> HoprRpcOperations for RpcOperations<P> {
    async fn genesis_block(&self) -> Result<u64> {
        Ok(self.cfg.indexer_start_block_number)
    }

    async fn block_number(&self) -> Result<u64> {
        Ok(self.provider.get_block_number()?.as_u64())
    }

    async fn get_native_balance(&self) -> Result<Balance> {
        todo!()
    }

    async fn get_token_balance(&self) -> Result<Balance> {
        todo!()
    }

    async fn get_transactions_in_block(&self) -> Result<Vec<Hash>> {
        todo!()
    }

    async fn get_node_management_module_target_info(&self) -> Result<U256> {
        todo!()
    }

    async fn get_safe_from_node_safe_registry(&self) -> Result<Address> {
        todo!()
    }

    async fn get_module_target_address(&self) -> Result<Address> {
        todo!()
    }
}