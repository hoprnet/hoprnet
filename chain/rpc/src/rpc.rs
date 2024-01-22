use async_trait::async_trait;
use bindings::hopr_node_management_module::HoprNodeManagementModule;
use chain_types::{ContractAddresses, ContractInstances};
use ethers::middleware::{MiddlewareBuilder, NonceManagerMiddleware, SignerMiddleware};
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::transaction::eip2718::TypedTransaction;
use ethers::signers::{LocalWallet, Signer, Wallet};
use ethers::types::{BlockId, NameOrAddress};
use ethers_providers::{JsonRpcClient, Middleware, Provider};
use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
use hopr_primitive_types::prelude::*;
use log::debug;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use validator::Validate;

use crate::errors::Result;
use crate::{HoprRpcOperations, PendingTransaction};

/// Configuration of the RPC related parameters.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct RpcOperationsConfig {
    /// Blockchain id
    /// Default is 100.
    pub chain_id: u64,
    /// Addresses of all deployed contracts
    /// Default contains empty (null) addresses.
    pub contract_addrs: ContractAddresses,
    /// Address of the node's module.
    /// Defaults to null address.
    pub module_address: Address,
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
            module_address: Default::default(),
            logs_page_size: 2500,
            expected_block_time: Duration::from_secs(5),
            tx_polling_interval: Duration::from_secs(7),
            tx_confirmations: 8,
        }
    }
}

pub(crate) type HoprMiddleware<P> = NonceManagerMiddleware<SignerMiddleware<Provider<P>, Wallet<SigningKey>>>;

/// Implementation of `HoprRpcOperations` and `HoprIndexerRpcOperations` trait via `ethers`
#[derive(Debug)]
pub struct RpcOperations<P: JsonRpcClient + 'static> {
    pub(crate) provider: Arc<HoprMiddleware<P>>,
    pub(crate) cfg: RpcOperationsConfig,
    contract_instances: Arc<ContractInstances<HoprMiddleware<P>>>,
    node_module: HoprNodeManagementModule<HoprMiddleware<P>>,
}

// Needs manual impl not to impose Clone requirements on P
impl<P: JsonRpcClient> Clone for RpcOperations<P> {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider.clone(),
            cfg: self.cfg,
            contract_instances: self.contract_instances.clone(),
            node_module: HoprNodeManagementModule::new(self.cfg.module_address, self.provider.clone()),
        }
    }
}

impl<P: JsonRpcClient + 'static> RpcOperations<P> {
    pub fn new(json_rpc: P, chain_key: &ChainKeypair, cfg: RpcOperationsConfig) -> Result<Self> {
        let wallet = LocalWallet::from_bytes(chain_key.secret().as_ref())?.with_chain_id(cfg.chain_id);

        let provider = Arc::new(
            Provider::new(json_rpc)
                .interval(cfg.tx_polling_interval)
                .with_signer(wallet)
                .nonce_manager(chain_key.public().to_address().into()),
        );

        debug!("{:?}", cfg.contract_addrs);

        Ok(Self {
            contract_instances: Arc::new(ContractInstances::new(
                &cfg.contract_addrs,
                provider.clone(),
                cfg!(test),
            )),
            cfg,
            node_module: HoprNodeManagementModule::new(cfg.module_address, provider.clone()),
            provider,
        })
    }
}

#[async_trait]
impl<P: JsonRpcClient + 'static> HoprRpcOperations for RpcOperations<P> {
    async fn get_timestamp(&self, block_number: u64) -> Result<Option<u64>> {
        let ts = self
            .provider
            .get_block(BlockId::Number(block_number.into()))
            .await?
            .map(|b| b.timestamp.as_u64());

        Ok(ts)
    }

    async fn get_balance(&self, address: Address, balance_type: BalanceType) -> Result<Balance> {
        match balance_type {
            BalanceType::Native => {
                let native = self
                    .provider
                    .get_balance(NameOrAddress::Address(address.into()), None)
                    .await?;

                Ok(Balance::new(native, BalanceType::Native))
            }
            BalanceType::HOPR => {
                let token_balance = self.contract_instances.token.balance_of(address.into()).call().await?;

                Ok(Balance::new(token_balance, BalanceType::HOPR))
            }
        }
    }

    async fn get_node_management_module_target_info(&self, target: Address) -> Result<Option<U256>> {
        let (exists, target) = self.node_module.try_get_target(target.into()).call().await?;

        Ok(exists.then_some(target))
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
        let owner = self.node_module.owner().call().await?;
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

        Ok(Duration::from_secs(notice_period as u64))
    }

    async fn send_transaction(&self, tx: TypedTransaction) -> Result<PendingTransaction> {
        // This only sets the nonce on the first TX, otherwise it is a no-op
        let _ = self.provider.initialize_nonce(None).await;
        debug!("send outgoing tx: {:?}", tx);

        // Also fills the transaction including the EIP1559 fee estimates from the provider
        let sent_tx = self
            .provider
            .send_transaction(tx, None)
            .await?
            .confirmations(self.cfg.tx_confirmations)
            .interval(self.cfg.tx_polling_interval); // This is the default, but let's be explicit
                                                     // This has built-in max polling retries set to 3

        Ok(sent_tx.into())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::rpc::{RpcOperations, RpcOperationsConfig};
    use crate::{HoprRpcOperations, PendingTransaction};
    use async_std::task::sleep;
    use chain_types::{ContractAddresses, ContractInstances};
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
    use hopr_primitive_types::prelude::*;
    use std::time::Duration;

    use crate::client::native::SurfRequestor;
    use crate::client::{create_rpc_client_to_anvil, JsonRpcProviderClient, SimpleJsonRpcRetryPolicy};

    pub async fn wait_until_tx(pending: PendingTransaction<'_>, timeout: Duration) {
        let tx_hash = pending.tx_hash();
        sleep(timeout).await;
        pending
            .await
            .unwrap_or_else(|_| panic!("timeout awaiting tx hash {tx_hash} after {} seconds", timeout.as_secs()));
    }

    #[async_std::test]
    async fn test_should_send_tx() {
        let _ = env_logger::builder().is_test(true).try_init();

        let anvil = chain_types::utils::create_anvil(Some(Duration::from_secs(1)));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            tx_confirmations: 2,
            ..RpcOperationsConfig::default()
        };

        let client = JsonRpcProviderClient::new(
            &anvil.endpoint(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy::default(),
        );

        let rpc = RpcOperations::new(client, &chain_key_0, cfg).expect("failed to construct rpc");

        let balance_1 = rpc
            .get_balance((&chain_key_0).into(), BalanceType::Native)
            .await
            .unwrap();
        assert!(balance_1.amount().gt(&0.into()), "balance must be greater than 0");

        // Send 1 ETH to some random address
        let tx_hash = rpc
            .send_transaction(chain_types::utils::create_native_transfer(
                Address::random(),
                1000000_u32.into(),
            ))
            .await
            .expect("failed to send tx");

        wait_until_tx(tx_hash, Duration::from_secs(8)).await;
    }

    #[async_std::test]
    async fn test_should_send_consecutive_txs() {
        let _ = env_logger::builder().is_test(true).try_init();

        let anvil = chain_types::utils::create_anvil(Some(Duration::from_secs(1)));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            tx_confirmations: 2,
            ..RpcOperationsConfig::default()
        };

        let client = JsonRpcProviderClient::new(
            &anvil.endpoint(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy::default(),
        );

        let rpc = RpcOperations::new(client, &chain_key_0, cfg).expect("failed to construct rpc");

        let balance_1 = rpc
            .get_balance((&chain_key_0).into(), BalanceType::Native)
            .await
            .unwrap();
        assert!(balance_1.amount().gt(&0.into()), "balance must be greater than 0");

        let txs_count = 5_u64;
        let send_amount = 1000000_u64;

        // Send 1 ETH to some random address
        futures::future::join_all((0..txs_count).map(|_| async {
            rpc.send_transaction(chain_types::utils::create_native_transfer(
                Address::random(),
                send_amount.into(),
            ))
            .await
            .expect("tx should be sent")
            .await
            .expect("tx should resolve")
        }))
        .await;

        let balance_2 = rpc
            .get_balance((&chain_key_0).into(), BalanceType::Native)
            .await
            .unwrap();

        assert!(
            balance_2.amount() <= balance_1.amount() - txs_count * send_amount,
            "balance must be less"
        );
    }

    #[async_std::test]
    async fn test_get_balance_native() {
        let _ = env_logger::builder().is_test(true).try_init();

        let anvil = chain_types::utils::create_anvil(Some(Duration::from_secs(1)));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            tx_confirmations: 2,
            ..RpcOperationsConfig::default()
        };

        let client = JsonRpcProviderClient::new(
            &anvil.endpoint(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy::default(),
        );
        let rpc = RpcOperations::new(client, &chain_key_0, cfg).expect("failed to construct rpc");

        let balance_1 = rpc
            .get_balance((&chain_key_0).into(), BalanceType::Native)
            .await
            .unwrap();
        assert!(balance_1.amount().gt(&0.into()), "balance must be greater than 0");

        // Send 1 ETH to some random address
        let tx_hash = rpc
            .send_transaction(chain_types::utils::create_native_transfer(
                Address::random(),
                1_u32.into(),
            ))
            .await
            .expect("failed to send tx");

        wait_until_tx(tx_hash, Duration::from_secs(8)).await;

        let balance_2 = rpc
            .get_balance((&chain_key_0).into(), BalanceType::Native)
            .await
            .unwrap();
        assert!(balance_2.lt(&balance_1), "balance must be diminished");
    }

    #[async_std::test]
    async fn test_get_balance_token() {
        let _ = env_logger::builder().is_test(true).try_init();

        let anvil = chain_types::utils::create_anvil(None);
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

        // Deploy contracts
        let contract_instances = {
            let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &chain_key_0);
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
        chain_types::utils::mint_tokens(contract_instances.token, amount.into()).await;

        let client = JsonRpcProviderClient::new(
            &anvil.endpoint(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy::default(),
        );
        let rpc = RpcOperations::new(client, &chain_key_0, cfg).expect("failed to construct rpc");

        let balance = rpc.get_balance((&chain_key_0).into(), BalanceType::HOPR).await.unwrap();
        assert_eq!(amount, balance.amount().as_u64(), "invalid balance");
    }
}
