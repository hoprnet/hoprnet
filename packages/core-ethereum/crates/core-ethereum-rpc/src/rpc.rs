use async_trait::async_trait;
use core_crypto::keypairs::{ChainKeypair, Keypair};
use core_crypto::types::Hash;
use core_ethereum_types::{ContractAddresses, ContractInstances};
use ethers::middleware::{MiddlewareBuilder, NonceManagerMiddleware, SignerMiddleware};
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::transaction::eip2718::TypedTransaction;
use ethers::signers::{LocalWallet, Signer, Wallet};
use ethers::types::BlockId;
use ethers_providers::{JsonRpcClient, Middleware, Provider, RetryClient, RetryClientBuilder, RetryPolicy};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use utils_types::primitives::{Address, Balance, BalanceType, U256};
use validator::Validate;

use crate::errors::Result;
use crate::HoprRpcOperations;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct RpcOperationsConfig {
    pub chain_id: u64,
    pub contract_addrs: ContractAddresses,
    pub max_http_retries: u32,
    pub expected_block_time: Duration,
    pub logs_page_size: u64,
    pub tx_polling_interval: Duration,
}

impl Default for RpcOperationsConfig {
    fn default() -> Self {
        Self {
            chain_id: 100,
            contract_addrs: Default::default(),
            max_http_retries: 3,
            logs_page_size: 50,
            expected_block_time: Duration::from_secs(5),
            tx_polling_interval: Duration::from_secs(7),
        }
    }
}

pub(crate) type HoprMiddleware<P> =
    NonceManagerMiddleware<SignerMiddleware<Provider<RetryClient<P>>, Wallet<SigningKey>>>;

#[derive(Debug, Clone)]
pub struct RpcOperations<P: JsonRpcClient + 'static> {
    me: Address,
    pub(crate) provider: Arc<HoprMiddleware<P>>,
    pub(crate) cfg: RpcOperationsConfig,
    contract_instances: ContractInstances<HoprMiddleware<P>>,
}

impl<P: JsonRpcClient + 'static> RpcOperations<P> {
    pub fn new<R>(json_rpc: P, chain_key: &ChainKeypair, cfg: RpcOperationsConfig, retry_policy: R) -> Result<Self>
    where
        R: RetryPolicy<<P as JsonRpcClient>::Error> + 'static,
    {
        let provider_client = RetryClientBuilder::default()
            .rate_limit_retries(5)
            .timeout_retries(cfg.max_http_retries)
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
            me: chain_key.into(),
            contract_instances: ContractInstances::new(&cfg.contract_addrs, provider.clone(), cfg!(test)),
            cfg,
            provider,
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<P: JsonRpcClient + 'static> HoprRpcOperations for RpcOperations<P> {
    async fn get_timestamp(&self, block_number: u64) -> Result<Option<u64>> {
        Ok(self
            .provider
            .get_block(BlockId::Number(block_number.into()))
            .await?
            .map(|b| b.timestamp.as_u64()))
    }

    async fn get_balance(&self, balance_type: BalanceType) -> Result<Balance> {
        match balance_type {
            BalanceType::Native => {
                let addr: H160 = self.me.into();
                let native = self.provider.get_balance(addr, None).await?;
                Ok(Balance::new(native.into(), BalanceType::Native))
            }
            BalanceType::HOPR => {
                let token_balance = self.contract_instances.token.balance_of(self.me.into()).call().await?;
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
        Ok(exists.then_some(target.into()))
    }

    async fn get_safe_from_node_safe_registry(&self, node_address: Address) -> Result<Address> {
        let addr = self
            .contract_instances
            .safe_registry
            .node_to_safe(node_address.into())
            .call()
            .await?;
        Ok(addr.into())
    }

    async fn get_module_target_address(&self) -> Result<Address> {
        let owner = self.contract_instances.module_implementation.owner().call().await?;
        Ok(owner.into())
    }

    async fn send_transaction(&self, tx: TypedTransaction) -> Result<Hash> {
        // Also fills the transaction including the EIP1559 fee estimates from the provider
        let sent_tx = self.provider.send_transaction(tx, None).await?;
        Ok(sent_tx.0.into())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::rpc::{RpcOperations, RpcOperationsConfig};
    use crate::{HoprRpcOperations, TypedTransaction};
    use bindings::hopr_token::HoprToken;
    use core_crypto::keypairs::{ChainKeypair, Keypair};
    use core_crypto::types::Hash;
    use core_ethereum_types::{create_anvil, create_rpc_client_to_anvil, ContractAddresses, ContractInstances};
    use ethers::prelude::BlockId;
    use ethers::types::Eip1559TransactionRequest;
    use ethers_providers::{JsonRpcClient, Middleware};
    use futures::StreamExt;
    use primitive_types::{H160, H256};
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

    pub async fn wait_until_tx<P: JsonRpcClient + 'static>(
        tx_hash: Hash,
        rpc: &RpcOperations<P>,
        timeout: Duration,
    ) -> ethers::types::Block<H256> {
        let mut stream = rpc.provider.watch_blocks().await.unwrap();
        let prov_clone = rpc.provider.clone();

        tokio::time::timeout(timeout, async move {
            while let Some(hash) = stream.next().await {
                let block = prov_clone.get_block(BlockId::Hash(hash.into())).await.unwrap().unwrap();
                if block
                    .transactions
                    .iter()
                    .map(|tx| Hash::from(tx.0))
                    .any(|h| h.eq(&tx_hash))
                {
                    return Some(block);
                }
            }
            None
        })
        .await
        .expect(&format!(
            "timeout awaiting tx hash {tx_hash} after {} seconds",
            timeout.as_secs()
        ))
        .expect("expected block")
    }

    #[tokio::test]
    async fn test_should_send_tx() {
        let _ = env_logger::builder().is_test(true).try_init();

        let anvil = create_anvil(Some(Duration::from_secs(1)));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            ..RpcOperationsConfig::default()
        };

        let client = JsonRpcProviderClient::new(&anvil.endpoint(), ReqwestRequestor::default());

        let rpc =
            RpcOperations::new(client, &chain_key_0, cfg, SimpleJsonRpcRetryPolicy).expect("failed to construct rpc");

        let balance_1 = rpc.get_balance(BalanceType::Native).await.unwrap();
        assert!(balance_1.value().as_u64() > 0, "balance must be greater than 0");

        // Send 1 ETH to some random address
        let tx_hash = rpc
            .send_transaction(transfer_eth_tx(Address::random(), 1000000_u32.into()))
            .await
            .expect("failed to send tx");

        let _ = wait_until_tx(tx_hash, &rpc, Duration::from_secs(8)).await;
    }

    #[tokio::test]
    async fn test_get_balance_native() {
        let _ = env_logger::builder().is_test(true).try_init();

        let anvil = create_anvil(Some(Duration::from_secs(1)));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            ..RpcOperationsConfig::default()
        };

        let client = JsonRpcProviderClient::new(&anvil.endpoint(), ReqwestRequestor::default());
        let rpc =
            RpcOperations::new(client, &chain_key_0, cfg, SimpleJsonRpcRetryPolicy).expect("failed to construct rpc");

        let balance_1 = rpc.get_balance(BalanceType::Native).await.unwrap();
        assert!(balance_1.value().as_u64() > 0, "balance must be greater than 0");

        // Send 1 ETH to some random address
        let tx_hash = rpc
            .send_transaction(transfer_eth_tx(Address::random(), 1_u32.into()))
            .await
            .expect("failed to send tx");

        let _ = wait_until_tx(tx_hash, &rpc, Duration::from_secs(8)).await;

        let balance_2 = rpc.get_balance(BalanceType::Native).await.unwrap();
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

        let balance = rpc.get_balance(BalanceType::HOPR).await.unwrap();
        assert_eq!(amount, balance.value().as_u64(), "invalid balance");
    }
}
