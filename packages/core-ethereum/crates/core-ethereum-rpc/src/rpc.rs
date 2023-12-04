use async_trait::async_trait;
use core_crypto::keypairs::{ChainKeypair, Keypair};
use core_ethereum_types::{ContractAddresses, ContractInstances};
use ethers::middleware::{MiddlewareBuilder, NonceManagerMiddleware, SignerMiddleware};
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::transaction::eip2718::TypedTransaction;
use ethers::signers::{LocalWallet, Signer, Wallet};
use ethers::types::{BlockId, NameOrAddress};
use ethers_providers::{JsonRpcClient, Middleware, Provider, RetryClient, RetryClientBuilder, RetryPolicy};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use utils_types::primitives::{Address, Balance, BalanceType, U256};
use validator::Validate;

use crate::errors::Result;
use crate::{HoprRpcOperations, PendingTransaction};

#[cfg(feature = "prometheus")]
use utils_metrics::metrics::SimpleCounter;

#[cfg(feature = "prometheus")]
lazy_static::lazy_static! {
     pub(crate) static ref METRIC_COUNT_RPC_CALLS: SimpleCounter = SimpleCounter::new(
        "core_ethereum_counter_rpc_calls",
        "Number of RPC calls"
    )
    .unwrap();
}

/// Configuration of the RPC related parameters.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct RpcOperationsConfig {
    /// Blockchain id
    /// Default is 100.
    pub chain_id: u64,
    /// Addresses of all deployed contracts
    /// Default contains empty (null) addresses.
    pub contract_addrs: ContractAddresses,
    /// Number of HTTP retries on retry-able failures
    /// Defaults to 5
    pub max_http_retries: u32,
    /// Expected block time of the blockchain
    /// Defaults to 5 seconds
    pub expected_block_time: Duration,
    /// Single log fetch chunk size
    /// Defaults to 50
    #[validate(range(min = 2))]
    pub logs_page_size: u64,
    /// Interval for polling on TX submission
    /// Defaults to 7 seconds.
    pub tx_polling_interval: Duration,
    /// Number of confirmations to wait when performing
    /// transaction polling.
    /// Defaults to 8
    #[validate(range(min = 1))]
    pub tx_confirmations: usize,
}

impl Default for RpcOperationsConfig {
    fn default() -> Self {
        Self {
            chain_id: 100,
            contract_addrs: Default::default(),
            max_http_retries: 5,
            logs_page_size: 50,
            expected_block_time: Duration::from_secs(5),
            tx_polling_interval: Duration::from_secs(7),
            tx_confirmations: 8,
        }
    }
}

pub(crate) type HoprMiddleware<P> =
    NonceManagerMiddleware<SignerMiddleware<Provider<RetryClient<P>>, Wallet<SigningKey>>>;

/// Implementation of `HoprRpcOperations` and `HoprIndexerRpcOperations` trait via `ethers`
#[derive(Debug, Clone)]
pub struct RpcOperations<P: JsonRpcClient + 'static> {
    pub(crate) provider: Arc<HoprMiddleware<P>>,
    pub(crate) cfg: RpcOperationsConfig,
    contract_instances: Arc<ContractInstances<HoprMiddleware<P>>>,
}

impl<P: JsonRpcClient + 'static> RpcOperations<P> {
    pub fn new<R>(json_rpc: P, chain_key: &ChainKeypair, cfg: RpcOperationsConfig, retry_policy: R) -> Result<Self>
    where
        R: RetryPolicy<<P as JsonRpcClient>::Error> + 'static,
    {
        let provider_client = RetryClientBuilder::default()
            .rate_limit_retries(cfg.max_http_retries)
            .timeout_retries(cfg.max_http_retries) // Note that this does not take effect when using SimpleJsonRetryPolicy
            .initial_backoff(Duration::from_millis(500))
            .build(json_rpc, Box::new(retry_policy));

        let wallet = LocalWallet::from_bytes(chain_key.secret().as_ref())?;
        let provider = Arc::new(
            Provider::new(provider_client)
                .interval(cfg.tx_polling_interval)
                .with_signer(wallet.with_chain_id(cfg.chain_id))
                .nonce_manager(chain_key.public().to_address().into()),
        );

        Ok(Self {
            contract_instances: Arc::new(ContractInstances::new(
                &cfg.contract_addrs,
                provider.clone(),
                cfg!(test),
            )),
            cfg,
            provider,
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<P: JsonRpcClient + 'static> HoprRpcOperations for RpcOperations<P> {
    async fn get_timestamp(&self, block_number: u64) -> Result<Option<u64>> {
        let ts = self
            .provider
            .get_block(BlockId::Number(block_number.into()))
            .await?
            .map(|b| b.timestamp.as_u64());

        #[cfg(feature = "prometheus")]
        METRIC_COUNT_RPC_CALLS.increment();

        Ok(ts)
    }

    async fn get_balance(&self, address: Address, balance_type: BalanceType) -> Result<Balance> {
        match balance_type {
            BalanceType::Native => {
                let native = self
                    .provider
                    .get_balance(NameOrAddress::Address(address.into()), None)
                    .await?;

                #[cfg(feature = "prometheus")]
                METRIC_COUNT_RPC_CALLS.increment();

                Ok(Balance::new(native.into(), BalanceType::Native))
            }
            BalanceType::HOPR => {
                let token_balance = self.contract_instances.token.balance_of(address.into()).call().await?;

                #[cfg(feature = "prometheus")]
                METRIC_COUNT_RPC_CALLS.increment();

                Ok(Balance::new(token_balance.into(), BalanceType::HOPR))
            }
        }
    }

    async fn get_node_management_module_target_info(&self, target: Address) -> Result<Option<U256>> {
        let (exists, target) = self
            .contract_instances
            .module_implementation
            .try_get_target(target.into())
            .call()
            .await?;

        #[cfg(feature = "prometheus")]
        METRIC_COUNT_RPC_CALLS.increment();

        Ok(exists.then_some(target.into()))
    }

    async fn get_safe_from_node_safe_registry(&self, node_address: Address) -> Result<Address> {
        let addr = self
            .contract_instances
            .safe_registry
            .node_to_safe(node_address.into())
            .call()
            .await?;

        #[cfg(feature = "prometheus")]
        METRIC_COUNT_RPC_CALLS.increment();

        Ok(addr.into())
    }

    async fn get_module_target_address(&self) -> Result<Address> {
        let owner = self.contract_instances.module_implementation.owner().call().await?;

        #[cfg(feature = "prometheus")]
        METRIC_COUNT_RPC_CALLS.increment();

        Ok(owner.into())
    }

    async fn get_channel_closure_notice_period(&self) -> Result<Duration> {
        // TODO: should we cache this value internally ?
        let notice_period = self
            .contract_instances
            .channels
            .notice_period_channel_closure()
            .call()
            .await?;

        #[cfg(feature = "prometheus")]
        METRIC_COUNT_RPC_CALLS.increment();

        Ok(Duration::from_secs(notice_period as u64))
    }

    async fn send_transaction(&self, tx: TypedTransaction) -> Result<PendingTransaction> {
        // Also fills the transaction including the EIP1559 fee estimates from the provider
        let sent_tx = self
            .provider
            .send_transaction(tx, None)
            .await?
            .confirmations(self.cfg.tx_confirmations)
            .interval(self.cfg.tx_polling_interval); // This is the default, but let's be explicit
                                                     // This has built-in max polling retries set to 3

        #[cfg(feature = "prometheus")]
        METRIC_COUNT_RPC_CALLS.increment();

        Ok(sent_tx.into())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::rpc::{RpcOperations, RpcOperationsConfig};
    use crate::{HoprRpcOperations, PendingTransaction, TypedTransaction};
    use bindings::hopr_token::HoprToken;
    use core_crypto::keypairs::{ChainKeypair, Keypair};
    use core_ethereum_types::{create_anvil, create_rpc_client_to_anvil, ContractAddresses, ContractInstances};
    use ethers::types::Eip1559TransactionRequest;
    use ethers_providers::Middleware;
    use primitive_types::H160;
    use std::future::IntoFuture;
    use std::time::Duration;
    use utils_types::primitives::{Address, BalanceType, U256};

    use crate::client::tests::ReqwestRequestor;
    use crate::client::{JsonRpcProviderClient, SimpleJsonRpcRetryPolicy};

    pub async fn mint_tokens<M: Middleware + 'static>(
        hopr_token: HoprToken<M>,
        amount: u128,
        deployer: Address,
    ) -> u64 {
        hopr_token
            .grant_role(hopr_token.minter_role().await.unwrap(), deployer.into())
            .send()
            .await
            .unwrap()
            .await
            .unwrap();

        hopr_token
            .mint(
                deployer.into(),
                amount.into(),
                ethers::types::Bytes::new(),
                ethers::types::Bytes::new(),
            )
            .send()
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap()
            .block_number
            .unwrap()
            .as_u64()
    }

    fn transfer_eth_tx(to: Address, amount: U256) -> TypedTransaction {
        let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());
        tx.set_to(H160::from(to));
        tx.set_value(ethers::types::U256(primitive_types::U256::from(amount).0));
        tx
    }

    pub async fn wait_until_tx(pending: PendingTransaction<'_>, timeout: Duration) {
        let tx_hash = pending.tx_hash();
        tokio::time::timeout(timeout, pending.into_future())
            .await
            .expect(&format!(
                "timeout awaiting tx hash {tx_hash} after {} seconds",
                timeout.as_secs()
            ))
            .expect("expected block");
    }

    #[tokio::test]
    async fn test_should_send_tx() {
        let _ = env_logger::builder().is_test(true).try_init();

        let anvil = create_anvil(Some(Duration::from_secs(1)));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            tx_confirmations: 2,
            ..RpcOperationsConfig::default()
        };

        let client = JsonRpcProviderClient::new(&anvil.endpoint(), ReqwestRequestor::default());

        let rpc =
            RpcOperations::new(client, &chain_key_0, cfg, SimpleJsonRpcRetryPolicy).expect("failed to construct rpc");

        let balance_1 = rpc
            .get_balance((&chain_key_0).into(), BalanceType::Native)
            .await
            .unwrap();
        assert!(balance_1.value().as_u64() > 0, "balance must be greater than 0");

        // Send 1 ETH to some random address
        let tx_hash = rpc
            .send_transaction(transfer_eth_tx(Address::random(), 1000000_u32.into()))
            .await
            .expect("failed to send tx");

        wait_until_tx(tx_hash, Duration::from_secs(8)).await;
    }

    #[tokio::test]
    async fn test_get_balance_native() {
        let _ = env_logger::builder().is_test(true).try_init();

        let anvil = create_anvil(Some(Duration::from_secs(1)));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            tx_confirmations: 2,
            ..RpcOperationsConfig::default()
        };

        let client = JsonRpcProviderClient::new(&anvil.endpoint(), ReqwestRequestor::default());
        let rpc =
            RpcOperations::new(client, &chain_key_0, cfg, SimpleJsonRpcRetryPolicy).expect("failed to construct rpc");

        let balance_1 = rpc
            .get_balance((&chain_key_0).into(), BalanceType::Native)
            .await
            .unwrap();
        assert!(balance_1.value().as_u64() > 0, "balance must be greater than 0");

        // Send 1 ETH to some random address
        let tx_hash = rpc
            .send_transaction(transfer_eth_tx(Address::random(), 1_u32.into()))
            .await
            .expect("failed to send tx");

        wait_until_tx(tx_hash, Duration::from_secs(8)).await;

        let balance_2 = rpc
            .get_balance((&chain_key_0).into(), BalanceType::Native)
            .await
            .unwrap();
        assert!(balance_2.lt(&balance_1), "balance must be diminished");
    }

    #[tokio::test]
    async fn test_get_balance_token() {
        let _ = env_logger::builder().is_test(true).try_init();

        let anvil = create_anvil(None);
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

        // Deploy contracts
        let contract_instances = {
            let client = create_rpc_client_to_anvil(&anvil, &chain_key_0);
            ContractInstances::deploy_for_testing(client, &chain_key_0)
                .await
                .expect("could not deploy contracts")
        };

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            contract_addrs: ContractAddresses::from(&contract_instances),
            ..RpcOperationsConfig::default()
        };

        let amount = 1024_u64;
        mint_tokens(
            contract_instances.token,
            amount as u128,
            chain_key_0.public().to_address(),
        )
        .await;

        let client = JsonRpcProviderClient::new(&anvil.endpoint(), ReqwestRequestor::default());
        let rpc =
            RpcOperations::new(client, &chain_key_0, cfg, SimpleJsonRpcRetryPolicy).expect("failed to construct rpc");

        let balance = rpc.get_balance((&chain_key_0).into(), BalanceType::HOPR).await.unwrap();
        assert_eq!(amount, balance.value().as_u64(), "invalid balance");
    }
}
