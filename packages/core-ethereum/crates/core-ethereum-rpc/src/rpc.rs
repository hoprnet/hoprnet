use async_trait::async_trait;
use core_crypto::keypairs::{ChainKeypair, Keypair};
use core_crypto::types::Hash;
use core_ethereum_misc::ContractAddresses;
use ethers::middleware::{MiddlewareBuilder, NonceManagerMiddleware, SignerMiddleware};
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::transaction::eip2718::TypedTransaction;
use ethers::signers::{LocalWallet, Signer, Wallet};
use ethers::types::BlockId;
use ethers_providers::{
    HttpRateLimitRetryPolicy, JsonRpcClient, Middleware, Provider, RetryClient, RetryClientBuilder, RetryPolicy,
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use utils_types::primitives::{Address, Balance, BalanceType, U256};
use validator::Validate;

use bindings::hopr_announcements::HoprAnnouncements;
use bindings::hopr_channels::HoprChannels;
use bindings::hopr_network_registry::HoprNetworkRegistry;
use bindings::hopr_node_management_module::HoprNodeManagementModule;
use bindings::hopr_node_safe_registry::HoprNodeSafeRegistry;
use bindings::hopr_token::HoprToken;

use crate::errors::Result;
use crate::HoprRpcOperations;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct RpcOperationsConfig {
    pub chain_id: u64,
    pub contract_addrs: ContractAddresses,
    pub node_module: Address,
    pub max_http_retries: u32,
    pub expected_block_time: Duration,
}

pub(crate) type HoprMiddleware<P> =
    NonceManagerMiddleware<SignerMiddleware<Provider<RetryClient<P>>, Wallet<SigningKey>>>;

pub struct RpcOperations<P: JsonRpcClient + 'static> {
    me: Address,
    pub(crate) provider: Arc<HoprMiddleware<P>>,
    pub(crate) cfg: RpcOperationsConfig,
    pub(crate) channels: HoprChannels<HoprMiddleware<P>>,
    announcements: HoprAnnouncements<HoprMiddleware<P>>,
    safe_registry: HoprNodeSafeRegistry<HoprMiddleware<P>>,
    network_registry: HoprNetworkRegistry<HoprMiddleware<P>>,
    node_module: HoprNodeManagementModule<HoprMiddleware<P>>,
    pub(crate) token: HoprToken<HoprMiddleware<P>>,
}

impl<P: JsonRpcClient + 'static> RpcOperations<P>
where
    HttpRateLimitRetryPolicy: RetryPolicy<<P as JsonRpcClient>::Error>,
{
    pub fn new(json_rpc: P, chain_key: &ChainKeypair, cfg: RpcOperationsConfig) -> Result<Self> {
        let provider_client = RetryClientBuilder::default()
            .rate_limit_retries(5)
            .timeout_retries(cfg.max_http_retries)
            .initial_backoff(Duration::from_millis(500))
            .build(json_rpc, Box::<HttpRateLimitRetryPolicy>::default());

        let wallet = LocalWallet::from_bytes(chain_key.secret().as_ref())?;
        let provider = Arc::new(
            Provider::new(provider_client)
                .with_signer(wallet.with_chain_id(cfg.chain_id))
                .nonce_manager(chain_key.public().to_address().into()),
        );

        Ok(Self {
            me: chain_key.public().to_address(),
            channels: HoprChannels::new::<H160>(cfg.contract_addrs.channels.into(), provider.clone()),
            announcements: HoprAnnouncements::new::<H160>(cfg.contract_addrs.announcements.into(), provider.clone()),
            safe_registry: HoprNodeSafeRegistry::new::<H160>(cfg.contract_addrs.safe_registry.into(), provider.clone()),
            network_registry: HoprNetworkRegistry::new::<H160>(
                cfg.contract_addrs.network_registry.into(),
                provider.clone(),
            ),
            node_module: HoprNodeManagementModule::new::<H160>(cfg.node_module.into(), provider.clone()),
            token: HoprToken::new::<H160>(cfg.contract_addrs.token.into(), provider.clone()),
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
                let token_balance = self.token.balance_of(self.me.into()).call().await?;
                Ok(Balance::new(token_balance.into(), BalanceType::HOPR))
            }
        }
    }

    async fn get_node_management_module_target_info(&self, target: Address) -> Result<Option<U256>> {
        let (exists, target) = self.node_module.try_get_target(target.into()).call().await?;
        Ok(exists.then_some(target.into()))
    }

    async fn get_safe_from_node_safe_registry(&self, node_address: Address) -> Result<Address> {
        let addr = self.safe_registry.node_to_safe(node_address.into()).call().await?;
        Ok(addr.into())
    }

    async fn get_module_target_address(&self) -> Result<Address> {
        let owner = self.node_module.owner().call().await?;
        Ok(owner.into())
    }

    async fn send_transaction(&self, tx: TypedTransaction) -> Result<Hash> {
        let sent_tx = self.provider.send_transaction(tx, None).await?;
        Ok(sent_tx.0.into())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::rpc::{RpcOperations, RpcOperationsConfig};
    use crate::{Block, HoprRpcOperations, TypedTransaction};
    use core_crypto::keypairs::{ChainKeypair, Keypair};
    use core_crypto::types::Hash;
    use core_ethereum_misc::ContractAddresses;
    use ethers::core::k256::ecdsa::SigningKey;
    use ethers::middleware::SignerMiddleware;
    use ethers::prelude::BlockId;
    use ethers::signers::{LocalWallet, Signer, Wallet};
    use ethers::types::Eip1559TransactionRequest;
    use ethers::utils::{Anvil, AnvilInstance};
    use ethers_providers::{Http, JsonRpcClient, Middleware, Provider};
    use futures::future::Either;
    use futures::StreamExt;
    use primitive_types::H160;
    use std::path::PathBuf;
    use std::pin::pin;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Duration;
    use utils_types::primitives::{Address, BalanceType, U256};

    pub fn anvil_provider() -> (
        Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
        ChainKeypair,
        AnvilInstance,
    ) {
        let anvil: AnvilInstance = Anvil::new()
            .path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../.foundry/bin/anvil"))
            .block_time(1_u64)
            .spawn();

        let wallet: LocalWallet = anvil.keys()[0].clone().into();

        let provider = Provider::<Http>::try_from(anvil.endpoint())
            .unwrap()
            .interval(Duration::from_millis(10u64));

        let client = SignerMiddleware::new(provider, wallet.with_chain_id(anvil.chain_id()));

        (
            Arc::new(client),
            ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap(),
            anvil,
        )
    }

    pub fn mock_config() -> RpcOperationsConfig {
        RpcOperationsConfig {
            chain_id: 31337, // Anvil default chain id
            contract_addrs: ContractAddresses {
                channels: Address::random(),
                announcements: Address::random(),
                token: Address::random(),
                safe_registry: Address::random(),
                network_registry: Address::random(),
                price_oracle: Address::random(),
            },
            node_module: Address::random(),
            max_http_retries: 5,
            expected_block_time: Duration::from_secs(1),
        }
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
    ) -> Block {
        let mut stream = rpc.provider.watch_blocks().await.unwrap();

        let prov_clone = rpc.provider.clone();

        let timeout_fut = pin!(tokio::time::sleep(timeout));
        let block_fut = pin!(async move {
            while let Some(hash) = stream.next().await {
                let block = prov_clone.get_block(BlockId::Hash(hash.into())).await.unwrap().unwrap();
                if block
                    .transactions
                    .iter()
                    .map(|tx| Hash::from(tx.0))
                    .any(|h| h.eq(&tx_hash))
                {
                    return Some(Block::from(block));
                }
            }
            None
        });

        match futures::future::select(block_fut, timeout_fut).await {
            Either::Left((block, _)) => block.expect("expected a block"),
            Either::Right(_) => panic!("timeout awaiting tx hash {tx_hash} after {} seconds", timeout.as_secs()),
        }
    }

    #[tokio::test]
    async fn test_should_send_tx() {
        let _ = env_logger::builder().is_test(true).try_init();

        let (_, chain_key, anvil) = anvil_provider();

        let cfg = mock_config();
        let rpc = RpcOperations::new(Http::from_str(&anvil.endpoint()).unwrap(), &chain_key, cfg)
            .expect("failed to construct rpc");

        let balance_1 = rpc.get_balance(BalanceType::Native).await.unwrap();
        assert!(balance_1.value().as_u64() > 0, "balance must be greater than 0");

        // Send 1 ETH to some random address
        let tx_hash = rpc
            .send_transaction(transfer_eth_tx(Address::random(), 1_u32.into()))
            .await
            .expect("failed to send tx");

        let _ = wait_until_tx(tx_hash, &rpc, Duration::from_secs(8)).await;
    }

    #[tokio::test]
    async fn test_get_balance_native() {
        let _ = env_logger::builder().is_test(true).try_init();

        let (_, chain_key, anvil) = anvil_provider();

        let cfg = mock_config();
        let rpc = RpcOperations::new(Http::from_str(&anvil.endpoint()).unwrap(), &chain_key, cfg)
            .expect("failed to construct rpc");

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
}
