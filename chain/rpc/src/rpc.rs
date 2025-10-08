//! General purpose high-level RPC operations implementation (`HoprRpcOperations`).
//!
//! The purpose of this module is to give implementation of the [HoprRpcOperations] trait:
//! [RpcOperations] type, which is the main API exposed by this crate.
use std::{sync::Arc, time::Duration};

use SafeSingleton::SafeSingletonInstance;
use alloy::{
    network::EthereumWallet,
    providers::{
        CallItemBuilder, Identity, PendingTransaction, Provider, ProviderBuilder, RootProvider,
        fillers::{
            BlobGasFiller, CachedNonceManager, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
            WalletFiller,
        },
    },
    rpc::{
        client::RpcClient,
        types::{Block, TransactionRequest},
    },
    signers::local::PrivateKeySigner,
    sol,
};
use async_trait::async_trait;
use hopr_bindings::hoprnodemanagementmodule::HoprNodeManagementModule::{self, HoprNodeManagementModuleInstance};
use hopr_chain_types::{ContractAddresses, ContractInstances, NetworkRegistryProxy};
use hopr_crypto_types::{
    keypairs::{ChainKeypair, Keypair},
    prelude::Hash,
};
use hopr_internal_types::prelude::{EncodedWinProb, WinningProbability};
use hopr_primitive_types::prelude::*;
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use tracing::debug;
use url::Url;
use validator::Validate;

// use crate::middleware::GnosisScan;
use crate::{
    HoprRpcOperations, NodeSafeModuleStatus,
    client::GasOracleFiller,
    errors::{Result, RpcError},
    transport::HttpRequestor,
};

// define basic safe abi
sol!(
    #![sol(abi)]
    #![sol(rpc)]
    contract SafeSingleton {
        function isModuleEnabled(address module) public view returns (bool);
    }
);

/// Default gas oracle URL for Gnosis chain.
pub const DEFAULT_GAS_ORACLE_URL: &str = "https://ggnosis.blockscan.com/gasapi.ashx?apikey=key&method=gasoracle";

/// Configuration of the RPC related parameters.
#[derive(Clone, Debug, PartialEq, Eq, smart_default::SmartDefault, Serialize, Deserialize, Validate)]
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
    /// URL to the gas price oracle.
    ///
    /// Defaults to [`DEFAULT_GAS_ORACLE_URL`].
    #[default(Some(DEFAULT_GAS_ORACLE_URL.parse().unwrap()))]
    pub gas_oracle_url: Option<Url>,
}

pub(crate) type HoprProvider<R> = FillProvider<
    JoinFill<
        JoinFill<
            JoinFill<
                JoinFill<
                    JoinFill<JoinFill<Identity, WalletFiller<EthereumWallet>>, ChainIdFiller>,
                    NonceFiller<CachedNonceManager>,
                >,
                GasFiller,
            >,
            GasOracleFiller<R>,
        >,
        BlobGasFiller,
    >,
    RootProvider,
>;

/// Implementation of `HoprRpcOperations` and `HoprIndexerRpcOperations` trait via `alloy`
#[derive(Debug, Clone)]
pub struct RpcOperations<R: HttpRequestor + 'static + Clone> {
    pub(crate) provider: Arc<HoprProvider<R>>,
    pub(crate) cfg: RpcOperationsConfig,
    contract_instances: Arc<ContractInstances<HoprProvider<R>>>,
    node_module: HoprNodeManagementModuleInstance<HoprProvider<R>>,
    node_safe: SafeSingletonInstance<HoprProvider<R>>,
}

#[cfg_attr(test, mockall::automock)]
impl<R: HttpRequestor + 'static + Clone> RpcOperations<R> {
    pub fn new(
        rpc_client: RpcClient,
        requestor: R,
        chain_key: &ChainKeypair,
        cfg: RpcOperationsConfig,
        use_dummy_nr: Option<bool>,
    ) -> Result<Self> {
        let wallet =
            PrivateKeySigner::from_slice(chain_key.secret().as_ref()).map_err(|e| RpcError::SignerError(e.into()))?;

        let provider = ProviderBuilder::new()
            .disable_recommended_fillers()
            .wallet(wallet)
            .filler(ChainIdFiller::default())
            .filler(NonceFiller::new(CachedNonceManager::default()))
            .filler(GasFiller)
            .filler(GasOracleFiller::new(requestor.clone(), cfg.gas_oracle_url.clone()))
            .filler(BlobGasFiller)
            .connect_client(rpc_client);

        debug!("{:?}", cfg.contract_addrs);

        Ok(Self {
            contract_instances: Arc::new(ContractInstances::new(
                &cfg.contract_addrs,
                provider.clone(),
                use_dummy_nr.unwrap_or(cfg!(test)),
            )),
            node_module: HoprNodeManagementModule::new(cfg.module_address.into(), provider.clone()),
            node_safe: SafeSingleton::new(cfg.safe_address.into(), provider.clone()),
            cfg,
            provider: Arc::new(provider),
        })
    }

    pub(crate) async fn get_block_number(&self) -> Result<u64> {
        Ok(self
            .provider
            .get_block_number()
            .await?
            .saturating_sub(self.cfg.finality as u64))
    }

    pub(crate) async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        let sanitized_block_number = block_number.saturating_sub(self.cfg.finality as u64);
        let result = self.provider.get_block_by_number(sanitized_block_number.into()).await?;
        Ok(result)
    }

    pub(crate) async fn get_xdai_balance(&self, address: Address) -> Result<XDaiBalance> {
        Ok(XDaiBalance::from(U256::from_be_bytes(
            self.provider.get_balance(address.into()).await?.to_be_bytes::<32>(),
        )))
    }

    pub(crate) async fn get_hopr_balance(&self, address: Address) -> Result<HoprBalance> {
        Ok(HoprBalance::from(U256::from_be_bytes(
            self.contract_instances
                .token
                .balanceOf(address.into())
                .call()
                .await?
                .to_be_bytes::<32>(),
        )))
    }

    pub(crate) async fn get_hopr_allowance(&self, owner: Address, spender: Address) -> Result<HoprBalance> {
        Ok(HoprBalance::from(U256::from_be_bytes(
            self.contract_instances
                .token
                .allowance(owner.into(), spender.into())
                .call()
                .await?
                .to_be_bytes::<32>(),
        )))
    }
}

#[async_trait]
impl<R: HttpRequestor + 'static + Clone> HoprRpcOperations for RpcOperations<R> {
    async fn get_timestamp(&self, block_number: u64) -> Result<Option<u64>> {
        Ok(self.get_block(block_number).await?.map(|b| b.header.timestamp))
    }

    async fn get_xdai_balance(&self, address: Address) -> Result<XDaiBalance> {
        self.get_xdai_balance(address).await
    }

    async fn get_hopr_balance(&self, address: Address) -> Result<HoprBalance> {
        self.get_hopr_balance(address).await
    }

    async fn get_hopr_allowance(&self, owner: Address, spender: Address) -> Result<HoprBalance> {
        self.get_hopr_allowance(owner, spender).await
    }

    async fn get_minimum_network_winning_probability(&self) -> Result<WinningProbability> {
        match self.contract_instances.win_prob_oracle.currentWinProb().call().await {
            Ok(encoded_win_prob) => {
                let mut encoded: EncodedWinProb = Default::default();
                encoded.copy_from_slice(&encoded_win_prob.to_be_bytes_vec());
                Ok(encoded.into())
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn get_minimum_network_ticket_price(&self) -> Result<HoprBalance> {
        Ok(self
            .contract_instances
            .price_oracle
            .currentTicketPrice()
            .call()
            .await
            .map(|v| HoprBalance::from(U256::from_be_bytes(v.to_be_bytes::<32>())))?)
    }

    async fn get_eligibility_status(&self, address: Address) -> Result<bool> {
        // 2) check if the node is registered to an account. In case the selfRegister is disabled
        // (i.e., when the staking threshold is zero) this value is used to check if the node is eligible
        let tx_2 = CallItemBuilder::new(
            self.contract_instances
                .network_registry
                .nodeRegisterdToAccount(address.into()),
        )
        .allow_failure(false);

        // 3) check if the node is registered and eligible
        let tx_3 = CallItemBuilder::new(
            self.contract_instances
                .network_registry
                .isNodeRegisteredAndEligible(address.into()),
        )
        .allow_failure(true);

        debug!(address = %self.contract_instances.network_registry_proxy.address(),"building tx_3 for eligibility check");
        // 1) check if the staking threshold set in the implementation is above zero
        let (stake_threshold, node_registration, quick_check) = match &self.contract_instances.network_registry_proxy {
            NetworkRegistryProxy::Dummy(c) => {
                debug!(proxy_address = %c.address(), "Using dummy network registry proxy for eligibility check");
                let tx_1_dummy = CallItemBuilder::new(c.maxAllowedRegistrations(address.into())).allow_failure(false);
                let multicall = self
                    .provider
                    .multicall()
                    .add_call(tx_1_dummy)
                    .add_call(tx_2)
                    .add_call(tx_3);
                let (max_allowed_registration, node_registered_to_account, quick_check) =
                    multicall.aggregate3().await.map_err(RpcError::MulticallError)?;
                (
                    max_allowed_registration
                        .map_err(|e| RpcError::MulticallFailure(e.idx, e.return_data.to_string()))?,
                    node_registered_to_account
                        .map_err(|e| RpcError::MulticallFailure(e.idx, e.return_data.to_string()))?,
                    quick_check,
                )
            }
            NetworkRegistryProxy::Safe(c) => {
                debug!(proxy_address = %c.address(), "Using safe network registry proxy for eligibility check");
                let tx_1_proxy = CallItemBuilder::new(c.stakeThreshold()).allow_failure(false);
                let multicall = self
                    .provider
                    .multicall()
                    .add_call(tx_1_proxy)
                    .add_call(tx_2)
                    .add_call(tx_3);
                let (stake_threshold, node_registered_to_account, quick_check) =
                    multicall.aggregate3().await.map_err(RpcError::MulticallError)?;
                (
                    stake_threshold.map_err(|e| RpcError::MulticallFailure(e.idx, e.return_data.to_string()))?,
                    node_registered_to_account
                        .map_err(|e| RpcError::MulticallFailure(e.idx, e.return_data.to_string()))?,
                    quick_check,
                )
            }
        };

        match &quick_check {
            Ok(eligibility_quick_check) => return Ok(*eligibility_quick_check),
            Err(e) => {
                // check in details what is the failure message
                // The "division by zero" error is caused by the self-registration,
                // which is forbidden to the public and thus returns false
                // therefore the eligibility check should be ignored
                // In EVM it returns `Panic(0x12)` error, where `0x4e487b71` is the function selector
                // https://docs.soliditylang.org/en/v0.8.12/control-structures.html#panic-via-assert-and-error-via-require
                if e.return_data.starts_with(&[0x4e, 0x48, 0x7b, 0x71])
                    && e.return_data[e.return_data.len() - 1] == 0x12
                {
                    // when receiving division by zero error, if the staking threshold is zero
                    // and registered account is not zero, return true
                    return Ok(stake_threshold.is_zero() && !node_registration.is_zero());
                }
                return Ok(false);
            }
        }
    }

    async fn get_node_management_module_target_info(&self, target: Address) -> Result<Option<U256>> {
        match self.node_module.tryGetTarget(target.into()).call().await {
            Ok(returned_result) => Ok(returned_result
                ._0
                .then_some(U256::from_big_endian(returned_result._1.to_be_bytes_vec().as_slice()))),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_safe_from_node_safe_registry(&self, node_address: Address) -> Result<Address> {
        match self
            .contract_instances
            .safe_registry
            .nodeToSafe(node_address.into())
            .call()
            .await
        {
            Ok(returned_result) => Ok(returned_result.into()),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_module_target_address(&self) -> Result<Address> {
        match self.node_module.owner().call().await {
            Ok(returned_result) => Ok(returned_result.into()),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_channel_closure_notice_period(&self) -> Result<Duration> {
        // TODO: should we cache this value internally ?
        match self
            .contract_instances
            .channels
            .noticePeriodChannelClosure()
            .call()
            .await
        {
            Ok(returned_result) => Ok(Duration::from_secs(returned_result.into())),
            Err(e) => Err(e.into()),
        }
    }

    // Check on-chain status of, node, safe, and module
    async fn check_node_safe_module_status(&self, node_address: Address) -> Result<NodeSafeModuleStatus> {
        // 1) check if the node is already included into the module
        let tx_1 = CallItemBuilder::new(self.node_module.isNode(node_address.into())).allow_failure(false);
        // 2) if the module is enabled in the safe
        let tx_2 =
            CallItemBuilder::new(self.node_safe.isModuleEnabled(self.cfg.module_address.into())).allow_failure(false);
        // 3) if the safe is the owner of the module
        let tx_3 = CallItemBuilder::new(self.node_module.owner()).allow_failure(false);
        let multicall = self.provider.multicall().add_call(tx_1).add_call(tx_2).add_call(tx_3);

        let (node_in_module_inclusion, module_safe_enabling, safe_of_module_ownership) =
            multicall.aggregate3_value().await.map_err(RpcError::MulticallError)?;

        let is_node_included_in_module =
            node_in_module_inclusion.map_err(|e| RpcError::MulticallFailure(e.idx, e.return_data.to_string()))?;
        let is_module_enabled_in_safe =
            module_safe_enabling.map_err(|e| RpcError::MulticallFailure(e.idx, e.return_data.to_string()))?;
        let is_safe_owner_of_module = self.cfg.safe_address.eq(&safe_of_module_ownership
            .map_err(|e| RpcError::MulticallFailure(e.idx, e.return_data.to_string()))?
            .0
            .0
            .into());

        Ok(NodeSafeModuleStatus {
            is_node_included_in_module,
            is_module_enabled_in_safe,
            is_safe_owner_of_module,
        })
    }

    async fn send_transaction(&self, tx: TransactionRequest) -> Result<PendingTransaction> {
        let sent_tx = self.provider.send_transaction(tx).await?;

        let pending_tx = sent_tx.register().await.map_err(RpcError::PendingTransactionError)?;

        Ok(pending_tx)
    }

    async fn send_transaction_with_confirm(&self, tx: TransactionRequest) -> Result<Hash> {
        let sent_tx = self.provider.send_transaction(tx).await?;

        let receipt = sent_tx.get_receipt().await.map_err(RpcError::PendingTransactionError)?;

        let tx_hash = Hash::from(receipt.transaction_hash.0);

        // Check the transaction status. `status()` returns `true` for successful transactions
        // and `false` for failed or reverted transactions.
        if receipt.status() {
            Ok(tx_hash)
        } else {
            // Transaction failed, raise an error
            Err(RpcError::TransactionFailed(tx_hash))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use alloy::{
        network::{Ethereum, TransactionBuilder},
        primitives::{U256, address},
        providers::Provider,
        rpc::{client::ClientBuilder, types::TransactionRequest},
        transports::{http::ReqwestTransport, layers::RetryBackoffLayer},
    };
    use hex_literal::hex;
    use hopr_async_runtime::prelude::sleep;
    use hopr_chain_types::{ContractAddresses, ContractInstances, NetworkRegistryProxy, utils::create_native_transfer};
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
    use hopr_primitive_types::prelude::*;
    use primitive_types::H160;

    use crate::{
        HoprRpcOperations, PendingTransaction,
        client::{AnvilRpcClient, create_rpc_client_to_anvil},
        errors::Result,
        rpc::{RpcOperations, RpcOperationsConfig},
    };

    lazy_static::lazy_static! {
        static ref RANDY: Address = hex!("762614a5ed652457a2f1cdb8006380530c26ae6a").into();
        static ref MULTICALL3_DEPLOY_CODE: [u8; 3926] = hex!("f90f538085174876e800830f42408080b90f00608060405234801561001057600080fd5b50610ee0806100206000396000f3fe6080604052600436106100f35760003560e01c80634d2301cc1161008a578063a8b0574e11610059578063a8b0574e1461025a578063bce38bd714610275578063c3077fa914610288578063ee82ac5e1461029b57600080fd5b80634d2301cc146101ec57806372425d9d1461022157806382ad56cb1461023457806386d516e81461024757600080fd5b80633408e470116100c65780633408e47014610191578063399542e9146101a45780633e64a696146101c657806342cbb15c146101d957600080fd5b80630f28c97d146100f8578063174dea711461011a578063252dba421461013a57806327e86d6e1461015b575b600080fd5b34801561010457600080fd5b50425b6040519081526020015b60405180910390f35b61012d610128366004610a85565b6102ba565b6040516101119190610bbe565b61014d610148366004610a85565b6104ef565b604051610111929190610bd8565b34801561016757600080fd5b50437fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0140610107565b34801561019d57600080fd5b5046610107565b6101b76101b2366004610c60565b610690565b60405161011193929190610cba565b3480156101d257600080fd5b5048610107565b3480156101e557600080fd5b5043610107565b3480156101f857600080fd5b50610107610207366004610ce2565b73ffffffffffffffffffffffffffffffffffffffff163190565b34801561022d57600080fd5b5044610107565b61012d610242366004610a85565b6106ab565b34801561025357600080fd5b5045610107565b34801561026657600080fd5b50604051418152602001610111565b61012d610283366004610c60565b61085a565b6101b7610296366004610a85565b610a1a565b3480156102a757600080fd5b506101076102b6366004610d18565b4090565b60606000828067ffffffffffffffff8111156102d8576102d8610d31565b60405190808252806020026020018201604052801561031e57816020015b6040805180820190915260008152606060208201528152602001906001900390816102f65790505b5092503660005b8281101561047757600085828151811061034157610341610d60565b6020026020010151905087878381811061035d5761035d610d60565b905060200281019061036f9190610d8f565b6040810135958601959093506103886020850185610ce2565b73ffffffffffffffffffffffffffffffffffffffff16816103ac6060870187610dcd565b6040516103ba929190610e32565b60006040518083038185875af1925050503d80600081146103f7576040519150601f19603f3d011682016040523d82523d6000602084013e6103fc565b606091505b50602080850191909152901515808452908501351761046d577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260846000fd5b5050600101610325565b508234146104e6576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601a60248201527f4d756c746963616c6c333a2076616c7565206d69736d6174636800000000000060448201526064015b60405180910390fd5b50505092915050565b436060828067ffffffffffffffff81111561050c5761050c610d31565b60405190808252806020026020018201604052801561053f57816020015b606081526020019060019003908161052a5790505b5091503660005b8281101561068657600087878381811061056257610562610d60565b90506020028101906105749190610e42565b92506105836020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166105a66020850185610dcd565b6040516105b4929190610e32565b6000604051808303816000865af19150503d80600081146105f1576040519150601f19603f3d011682016040523d82523d6000602084013e6105f6565b606091505b5086848151811061060957610609610d60565b602090810291909101015290508061067d576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b50600101610546565b5050509250929050565b43804060606106a086868661085a565b905093509350939050565b6060818067ffffffffffffffff8111156106c7576106c7610d31565b60405190808252806020026020018201604052801561070d57816020015b6040805180820190915260008152606060208201528152602001906001900390816106e55790505b5091503660005b828110156104e657600084828151811061073057610730610d60565b6020026020010151905086868381811061074c5761074c610d60565b905060200281019061075e9190610e76565b925061076d6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166107906040850185610dcd565b60405161079e929190610e32565b6000604051808303816000865af19150503d80600081146107db576040519150601f19603f3d011682016040523d82523d6000602084013e6107e0565b606091505b506020808401919091529015158083529084013517610851577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260646000fd5b50600101610714565b6060818067ffffffffffffffff81111561087657610876610d31565b6040519080825280602002602001820160405280156108bc57816020015b6040805180820190915260008152606060208201528152602001906001900390816108945790505b5091503660005b82811015610a105760008482815181106108df576108df610d60565b602002602001015190508686838181106108fb576108fb610d60565b905060200281019061090d9190610e42565b925061091c6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff1661093f6020850185610dcd565b60405161094d929190610e32565b6000604051808303816000865af19150503d806000811461098a576040519150601f19603f3d011682016040523d82523d6000602084013e61098f565b606091505b506020830152151581528715610a07578051610a07576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b506001016108c3565b5050509392505050565b6000806060610a2b60018686610690565b919790965090945092505050565b60008083601f840112610a4b57600080fd5b50813567ffffffffffffffff811115610a6357600080fd5b6020830191508360208260051b8501011115610a7e57600080fd5b9250929050565b60008060208385031215610a9857600080fd5b823567ffffffffffffffff811115610aaf57600080fd5b610abb85828601610a39565b90969095509350505050565b6000815180845260005b81811015610aed57602081850181015186830182015201610ad1565b81811115610aff576000602083870101525b50601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0169290920160200192915050565b600082825180855260208086019550808260051b84010181860160005b84811015610bb1578583037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe001895281518051151584528401516040858501819052610b9d81860183610ac7565b9a86019a9450505090830190600101610b4f565b5090979650505050505050565b602081526000610bd16020830184610b32565b9392505050565b600060408201848352602060408185015281855180845260608601915060608160051b870101935082870160005b82811015610c52577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa0888703018452610c40868351610ac7565b95509284019290840190600101610c06565b509398975050505050505050565b600080600060408486031215610c7557600080fd5b83358015158114610c8557600080fd5b9250602084013567ffffffffffffffff811115610ca157600080fd5b610cad86828701610a39565b9497909650939450505050565b838152826020820152606060408201526000610cd96060830184610b32565b95945050505050565b600060208284031215610cf457600080fd5b813573ffffffffffffffffffffffffffffffffffffffff81168114610bd157600080fd5b600060208284031215610d2a57600080fd5b5035919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81833603018112610dc357600080fd5b9190910192915050565b60008083357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe1843603018112610e0257600080fd5b83018035915067ffffffffffffffff821115610e1d57600080fd5b602001915036819003821315610a7e57600080fd5b8183823760009101908152919050565b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc1833603018112610dc357600080fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa1833603018112610dc357600080fdfea2646970667358221220bb2b5c71a328032f97c676ae39a1ec2148d3e5d6f73d95e9b17910152d61f16264736f6c634300080c00331ca0edce47092c0f398cebf3ffc267f05c8e7076e3b89445e0fe50f6332273d4569ba01b0b9d000e19b24c5869b0fc3b22b0d6fa47cd63316875cbbd577d76e6fde086");
        static ref MULTICALL3_DEPLOYER_ADDRESS: alloy::primitives::Address = address!("05f32b3cc3888453ff71b01135b34ff8e41263f2");
    }

    pub const ETH_VALUE_FOR_MULTICALL3_DEPLOYER: u128 = 100_000_000_000_000_000;
    // 0.1 (anvil) ETH

    pub async fn wait_until_tx(pending: PendingTransaction, timeout: Duration) {
        let tx_hash = *pending.tx_hash();
        sleep(timeout).await;
        pending
            .await
            .unwrap_or_else(|_| panic!("timeout awaiting tx hash {tx_hash} after {} seconds", timeout.as_secs()));
    }

    /// Deploy a MULTICALL contract into Anvil local chain for testing
    pub async fn deploy_multicall3_to_anvil<P: Provider>(provider: &P) -> Result<()> {
        // Fund Multicall3 deployer and deploy ERC1820Registry
        let tx = TransactionRequest::default()
            .with_to(*MULTICALL3_DEPLOYER_ADDRESS)
            .with_value(U256::from(ETH_VALUE_FOR_MULTICALL3_DEPLOYER));

        provider.send_transaction(tx).await?.watch().await?;

        provider
            .send_raw_transaction(MULTICALL3_DEPLOY_CODE.as_ref())
            .await?
            .watch()
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_should_estimate_tx() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);
        let anvil = hopr_chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;

        let mut server = mockito::Server::new_async().await;
        let gas_oracle_mock = server.mock("GET", "/gas_oracle")
            .with_status(200)
            .with_body(r#"{"status":"1","message":"OK","result":{"LastBlock":"38791478","SafeGasPrice":"1.1","ProposeGasPrice":"1.1","FastGasPrice":"1.6","UsdPrice":"0.999985432689946"}}"#)
            .expect(0)
            .create_async()
            .await;

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            expected_block_time,
            finality: 2,
            gas_oracle_url: Some((server.url() + "/gas_oracle").parse()?),
            ..RpcOperationsConfig::default()
        };

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(2, 100, 100))
            .transport(transport_client.clone(), transport_client.guess_local());

        // Wait until contracts deployments are final
        sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(rpc_client, transport_client.client().clone(), &chain_key_0, cfg, None)?;

        // call eth_gas_estimate
        let fees = rpc.provider.estimate_eip1559_fees().await?;

        assert!(
            fees.max_priority_fee_per_gas.ge(&0_u128),
            // fees.max_priority_fee_per_gas.ge(&100_000_000_u128),
            "estimated_max_priority_fee must be equal or greater than 0, 0.1 gwei"
        );

        let estimated_gas_price = rpc.provider.get_gas_price().await?;
        assert!(
            estimated_gas_price.ge(&100_000_000_u128),
            "estimated_max_fee must be greater than 0.1 gwei"
        );

        gas_oracle_mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn test_should_send_tx() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);
        let anvil = hopr_chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            expected_block_time,
            finality: 2,
            gas_oracle_url: None,
            ..RpcOperationsConfig::default()
        };

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(2, 100, 100))
            .transport(transport_client.clone(), transport_client.guess_local());

        // Wait until contracts deployments are final
        sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(rpc_client, transport_client.client().clone(), &chain_key_0, cfg, None)?;

        let balance_1: XDaiBalance = rpc.get_xdai_balance((&chain_key_0).into()).await?;
        assert!(balance_1.amount().gt(&0.into()), "balance must be greater than 0");

        // Test 1: Send 1 ETH to some random address, do not wait for confirmation
        let tx_1 = create_native_transfer::<Ethereum>(*RANDY, U256::from(1000000_u32));
        let tx_hash = rpc.send_transaction(tx_1).await?;

        wait_until_tx(tx_hash, Duration::from_secs(8)).await;

        // Test 2: Send 1 ETH to some random address, wait for confirmation
        let tx_2 = create_native_transfer::<Ethereum>(*RANDY, U256::from(1000000_u32));
        let _receipt = rpc.send_transaction_with_confirm(tx_2).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_should_send_consecutive_txs() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);
        let anvil = hopr_chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            expected_block_time,
            finality: 2,
            gas_oracle_url: None,
            ..RpcOperationsConfig::default()
        };

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(2, 100, 100))
            .transport(transport_client.clone(), transport_client.guess_local());

        // Wait until contracts deployments are final
        sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(
            rpc_client,
            transport_client.client().clone(),
            &chain_key_0,
            cfg.clone(),
            None,
        )?;

        let balance_1: XDaiBalance = rpc.get_xdai_balance((&chain_key_0).into()).await?;
        assert!(balance_1.amount().gt(&0.into()), "balance must be greater than 0");

        let txs_count = 5_u64;
        let send_amount = 1000000_u64;

        // Send 1 ETH to some random address
        futures::future::join_all((0..txs_count).map(|_| async {
            rpc.send_transaction(create_native_transfer::<Ethereum>(*RANDY, U256::from(send_amount)))
                .await
                .expect("tx should be sent")
                .await
                .expect("tx should resolve")
        }))
        .await;

        sleep((1 + cfg.finality) * expected_block_time).await;

        let balance_2: XDaiBalance = rpc.get_xdai_balance((&chain_key_0).into()).await?;

        assert!(
            balance_2.amount() <= balance_1.amount() - txs_count * send_amount,
            "balance must be less"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_get_balance_native() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);
        let anvil = hopr_chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            expected_block_time,
            finality: 2,
            gas_oracle_url: None,
            ..RpcOperationsConfig::default()
        };

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(2, 100, 100))
            .transport(transport_client.clone(), transport_client.guess_local());

        // Wait until contracts deployments are final
        sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(rpc_client, transport_client.client().clone(), &chain_key_0, cfg, None)?;

        let balance_1: XDaiBalance = rpc.get_xdai_balance((&chain_key_0).into()).await?;
        assert!(balance_1.amount().gt(&0.into()), "balance must be greater than 0");

        // Send 1 ETH to some random address
        let tx_hash = rpc
            .send_transaction(create_native_transfer::<Ethereum>(*RANDY, U256::from(1_u32)))
            .await?;

        wait_until_tx(tx_hash, Duration::from_secs(8)).await;

        let balance_2: XDaiBalance = rpc.get_xdai_balance((&chain_key_0).into()).await?;
        assert!(balance_2.lt(&balance_1), "balance must be diminished");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_balance_token() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);
        let anvil = hopr_chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;

        // Deploy contracts
        let contract_instances = {
            let client = create_rpc_client_to_anvil(&anvil, &chain_key_0);
            ContractInstances::deploy_for_testing(client, &chain_key_0).await?
        };

        let cfg = RpcOperationsConfig {
            chain_id: anvil.chain_id(),
            tx_polling_interval: Duration::from_millis(10),
            expected_block_time,
            finality: 2,
            contract_addrs: ContractAddresses::from(&contract_instances),
            gas_oracle_url: None,
            ..RpcOperationsConfig::default()
        };

        let amount = 1024_u64;
        let _ = hopr_chain_types::utils::mint_tokens(contract_instances.token, U256::from(amount)).await;

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(2, 100, 100))
            .transport(transport_client.clone(), transport_client.guess_local());

        // Wait until contracts deployments are final
        sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(rpc_client, transport_client.client().clone(), &chain_key_0, cfg, None)?;

        let balance: HoprBalance = rpc.get_hopr_balance((&chain_key_0).into()).await?;
        assert_eq!(amount, balance.amount().as_u64(), "invalid balance");

        Ok(())
    }

    #[tokio::test]
    async fn test_check_node_safe_module_status() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);
        let anvil = hopr_chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;

        // Deploy contracts
        let (contract_instances, module, safe) = {
            let client = create_rpc_client_to_anvil(&anvil, &chain_key_0);
            let instances = ContractInstances::deploy_for_testing(client.clone(), &chain_key_0).await?;

            // deploy MULTICALL contract to anvil
            deploy_multicall3_to_anvil(&client.clone()).await?;

            let (module, safe) = hopr_chain_types::utils::deploy_one_safe_one_module_and_setup_for_testing::<
                Arc<AnvilRpcClient>,
            >(&instances, client.clone(), &chain_key_0)
            .await?;

            // deploy a module and safe instance and add node into the module. The module is enabled by default in the
            // safe
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
            gas_oracle_url: None,
            ..RpcOperationsConfig::default()
        };

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(2, 100, 100))
            .transport(transport_client.clone(), transport_client.guess_local());

        // Wait until contracts deployments are final
        sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(rpc_client, transport_client.client().clone(), &chain_key_0, cfg, None)?;

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
        hopr_chain_types::utils::include_node_to_module_by_safe(
            contract_instances.channels.provider().clone(),
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

    #[tokio::test]
    async fn test_get_eligibility_status() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);
        let anvil = hopr_chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let node_address: H160 = chain_key_0.public().to_address().into();

        // Deploy contracts
        let (contract_instances, module, safe) = {
            let client = create_rpc_client_to_anvil(&anvil, &chain_key_0);
            let instances = ContractInstances::deploy_for_testing(client.clone(), &chain_key_0).await?;

            // deploy MULTICALL contract to anvil
            deploy_multicall3_to_anvil(&client.clone()).await?;

            let (module, safe) = hopr_chain_types::utils::deploy_one_safe_one_module_and_setup_for_testing::<
                Arc<AnvilRpcClient>,
            >(&instances, client.clone(), &chain_key_0)
            .await?;

            // deploy a module and safe instance and add node into the module. The module is enabled by default in the
            // safe
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
            gas_oracle_url: None,
            ..RpcOperationsConfig::default()
        };

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(2, 100, 100))
            .transport(transport_client.clone(), transport_client.guess_local());

        // Wait until contracts deployments are final
        sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(
            rpc_client,
            transport_client.client().clone(),
            &chain_key_0,
            cfg.clone(),
            None,
        )?;

        // check the eligibility status (before registering in the NetworkRegistry contract)
        let result_before_register_in_the_network_registry = rpc.get_eligibility_status(node_address.into()).await?;

        assert!(
            !result_before_register_in_the_network_registry,
            "node should not be eligible"
        );

        // register the node
        match &rpc.contract_instances.network_registry_proxy {
            NetworkRegistryProxy::Dummy(e) => {
                let _ = e.ownerAddAccount(cfg.safe_address.into()).send().await?.watch().await?;
            }
            NetworkRegistryProxy::Safe(_) => {}
        };

        let _ = rpc
            .contract_instances
            .network_registry
            .managerRegister(
                vec![cfg.safe_address.into()],
                vec![alloy::primitives::Address::from_slice(node_address.as_ref())],
            )
            .send()
            .await?
            .watch()
            .await?;

        // check the eligibility status (after registering in the NetworkRegistry contract)
        let result_after_register_in_the_network_registry = rpc.get_eligibility_status(node_address.into()).await?;

        assert!(result_after_register_in_the_network_registry, "node should be eligible");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_eligibility_status_for_staking_proxy() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let expected_block_time = Duration::from_secs(1);
        let anvil = hopr_chain_types::utils::create_anvil(Some(expected_block_time));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let node_address: H160 = chain_key_0.public().to_address().into();

        // Deploy contracts
        let (contract_instances, module, safe) = {
            let client = create_rpc_client_to_anvil(&anvil, &chain_key_0);
            let instances =
                ContractInstances::deploy_for_testing_with_staking_proxy(client.clone(), &chain_key_0).await?;

            // deploy MULTICALL contract to anvil
            deploy_multicall3_to_anvil(&client.clone()).await?;

            let (module, safe) = hopr_chain_types::utils::deploy_one_safe_one_module_and_setup_for_testing::<
                Arc<AnvilRpcClient>,
            >(&instances, client.clone(), &chain_key_0)
            .await?;

            // deploy a module and safe instance and add node into the module. The module is enabled by default in the
            // safe
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
            gas_oracle_url: None,
            ..RpcOperationsConfig::default()
        };

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(2, 100, 100))
            .transport(transport_client.clone(), transport_client.guess_local());

        // Wait until contracts deployments are final
        sleep((1 + cfg.finality) * expected_block_time).await;

        let rpc = RpcOperations::new(
            rpc_client,
            transport_client.client().clone(),
            &chain_key_0,
            cfg.clone(),
            Some(false),
        )?;

        // check the eligibility status (before registering in the NetworkRegistry contract)
        let result_before_register_in_the_network_registry = rpc.get_eligibility_status(node_address.into()).await?;

        assert!(
            !result_before_register_in_the_network_registry,
            "node should not be eligible"
        );

        // register the node
        match &rpc.contract_instances.network_registry_proxy {
            NetworkRegistryProxy::Dummy(p) => {
                let _ = p.ownerAddAccount(cfg.safe_address.into()).send().await?.watch().await?;
            }
            NetworkRegistryProxy::Safe(_) => {}
        };

        let _ = rpc
            .contract_instances
            .network_registry
            .managerRegister(
                vec![cfg.safe_address.into()],
                vec![alloy::primitives::Address::from_slice(node_address.as_ref())],
            )
            .send()
            .await?
            .watch()
            .await?;

        // check the eligibility status (after registering in the NetworkRegistry contract)
        let result_after_register_in_the_network_registry = rpc.get_eligibility_status(node_address.into()).await?;

        assert!(result_after_register_in_the_network_registry, "node should be eligible");
        Ok(())
    }
}
