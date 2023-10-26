use async_trait::async_trait;
use ethers::types::transaction::eip2718::TypedTransaction;
use core_crypto::types::Hash;
use utils_types::primitives::{Address, Balance, BalanceType, U256};

use crate::errors::Result;

pub mod errors;
pub mod rpc;

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub mod nodejs_provider;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait HoprRpcOperations {

    async fn genesis_block(&self) -> Result<u64>;

    async fn block_number(&self) -> Result<u64>;

    async fn get_timestamp(&self, block_number: u64) -> Result<Option<u64>>;

    async fn get_balance(&self, balance_type: BalanceType) -> Result<Balance>;

    async fn get_transactions_in_block(&self, block_number: u64) -> Result<Vec<Hash>>;

    async fn get_node_management_module_target_info(&self, target: Address) -> Result<Option<U256>>;

    async fn get_safe_from_node_safe_registry(&self, node: Address) -> Result<Address>;

    async fn get_module_target_address(&self) -> Result<Address>;

    async fn send_transaction(&self, tx: TypedTransaction) -> Result<Hash>;
}
