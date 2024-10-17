//! General purpose high-level RPC operations implementation (`HoprRpcOperations`).
//!
//! The purpose of this module is to give implementation of the [HoprRpcOperations] trait:
//! [RpcOperations] type, which is the main API exposed by this crate.
use async_trait::async_trait;
use bindings::hopr_node_management_module::HoprNodeManagementModule;
use chain_types::{ContractAddresses, ContractInstances};
use ethers::contract::{abigen, Multicall, MULTICALL_ADDRESS};
use ethers::middleware::{MiddlewareBuilder, NonceManagerMiddleware, SignerMiddleware};
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::transaction::eip2718::TypedTransaction;
use ethers::providers::{JsonRpcClient, Middleware, Provider};
use ethers::signers::{LocalWallet, Signer, Wallet};
use ethers::types::{BlockId, NameOrAddress};
use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
use hopr_primitive_types::prelude::*;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;
use validator::Validate;

use crate::errors::RpcError::ContractError;
use crate::errors::{Result, RpcError};
use crate::{HoprRpcOperations, NodeSafeModuleStatus, PendingTransaction};

// define basic safe abi
abigen!(
    SafeSingleton,
    r#"[
        function isModuleEnabled(address module) public view returns (bool)
    ]"#,
);

/// Configuration of the RPC related parameters.
#[derive(Clone, Debug, Copy, PartialEq, Eq, smart_default::SmartDefault, Serialize, Deserialize, Validate)]
pub struct RpcOperationsConfig {
    /// Blockchain id
    ///
    /// Default is 100.
    #[default = 100]
    pub chain_id: u64,
    /// Addresses of all deployed contracts
    ///
    /// Default contains empty (null) addresses.
    pub contract_addrs: ContractAddresses,
    /// Address of the node's module.
    ///
    /// Defaults to null address.
    pub module_address: Address,
    /// Address of the node's safe contract.
    ///
    /// Defaults to null address.
    pub safe_address: Address,
    /// Expected block time of the blockchain
    ///
    /// Defaults to 5 seconds
    #[default(Duration::from_secs(5))]
    pub expected_block_time: Duration,
    /// The largest amount of blocks to fetch at once when fetching a range of blocks.
    ///
    /// If the requested block range size is N, then the client will always fetch `min(N, max_block_range_fetch_size)`
    ///
    /// Defaults to 2000 blocks
    #[validate(range(min = 1))]
    #[default = 2000]
    pub max_block_range_fetch_size: u64,
    /// Interval for polling on TX submission
    ///
    /// Defaults to 7 seconds.
    #[default(Duration::from_secs(7))]
    pub tx_polling_interval: Duration,
    /// Finalization chain length
    ///
    /// The number of blocks including and decreasing from the chain HEAD
    /// that the logs will be buffered for before being considered
    /// successfully joined to the chain.
    ///
    /// Defaults to 8
    #[validate(range(min = 1, max = 100))]
    #[default = 8]
    pub finality: u32,
}

pub(crate) type HoprMiddleware<P> = NonceManagerMiddleware<SignerMiddleware<Provider<P>, Wallet<SigningKey>>>;

/// Implementation of `HoprRpcOperations` and `HoprIndexerRpcOperations` trait via `ethers`
#[derive(Debug)]
pub struct RpcOperations<P: JsonRpcClient + 'static> {
    pub(crate) provider: Arc<HoprMiddleware<P>>,
    pub(crate) cfg: RpcOperationsConfig,
    contract_instances: Arc<ContractInstances<HoprMiddleware<P>>>,
    node_module: HoprNodeManagementModule<HoprMiddleware<P>>,
    node_safe: SafeSingleton<HoprMiddleware<P>>,
}

// Needs manual impl not to impose Clone requirements on P
impl<P: JsonRpcClient> Clone for RpcOperations<P> {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider.clone(),
            cfg: self.cfg,
            contract_instances: self.contract_instances.clone(),
            node_module: HoprNodeManagementModule::new(self.cfg.module_address, self.provider.clone()),
            node_safe: SafeSingleton::new(self.cfg.safe_address, self.provider.clone()),
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
            node_safe: SafeSingleton::new(cfg.safe_address, provider.clone()),
            provider,
        })
    }

    pub(crate) async fn get_block_number(&self) -> Result<u64> {
        Ok(self
            .provider
            .get_block_number()
            .await?
            .as_u64()
            .saturating_sub(self.cfg.finality as u64))
    }

    pub(crate) async fn get_block(
        &self,
        block_number: u64,
    ) -> Result<Option<ethers::types::Block<ethers::types::H256>>> {
        let sanitized_block_number = block_number.saturating_sub(self.cfg.finality as u64);
        Ok(self
            .provider
            .get_block(BlockId::Number(sanitized_block_number.into()))
            .await?)
    }
}

#[async_trait]
impl<P: JsonRpcClient + 'static> HoprRpcOperations for RpcOperations<P> {
    async fn get_timestamp(&self, block_number: u64) -> Result<Option<u64>> {
        Ok(self.get_block(block_number).await?.map(|b| b.timestamp.as_u64()))
    }

    async fn get_balance(&self, address: Address, balance_type: BalanceType) -> Result<Balance> {
        match balance_type {
            BalanceType::Native => {
                let native = self
                    .provider
                    .get_balance(
                        NameOrAddress::Address(address.into()),
                        Some(BlockId::Number(self.get_block_number().await?.into())),
                    )
                    .await?;

                Ok(Balance::new(native, BalanceType::Native))
            }
            BalanceType::HOPR => match self.contract_instances.token.balance_of(address.into()).call().await {
                Ok(token_balance) => Ok(Balance::new(token_balance, BalanceType::HOPR)),
                Err(e) => Err(ContractError(
                    "HoprToken".to_string(),
                    "balance_of".to_string(),
                    e.to_string(),
                )),
            },
        }
    }

    async fn get_eligibility_status(&self, address: Address) -> Result<bool> {
        match self
            .contract_instances
            .network_registry
            .is_node_registered_and_eligible(address.into())
            .call()
            .await
        {
            Ok(eligible) => Ok(eligible),
            Err(e) => Err(ContractError(
                "NetworkRegistry".to_string(),
                "is_node_registered_and_eligible".to_string(),
                e.to_string(),
            )),
        }
    }

    async fn get_node_management_module_target_info(&self, target: Address) -> Result<Option<U256>> {
        match self.node_module.try_get_target(target.into()).call().await {
            Ok((exists, target)) => Ok(exists.then_some(target)),
            Err(e) => Err(ContractError(
                "NodeModule".to_string(),
                "try_get_target".to_string(),
                e.to_string(),
            )),
        }
    }

    async fn get_safe_from_node_safe_registry(&self, node_address: Address) -> Result<Address> {
        match self
            .contract_instances
            .safe_registry
            .node_to_safe(node_address.into())
            .call()
            .await
        {
            Ok(addr) => Ok(addr.into()),
            Err(e) => Err(ContractError(
                "SafeRegistry".to_string(),
                "node_to_safe".to_string(),
                e.to_string(),
            )),
        }
    }

    async fn get_module_target_address(&self) -> Result<Address> {
        match self.node_module.owner().call().await {
            Ok(owner) => Ok(owner.into()),
            Err(e) => Err(ContractError(
                "NodeModule".to_string(),
                "owner".to_string(),
                e.to_string(),
            )),
        }
    }

    async fn get_channel_closure_notice_period(&self) -> Result<Duration> {
        // TODO: should we cache this value internally ?
        match self
            .contract_instances
            .channels
            .notice_period_channel_closure()
            .call()
            .await
        {
            Ok(notice_period) => Ok(Duration::from_secs(notice_period as u64)),
            Err(e) => Err(ContractError(
                "HoprChannels".to_string(),
                "notice_period_channel_closure".to_string(),
                e.to_string(),
            )),
        }
    }

    // Check on-chain status of, node, safe, and module
    async fn check_node_safe_module_status(&self, node_address: Address) -> Result<NodeSafeModuleStatus> {
        let mut multicall = Multicall::new(self.provider.clone(), Some(MULTICALL_ADDRESS))
            .await
            .map_err(|e| RpcError::MulticallError(e.to_string()))?;

        multicall
            // 1) check if the node is already included into the module
            .add_call(
                self.node_module
                    .method::<H160, bool>("isNode", node_address.into())
                    .map_err(|e| {
                        RpcError::MulticallError(format!(
                            "Error in checking isNode function in check_node_safe_module_status, due to {e}"
                        ))
                    })?,
                false,
            )
            // 2) if the module is enabled in the safe
            .add_call(
                self.node_safe
                    .method::<H160, bool>("isModuleEnabled", self.cfg.module_address.into())
                    .map_err(|e| {
                        RpcError::MulticallError(format!(
                            "Error in checking isModuleEnabled function in check_node_safe_module_status, due to {e}"
                        ))
                    })?,
                false,
            )
            // 3) if the safe is the owner of the module
            .add_call(
                self.node_module.method::<_, H160>("owner", ()).map_err(|e| {
                    RpcError::MulticallError(format!(
                        "Error in checking owner function in check_node_safe_module_status, due to {e}"
                    ))
                })?,
                false,
            );

        let results: (bool, bool, H160) = multicall.call().await.map_err(|e| {
            RpcError::MulticallError(format!(
                "Error in getting result from check_node_safe_module_status, due to {e}"
            ))
        })?;
        Ok(NodeSafeModuleStatus {
            is_node_included_in_module: results.0,
            is_module_enabled_in_safe: results.1,
            is_safe_owner_of_module: results.2 == self.cfg.safe_address.into(),
        })
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
            .confirmations(self.cfg.finality as usize)
            .interval(self.cfg.tx_polling_interval);

        Ok(sent_tx.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::rpc::{RpcOperations, RpcOperationsConfig};
    use crate::{HoprRpcOperations, PendingTransaction};
    use chain_types::{ContractAddresses, ContractInstances};
    use ethers::contract::ContractError;
    use ethers::providers::Middleware;
    use ethers::types::{Bytes, Eip1559TransactionRequest};
    use hex_literal::hex;
    use primitive_types::H160;
    use std::sync::Arc;
    use std::time::Duration;

    use hopr_async_runtime::prelude::sleep;
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
    use hopr_primitive_types::prelude::*;
    use std::str::FromStr;

    use crate::client::surf_client::SurfRequestor;
    use crate::client::{create_rpc_client_to_anvil, JsonRpcProviderClient, SimpleJsonRpcRetryPolicy};

    lazy_static::lazy_static! {
        static ref RANDY: Address = hex!("762614a5ed652457a2f1cdb8006380530c26ae6a").into();
        static ref MULTICALL3_DEPLOY_CODE: [u8; 3926] = hex!("f90f538085174876e800830f42408080b90f00608060405234801561001057600080fd5b50610ee0806100206000396000f3fe6080604052600436106100f35760003560e01c80634d2301cc1161008a578063a8b0574e11610059578063a8b0574e1461025a578063bce38bd714610275578063c3077fa914610288578063ee82ac5e1461029b57600080fd5b80634d2301cc146101ec57806372425d9d1461022157806382ad56cb1461023457806386d516e81461024757600080fd5b80633408e470116100c65780633408e47014610191578063399542e9146101a45780633e64a696146101c657806342cbb15c146101d957600080fd5b80630f28c97d146100f8578063174dea711461011a578063252dba421461013a57806327e86d6e1461015b575b600080fd5b34801561010457600080fd5b50425b6040519081526020015b60405180910390f35b61012d610128366004610a85565b6102ba565b6040516101119190610bbe565b61014d610148366004610a85565b6104ef565b604051610111929190610bd8565b34801561016757600080fd5b50437fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0140610107565b34801561019d57600080fd5b5046610107565b6101b76101b2366004610c60565b610690565b60405161011193929190610cba565b3480156101d257600080fd5b5048610107565b3480156101e557600080fd5b5043610107565b3480156101f857600080fd5b50610107610207366004610ce2565b73ffffffffffffffffffffffffffffffffffffffff163190565b34801561022d57600080fd5b5044610107565b61012d610242366004610a85565b6106ab565b34801561025357600080fd5b5045610107565b34801561026657600080fd5b50604051418152602001610111565b61012d610283366004610c60565b61085a565b6101b7610296366004610a85565b610a1a565b3480156102a757600080fd5b506101076102b6366004610d18565b4090565b60606000828067ffffffffffffffff8111156102d8576102d8610d31565b60405190808252806020026020018201604052801561031e57816020015b6040805180820190915260008152606060208201528152602001906001900390816102f65790505b5092503660005b8281101561047757600085828151811061034157610341610d60565b6020026020010151905087878381811061035d5761035d610d60565b905060200281019061036f9190610d8f565b6040810135958601959093506103886020850185610ce2565b73ffffffffffffffffffffffffffffffffffffffff16816103ac6060870187610dcd565b6040516103ba929190610e32565b60006040518083038185875af1925050503d80600081146103f7576040519150601f19603f3d011682016040523d82523d6000602084013e6103fc565b606091505b50602080850191909152901515808452908501351761046d577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260846000fd5b5050600101610325565b508234146104e6576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601a60248201527f4d756c746963616c6c333a2076616c7565206d69736d6174636800000000000060448201526064015b60405180910390fd5b50505092915050565b436060828067ffffffffffffffff81111561050c5761050c610d31565b60405190808252806020026020018201604052801561053f57816020015b606081526020019060019003908161052a5790505b5091503660005b8281101561068657600087878381811061056257610562610d60565b90506020028101906105749190610e42565b92506105836020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166105a66020850185610dcd565b6040516105b4929190610e32565b6000604051808303816000865af19150503d80600081146105f1576040519150601f19603f3d011682016040523d82523d6000602084013e6105f6565b606091505b5086848151811061060957610609610d60565b602090810291909101015290508061067d576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b50600101610546565b5050509250929050565b43804060606106a086868661085a565b905093509350939050565b6060818067ffffffffffffffff8111156106c7576106c7610d31565b60405190808252806020026020018201604052801561070d57816020015b6040805180820190915260008152606060208201528152602001906001900390816106e55790505b5091503660005b828110156104e657600084828151811061073057610730610d60565b6020026020010151905086868381811061074c5761074c610d60565b905060200281019061075e9190610e76565b925061076d6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166107906040850185610dcd565b60405161079e929190610e32565b6000604051808303816000865af19150503d80600081146107db576040519150601f19603f3d011682016040523d82523d6000602084013e6107e0565b606091505b506020808401919091529015158083529084013517610851577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260646000fd5b50600101610714565b6060818067ffffffffffffffff81111561087657610876610d31565b6040519080825280602002602001820160405280156108bc57816020015b6040805180820190915260008152606060208201528152602001906001900390816108945790505b5091503660005b82811015610a105760008482815181106108df576108df610d60565b602002602001015190508686838181106108fb576108fb610d60565b905060200281019061090d9190610e42565b925061091c6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff1661093f6020850185610dcd565b60405161094d929190610e32565b6000604051808303816000865af19150503d806000811461098a576040519150601f19603f3d011682016040523d82523d6000602084013e61098f565b606091505b506020830152151581528715610a07578051610a07576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b506001016108c3565b5050509392505050565b6000806060610a2b60018686610690565b919790965090945092505050565b60008083601f840112610a4b57600080fd5b50813567ffffffffffffffff811115610a6357600080fd5b6020830191508360208260051b8501011115610a7e57600080fd5b9250929050565b60008060208385031215610a9857600080fd5b823567ffffffffffffffff811115610aaf57600080fd5b610abb85828601610a39565b90969095509350505050565b6000815180845260005b81811015610aed57602081850181015186830182015201610ad1565b81811115610aff576000602083870101525b50601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0169290920160200192915050565b600082825180855260208086019550808260051b84010181860160005b84811015610bb1578583037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe001895281518051151584528401516040858501819052610b9d81860183610ac7565b9a86019a9450505090830190600101610b4f565b5090979650505050505050565b602081526000610bd16020830184610b32565b9392505050565b600060408201848352602060408185015281855180845260608601915060608160051b870101935082870160005b82811015610c52577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa0888703018452610c40868351610ac7565b95509284019290840190600101610c06565b509398975050505050505050565b600080600060408486031215610c7557600080fd5b83358015158114610c8557600080fd5b9250602084013567ffffffffffffffff811115610ca157600080fd5b610cad86828701610a39565b9497909650939450505050565b838152826020820152606060408201526000610cd96060830184610b32565b95945050505050565b600060208284031215610cf457600080fd5b813573ffffffffffffffffffffffffffffffffffffffff81168114610bd157600080fd5b600060208284031215610d2a57600080fd5b5035919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81833603018112610dc357600080fd5b9190910192915050565b60008083357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe1843603018112610e0257600080fd5b83018035915067ffffffffffffffff821115610e1d57600080fd5b602001915036819003821315610a7e57600080fd5b8183823760009101908152919050565b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc1833603018112610dc357600080fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa1833603018112610dc357600080fdfea2646970667358221220bb2b5c71a328032f97c676ae39a1ec2148d3e5d6f73d95e9b17910152d61f16264736f6c634300080c00331ca0edce47092c0f398cebf3ffc267f05c8e7076e3b89445e0fe50f6332273d4569ba01b0b9d000e19b24c5869b0fc3b22b0d6fa47cd63316875cbbd577d76e6fde086");
    }
    // lazy_static::lazy_static! {
    // }

    pub const MULTICALL3_DEPLOYER: &str = "05f32b3cc3888453ff71b01135b34ff8e41263f2";
    pub const ETH_VALUE_FOR_MULTICALL3_DEPLOYER: u128 = 100_000_000_000_000_000; // 0.1 (anvil) ETH

    pub async fn wait_until_tx(pending: PendingTransaction<'_>, timeout: Duration) {
        let tx_hash = pending.tx_hash();
        sleep(timeout).await;
        pending
            .await
            .unwrap_or_else(|_| panic!("timeout awaiting tx hash {tx_hash} after {} seconds", timeout.as_secs()));
    }

    /// Deploy a MULTICALL contract into Anvil local chain for testing
    pub async fn deploy_multicall3_to_anvil<M: Middleware>(provider: Arc<M>) -> Result<(), ContractError<M>> {
        // Fund Multicall3 deployer and deploy ERC1820Registry
        let mut tx = Eip1559TransactionRequest::new();
        tx = tx.to(H160::from_str(MULTICALL3_DEPLOYER).unwrap());
        tx = tx.value(ETH_VALUE_FOR_MULTICALL3_DEPLOYER);

        provider
            .send_transaction(tx, None)
            .await
            .map_err(|e| ContractError::MiddlewareError { e })?
            .await?;

        provider
            .send_raw_transaction(Bytes::from_static(&*MULTICALL3_DEPLOY_CODE))
            .await
            .map_err(|e| ContractError::MiddlewareError { e })?
            .await?;
        Ok(())
    }

    #[async_std::test]
    async fn test_should_send_tx() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);
        let anvil = chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            expected_block_time,
            finality: 2,
            ..RpcOperationsConfig::default()
        };

        let client = JsonRpcProviderClient::new(
            &anvil.endpoint(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy::default(),
        );

        // Wait until contracts deployments are final
        sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(client, &chain_key_0, cfg)?;

        let balance_1 = rpc.get_balance((&chain_key_0).into(), BalanceType::Native).await?;
        assert!(balance_1.amount().gt(&0.into()), "balance must be greater than 0");

        // Send 1 ETH to some random address
        let tx_hash = rpc
            .send_transaction(chain_types::utils::create_native_transfer(*RANDY, 1000000_u32.into()))
            .await?;

        wait_until_tx(tx_hash, Duration::from_secs(8)).await;

        Ok(())
    }

    #[async_std::test]
    async fn test_should_send_consecutive_txs() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);
        let anvil = chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            expected_block_time,
            finality: 2,
            ..RpcOperationsConfig::default()
        };

        let client = JsonRpcProviderClient::new(
            &anvil.endpoint(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy::default(),
        );

        // Wait until contracts deployments are final
        sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(client, &chain_key_0, cfg)?;

        let balance_1 = rpc.get_balance((&chain_key_0).into(), BalanceType::Native).await?;
        assert!(balance_1.amount().gt(&0.into()), "balance must be greater than 0");

        let txs_count = 5_u64;
        let send_amount = 1000000_u64;

        // Send 1 ETH to some random address
        futures::future::join_all((0..txs_count).map(|_| async {
            rpc.send_transaction(chain_types::utils::create_native_transfer(*RANDY, send_amount.into()))
                .await
                .expect("tx should be sent")
                .await
                .expect("tx should resolve")
        }))
        .await;

        let balance_2 = rpc.get_balance((&chain_key_0).into(), BalanceType::Native).await?;

        assert!(
            balance_2.amount() <= balance_1.amount() - txs_count * send_amount,
            "balance must be less"
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_get_balance_native() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);
        let anvil = chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            expected_block_time,
            finality: 2,
            ..RpcOperationsConfig::default()
        };

        let client = JsonRpcProviderClient::new(
            &anvil.endpoint(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy::default(),
        );

        // Wait until contracts deployments are final
        sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(client, &chain_key_0, cfg)?;

        let balance_1 = rpc.get_balance((&chain_key_0).into(), BalanceType::Native).await?;
        assert!(balance_1.amount().gt(&0.into()), "balance must be greater than 0");

        // Send 1 ETH to some random address
        let tx_hash = rpc
            .send_transaction(chain_types::utils::create_native_transfer(*RANDY, 1_u32.into()))
            .await?;

        wait_until_tx(tx_hash, Duration::from_secs(8)).await;

        let balance_2 = rpc.get_balance((&chain_key_0).into(), BalanceType::Native).await?;
        assert!(balance_2.lt(&balance_1), "balance must be diminished");

        Ok(())
    }

    #[async_std::test]
    async fn test_get_balance_token() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);
        let anvil = chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;

        // Deploy contracts
        let contract_instances = {
            let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &chain_key_0);
            ContractInstances::deploy_for_testing(client, &chain_key_0).await?
        };

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            expected_block_time,
            finality: 2,
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

        // Wait until contracts deployments are final
        sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(client, &chain_key_0, cfg)?;

        let balance = rpc.get_balance((&chain_key_0).into(), BalanceType::HOPR).await?;
        assert_eq!(amount, balance.amount().as_u64(), "invalid balance");

        Ok(())
    }

    #[async_std::test]
    async fn test_check_node_safe_module_status() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);
        let anvil = chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;

        // Deploy contracts
        let (contract_instances, module, safe) = {
            let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &chain_key_0);
            let instances = ContractInstances::deploy_for_testing(client.clone(), &chain_key_0).await?;

            // deploy MULTICALL contract to anvil
            deploy_multicall3_to_anvil(client.clone()).await?;

            let (module, safe) = chain_types::utils::deploy_one_safe_one_module_and_setup_for_testing(
                &instances,
                client.clone(),
                &chain_key_0,
            )
            .await?;

            // deploy a module and safe instance and add node into the module. The module is enabled by default in the safe
            (instances, module, safe)
        };

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            expected_block_time,
            finality: 2,
            contract_addrs: ContractAddresses::from(&contract_instances),
            module_address: module,
            safe_address: safe,
            ..RpcOperationsConfig::default()
        };

        let client = JsonRpcProviderClient::new(
            &anvil.endpoint(),
            SurfRequestor::default(),
            SimpleJsonRpcRetryPolicy::default(),
        );

        // Wait until contracts deployments are final
        sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(client.clone(), &chain_key_0, cfg)?;

        let result_before_including_node = rpc.check_node_safe_module_status((&chain_key_0).into()).await?;
        // before including node to the safe and module, only the first chck is false, the others are true
        assert!(
            !result_before_including_node.is_node_included_in_module,
            "node should not be included in a default module"
        );
        assert!(
            result_before_including_node.is_module_enabled_in_safe,
            "module should be enabled in a default safe"
        );
        assert!(
            result_before_including_node.is_safe_owner_of_module,
            "safe should not be the owner of a default module"
        );

        // including node to the module
        chain_types::utils::include_node_to_module_by_safe(
            contract_instances.channels.client().clone(),
            safe,
            module,
            (&chain_key_0).into(),
            &chain_key_0,
        )
        .await?;

        let result_with_node_included = rpc.check_node_safe_module_status((&chain_key_0).into()).await?;
        // after the node gets included into the module, all checks should be true
        assert!(
            result_with_node_included.is_node_included_in_module,
            "node should be included in a default module"
        );
        assert!(
            result_with_node_included.is_module_enabled_in_safe,
            "module should be enabled in a default safe"
        );
        assert!(
            result_with_node_included.is_safe_owner_of_module,
            "safe should be the owner of a default module"
        );

        Ok(())
    }
}
