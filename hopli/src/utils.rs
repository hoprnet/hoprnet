//! This module contains errors produced in this crate

use std::str::FromStr;

use SafeContract::SafeContractInstance;
use alloy::{
    contract::{Error as ContractError, Result as ContractResult},
    hex::FromHexError,
    network::{ReceiptResponse, TransactionBuilder},
    primitives,
    primitives::{Address, Bytes, U256, address, aliases, keccak256},
    providers::{MULTICALL3_ADDRESS, MulticallError, PendingTransactionError},
    rpc::types::TransactionRequest,
    signers::{Signer, local::PrivateKeySigner},
    sol,
    sol_types::{SolCall, SolValue},
    transports::TransportErrorKind,
};
use hopr_bindings::{
    hoprannouncements::{HoprAnnouncements, HoprAnnouncements::HoprAnnouncementsInstance},
    hoprchannels::{HoprChannels, HoprChannels::HoprChannelsInstance},
    hoprdummyproxyfornetworkregistry::{
        HoprDummyProxyForNetworkRegistry, HoprDummyProxyForNetworkRegistry::HoprDummyProxyForNetworkRegistryInstance,
    },
    hoprnetworkregistry::HoprNetworkRegistry,
    hoprnodemanagementmodule::{HoprNodeManagementModule, HoprNodeManagementModule::HoprNodeManagementModuleInstance},
    hoprnodesaferegistry::{HoprNodeSafeRegistry, HoprNodeSafeRegistry::HoprNodeSafeRegistryInstance},
    hoprnodestakefactory::{HoprNodeStakeFactory, HoprNodeStakeFactory::HoprNodeStakeFactoryInstance},
    hoprsafeproxyfornetworkregistry::{
        HoprSafeProxyForNetworkRegistry, HoprSafeProxyForNetworkRegistry::HoprSafeProxyForNetworkRegistryInstance,
    },
    hoprticketpriceoracle::{HoprTicketPriceOracle, HoprTicketPriceOracle::HoprTicketPriceOracleInstance},
    hoprtoken::{HoprToken, HoprToken::HoprTokenInstance},
    hoprwinningprobabilityoracle::{
        HoprWinningProbabilityOracle, HoprWinningProbabilityOracle::HoprWinningProbabilityOracleInstance,
    },
};
use hopr_chain_types::{ContractAddresses, errors::ChainTypesError};
use hopr_crypto_keypair::errors::KeyPairError;
use hopr_crypto_types::{keypairs::ChainKeypair, prelude::Keypair};
use thiserror::Error;
use tracing::debug;

use crate::constants::{
    ERC_1820_DEPLOYER, ERC_1820_REGISTRY_DEPLOY_CODE, ETH_VALUE_FOR_ERC1820_DEPLOYER, MULTICALL3_DEPLOY_CODE,
};

pub trait Cmd: clap::Parser + Sized {
    fn run(self) -> Result<(), HelperErrors>;
    fn async_run(self) -> impl std::future::Future<Output = Result<(), HelperErrors>> + Send;
}

/// Enumerates different errors produced by this crate.
#[derive(Error, Debug)]
pub enum HelperErrors {
    /// Error of contract error
    #[error(transparent)]
    FromHexError(#[from] FromHexError),

    /// Error of contract error
    #[error(transparent)]
    ContractError(#[from] ContractError),

    /// Error propagated by pending transctions
    #[error(transparent)]
    PendingTransactionError(#[from] PendingTransactionError),

    /// Error propagated by IO operations
    #[error(transparent)]
    UnableToReadFromPath(#[from] std::io::Error),

    /// Error in parsing provided comma-separated addresses
    #[error("error parsing address: {0:?}")]
    UnableToParseAddress(String),

    /// System time rrror
    #[error(transparent)]
    SystemTime(#[from] std::time::SystemTimeError),

    /// Error when identity cannot be created
    #[error("unable to create identity")]
    UnableToCreateIdentity,

    #[error("unable to update identity password")]
    UnableToUpdateIdentityPassword,

    /// Error due to supplying a non-existing file name
    #[error("incorrect filename: {0}")]
    IncorrectFilename(String),

    /// Error when identity existed
    #[error("identity file exists: {0}")]
    IdentityFileExists(String),

    /// Fail to read identity
    #[error("unable to read identity")]
    UnableToReadIdentity,

    /// Fail to find the identity directory
    #[error("unable to read identity directory")]
    MissingIdentityDirectory,

    /// Fail to delete an identity
    #[error("unable to delete identity")]
    UnableToDeleteIdentity,

    /// Provided environement does not match with that in the `ethereum/contracts/contracts-addresses.json`
    #[error("environment info mismatch")]
    EnvironmentInfoMismatch,

    /// Wrong foundry contract root is provided
    #[error("unable to set foundry root")]
    UnableToSetFoundryRoot,

    /// Fail to run foundry
    #[error("unable to run foundry")]
    ErrorInRunningFoundry,

    /// Fail to read password
    #[error("unable read password")]
    UnableToReadPassword,

    /// Fail to read private key
    #[error("cannot read private key error: {0}")]
    UnableToReadPrivateKey(String),

    /// Paramters are missing
    #[error("missing parameter: {0}")]
    MissingParameter(String),

    /// Error with the keystore file
    #[error(transparent)]
    KeyStoreError(#[from] KeyPairError),

    #[error("deserialization Error: {0}")]
    /// Serde JSON Error
    SerdeJson(#[from] serde_json::Error),

    /// Cannot find network details from the given network name
    #[error("unable to find network details from the given network name ")]
    UnknownNetwork,

    /// Error with HTTP Json RPC provider
    #[error(transparent)]
    RpcTransportError(#[from] alloy::rpc::json_rpc::RpcError<TransportErrorKind>),

    /// Fail to make a multicall
    #[error(transparent)]
    MulticallError(#[from] MulticallError),

    /// Fail to make a multisend call
    #[error("internal transaction failure in multisend")]
    MultiSendError,

    /// Txn caller does not have the minter role
    #[error("caller does not have the privilege to mint tokens")]
    NotAMinter,

    /// Error with middleware
    #[error("middleware Error: {0}")]
    MiddlewareError(String),

    /// A required smart contract (Safe or module proxy instance) is not deployed
    #[error("contract not deployed: {0}")]
    ContractNotDeployed(String),

    // error of parsing addresses
    #[error("Cannot parse address: {0}")]
    InvalidAddress(String),

    // general error of parsing
    #[error("Cannot parse: {0}")]
    ParseError(String),
}

// Used instead of From implementation to avoid alloy being a dependency of the primitive crates
/// Converts [`alloy::primitives::Address`] into [`hopr_primitive_types::prelude::Address`]
pub fn h2a(h: alloy::primitives::Address) -> hopr_primitive_types::prelude::Address {
    hopr_primitive_types::prelude::Address::from(h.0.0)
}

/// Converts [`hopr_primitive_types::prelude::Address`] into [`alloy::primitives::Address`]
pub fn a2h(a: hopr_primitive_types::prelude::Address) -> alloy::primitives::Address {
    alloy::primitives::Address::from_slice(a.as_ref())
}

#[derive(Debug, Clone)]
pub enum NetworkRegistryProxy<P> {
    Dummy(HoprDummyProxyForNetworkRegistryInstance<P>),
    Safe(HoprSafeProxyForNetworkRegistryInstance<P>),
}

impl<P> NetworkRegistryProxy<P>
where
    P: alloy::providers::Provider + Clone,
{
    pub fn address(&self) -> hopr_primitive_types::prelude::Address {
        match self {
            NetworkRegistryProxy::Dummy(c) => h2a(*c.address()),
            NetworkRegistryProxy::Safe(c) => h2a(*c.address()),
        }
    }
}

/// Holds instances to contracts.
#[derive(Debug, Clone)]
pub struct ContractInstances<P> {
    pub token: HoprTokenInstance<P>,
    pub channels: HoprChannelsInstance<P>,
    pub announcements: HoprAnnouncementsInstance<P>,
    pub safe_registry: HoprNodeSafeRegistryInstance<P>,
    pub price_oracle: HoprTicketPriceOracleInstance<P>,
    pub win_prob_oracle: HoprWinningProbabilityOracleInstance<P>,
    pub stake_factory: HoprNodeStakeFactoryInstance<P>,
    pub module_implementation: HoprNodeManagementModuleInstance<P>,
}

impl<P> ContractInstances<P>
where
    P: alloy::providers::Provider + Clone,
{
    pub fn new(contract_addresses: &ContractAddresses, provider: P, _use_dummy_nr: bool) -> Self {
        Self {
            token: HoprTokenInstance::new(a2h(contract_addresses.token), provider.clone()),
            channels: HoprChannelsInstance::new(a2h(contract_addresses.channels), provider.clone()),
            announcements: HoprAnnouncementsInstance::new(a2h(contract_addresses.announcements), provider.clone()),
            safe_registry: HoprNodeSafeRegistryInstance::new(
                a2h(contract_addresses.node_safe_registry),
                provider.clone(),
            ),
            price_oracle: HoprTicketPriceOracleInstance::new(
                a2h(contract_addresses.ticket_price_oracle),
                provider.clone(),
            ),
            win_prob_oracle: HoprWinningProbabilityOracleInstance::new(
                a2h(contract_addresses.winning_probability_oracle),
                provider.clone(),
            ),
            stake_factory: HoprNodeStakeFactoryInstance::new(
                a2h(contract_addresses.node_stake_v2_factory),
                provider.clone(),
            ),
            module_implementation: HoprNodeManagementModuleInstance::new(
                a2h(contract_addresses.module_implementation),
                provider.clone(),
            ),
        }
    }

    /// Deploys testing environment (with dummy network registry proxy) via the given provider.
    async fn inner_deploy_common_contracts_for_testing(provider: P, deployer: &ChainKeypair) -> ContractResult<Self> {
        {
            debug!("deploying ERC1820 registry...");
            // Fund 1820 deployer and deploy ERC1820Registry
            let tx = TransactionRequest::default()
                .with_to(ERC_1820_DEPLOYER)
                .with_value(ETH_VALUE_FOR_ERC1820_DEPLOYER);

            // Sequentially executing the following transactions:
            // 1. Fund the deployer wallet
            provider.send_transaction(tx.clone()).await?.watch().await?;
            // 2. Use the funded deployer wallet to deploy ERC1820Registry with a signed txn
            provider
                .send_raw_transaction(&ERC_1820_REGISTRY_DEPLOY_CODE)
                .await?
                .watch()
                .await?;
        }

        {
            debug!("deploying Multicall3...");
            // Fund Multicall3 deployer and deploy Multicall3
            let multicall3_code = provider.get_code_at(MULTICALL3_ADDRESS).await?;
            if multicall3_code.is_empty() {
                // Fund Multicall3 deployer and deploy ERC1820Registry
                let tx = TransactionRequest::default()
                    .with_to(crate::constants::MULTICALL3_DEPLOYER)
                    .with_value(crate::constants::ETH_VALUE_FOR_MULTICALL3_DEPLOYER);
                // Sequentially executing the following transactions:
                // 1. Fund the deployer wallet
                provider.send_transaction(tx.clone()).await?.watch().await?;
                // 2. Use the funded deployer wallet to deploy Multicall3 with a signed txn
                provider
                    .send_raw_transaction(MULTICALL3_DEPLOY_CODE)
                    .await?
                    .watch()
                    .await?;
            }
        }

        debug!("deploying contracts...");

        // Get deployer address
        let self_address = a2h(deployer.public().to_address());

        let stake_factory = HoprNodeStakeFactory::deploy(provider.clone()).await?;
        let module_implementation = HoprNodeManagementModule::deploy(provider.clone()).await?;
        let safe_registry = HoprNodeSafeRegistry::deploy(provider.clone()).await?;
        let price_oracle = HoprTicketPriceOracle::deploy(
            provider.clone(),
            self_address,
            primitives::U256::from(100000000000000000_u128), // U256::from(100000000000000000_u128),
        )
        .await?;
        let win_prob_oracle = HoprWinningProbabilityOracle::deploy(
            provider.clone(),
            self_address,
            primitives::aliases::U56::from(0xFFFFFFFFFFFFFF_u64), /* 0xFFFFFFFFFFFFFF in hex or 72057594037927935 in
                                                                   * decimal values */
        )
        .await?;
        let token = HoprToken::deploy(provider.clone()).await?;
        let channels = HoprChannels::deploy(
            provider.clone(),
            primitives::Address::from(token.address().as_ref()),
            1_u32,
            primitives::Address::from(safe_registry.address().as_ref()),
        )
        .await?;
        let announcements = HoprAnnouncements::deploy(
            provider.clone(),
            primitives::Address::from(safe_registry.address().as_ref()),
        )
        .await?;

        Ok(Self {
            token,
            channels,
            announcements,
            safe_registry,
            price_oracle,
            win_prob_oracle,
            stake_factory,
            module_implementation,
        })
    }

    /// Deploys testing environment (with dummy network registry proxy) via the given provider.
    pub async fn deploy_for_testing(provider: P, deployer: &ChainKeypair) -> ContractResult<Self> {
        let instances = Self::inner_deploy_common_contracts_for_testing(provider.clone(), deployer).await?;

        // Get deployer address
        let self_address = a2h(deployer.public().to_address());
        // Deploy network registry proxy
        let network_registry_proxy = HoprDummyProxyForNetworkRegistry::deploy(provider.clone(), self_address).await?;
        let network_registry = HoprNetworkRegistry::deploy(
            provider.clone(),
            primitives::Address::from(network_registry_proxy.address().as_ref()),
            self_address,
            self_address,
        )
        .await?;

        // Disable network registry in local environment and wait for its confirmation
        network_registry.disableRegistry().send().await?.watch().await?;

        Ok(Self { ..instances })
    }

    /// Deploys testing environment (with dummy network registry proxy) via the given provider.
    pub async fn deploy_for_testing_with_staking_proxy(provider: P, deployer: &ChainKeypair) -> ContractResult<Self> {
        let instances = Self::inner_deploy_common_contracts_for_testing(provider.clone(), deployer).await?;

        // Get deployer address
        let self_address = a2h(deployer.public().to_address());
        // Deploy network registry proxy
        // TODO:
        let network_registry_proxy = HoprSafeProxyForNetworkRegistry::deploy(
            provider.clone(),
            self_address,
            self_address,
            primitives::Uint::ZERO,
            provider.get_block_number().await?.into(),
            primitives::Address::from(instances.token.address().as_ref()),
            primitives::Address::from(instances.safe_registry.address().as_ref()),
        )
        .await?;
        let network_registry = HoprNetworkRegistry::deploy(
            provider.clone(),
            primitives::Address::from(network_registry_proxy.address().as_ref()),
            self_address,
            self_address,
        )
        .await?;

        // Disable network registry in local environment and wait for its confirmation
        network_registry.disableRegistry().send().await?.watch().await?;

        Ok(Self { ..instances })
    }
}

impl<P> From<&ContractInstances<P>> for ContractAddresses
where
    P: alloy::providers::Provider + Clone,
{
    fn from(instances: &ContractInstances<P>) -> Self {
        Self {
            token: h2a(*instances.token.address()),
            channels: h2a(*instances.channels.address()),
            announcements: h2a(*instances.announcements.address()),
            node_safe_registry: h2a(*instances.safe_registry.address()),
            ticket_price_oracle: h2a(*instances.price_oracle.address()),
            winning_probability_oracle: h2a(*instances.win_prob_oracle.address()),
            node_stake_v2_factory: h2a(*instances.stake_factory.address()),
            module_implementation: h2a(*instances.module_implementation.address()),
        }
    }
}

/// Multicall3 deployer wallet
pub const MULTICALL3_DEPLOYER: &str = "05f32b3cc3888453ff71b01135b34ff8e41263f2";

/// Amount of ETH to fund MULTICALL3_DEPLOYER, to deployer Multicall3 contract
pub const ETH_VALUE_FOR_MULTICALL3_DEPLOYER: u128 = 100_000_000_000_000_000; // 0.1 (anvil) ETH

/// Contract creation code for mullticall3, as in <https://etherscan.io/address/0xcA11bde05977b3631167028862bE2a173976CA11>
pub const MULTICALL3_CONTRACT_CREATION_CODE: &str = "608060405234801561001057600080fd5b50610ee0806100206000396000f3fe6080604052600436106100f35760003560e01c80634d2301cc1161008a578063a8b0574e11610059578063a8b0574e1461025a578063bce38bd714610275578063c3077fa914610288578063ee82ac5e1461029b57600080fd5b80634d2301cc146101ec57806372425d9d1461022157806382ad56cb1461023457806386d516e81461024757600080fd5b80633408e470116100c65780633408e47014610191578063399542e9146101a45780633e64a696146101c657806342cbb15c146101d957600080fd5b80630f28c97d146100f8578063174dea711461011a578063252dba421461013a57806327e86d6e1461015b575b600080fd5b34801561010457600080fd5b50425b6040519081526020015b60405180910390f35b61012d610128366004610a85565b6102ba565b6040516101119190610bbe565b61014d610148366004610a85565b6104ef565b604051610111929190610bd8565b34801561016757600080fd5b50437fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0140610107565b34801561019d57600080fd5b5046610107565b6101b76101b2366004610c60565b610690565b60405161011193929190610cba565b3480156101d257600080fd5b5048610107565b3480156101e557600080fd5b5043610107565b3480156101f857600080fd5b50610107610207366004610ce2565b73ffffffffffffffffffffffffffffffffffffffff163190565b34801561022d57600080fd5b5044610107565b61012d610242366004610a85565b6106ab565b34801561025357600080fd5b5045610107565b34801561026657600080fd5b50604051418152602001610111565b61012d610283366004610c60565b61085a565b6101b7610296366004610a85565b610a1a565b3480156102a757600080fd5b506101076102b6366004610d18565b4090565b60606000828067ffffffffffffffff8111156102d8576102d8610d31565b60405190808252806020026020018201604052801561031e57816020015b6040805180820190915260008152606060208201528152602001906001900390816102f65790505b5092503660005b8281101561047757600085828151811061034157610341610d60565b6020026020010151905087878381811061035d5761035d610d60565b905060200281019061036f9190610d8f565b6040810135958601959093506103886020850185610ce2565b73ffffffffffffffffffffffffffffffffffffffff16816103ac6060870187610dcd565b6040516103ba929190610e32565b60006040518083038185875af1925050503d80600081146103f7576040519150601f19603f3d011682016040523d82523d6000602084013e6103fc565b606091505b50602080850191909152901515808452908501351761046d577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260846000fd5b5050600101610325565b508234146104e6576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601a60248201527f4d756c746963616c6c333a2076616c7565206d69736d6174636800000000000060448201526064015b60405180910390fd5b50505092915050565b436060828067ffffffffffffffff81111561050c5761050c610d31565b60405190808252806020026020018201604052801561053f57816020015b606081526020019060019003908161052a5790505b5091503660005b8281101561068657600087878381811061056257610562610d60565b90506020028101906105749190610e42565b92506105836020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166105a66020850185610dcd565b6040516105b4929190610e32565b6000604051808303816000865af19150503d80600081146105f1576040519150601f19603f3d011682016040523d82523d6000602084013e6105f6565b606091505b5086848151811061060957610609610d60565b602090810291909101015290508061067d576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b50600101610546565b5050509250929050565b43804060606106a086868661085a565b905093509350939050565b6060818067ffffffffffffffff8111156106c7576106c7610d31565b60405190808252806020026020018201604052801561070d57816020015b6040805180820190915260008152606060208201528152602001906001900390816106e55790505b5091503660005b828110156104e657600084828151811061073057610730610d60565b6020026020010151905086868381811061074c5761074c610d60565b905060200281019061075e9190610e76565b925061076d6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166107906040850185610dcd565b60405161079e929190610e32565b6000604051808303816000865af19150503d80600081146107db576040519150601f19603f3d011682016040523d82523d6000602084013e6107e0565b606091505b506020808401919091529015158083529084013517610851577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260646000fd5b50600101610714565b6060818067ffffffffffffffff81111561087657610876610d31565b6040519080825280602002602001820160405280156108bc57816020015b6040805180820190915260008152606060208201528152602001906001900390816108945790505b5091503660005b82811015610a105760008482815181106108df576108df610d60565b602002602001015190508686838181106108fb576108fb610d60565b905060200281019061090d9190610e42565b925061091c6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff1661093f6020850185610dcd565b60405161094d929190610e32565b6000604051808303816000865af19150503d806000811461098a576040519150601f19603f3d011682016040523d82523d6000602084013e61098f565b606091505b506020830152151581528715610a07578051610a07576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b506001016108c3565b5050509392505050565b6000806060610a2b60018686610690565b919790965090945092505050565b60008083601f840112610a4b57600080fd5b50813567ffffffffffffffff811115610a6357600080fd5b6020830191508360208260051b8501011115610a7e57600080fd5b9250929050565b60008060208385031215610a9857600080fd5b823567ffffffffffffffff811115610aaf57600080fd5b610abb85828601610a39565b90969095509350505050565b6000815180845260005b81811015610aed57602081850181015186830182015201610ad1565b81811115610aff576000602083870101525b50601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0169290920160200192915050565b600082825180855260208086019550808260051b84010181860160005b84811015610bb1578583037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe001895281518051151584528401516040858501819052610b9d81860183610ac7565b9a86019a9450505090830190600101610b4f565b5090979650505050505050565b602081526000610bd16020830184610b32565b9392505050565b600060408201848352602060408185015281855180845260608601915060608160051b870101935082870160005b82811015610c52577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa0888703018452610c40868351610ac7565b95509284019290840190600101610c06565b509398975050505050505050565b600080600060408486031215610c7557600080fd5b83358015158114610c8557600080fd5b9250602084013567ffffffffffffffff811115610ca157600080fd5b610cad86828701610a39565b9497909650939450505050565b838152826020820152606060408201526000610cd96060830184610b32565b95945050505050565b600060208284031215610cf457600080fd5b813573ffffffffffffffffffffffffffffffffffffffff81168114610bd157600080fd5b600060208284031215610d2a57600080fd5b5035919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81833603018112610dc357600080fd5b9190910192915050565b60008083357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe1843603018112610e0257600080fd5b83018035915067ffffffffffffffff821115610e1d57600080fd5b602001915036819003821315610a7e57600080fd5b8183823760009101908152919050565b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc1833603018112610dc357600080fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa1833603018112610dc357600080fdfea2646970667358221220bb2b5c71a328032f97c676ae39a1ec2148d3e5d6f73d95e9b17910152d61f16264736f6c634300080c0033";

/// Default capability permissions, as in smart contract
/// ```text
///     [
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultRedeemTicketSafeFunctionPermisson
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // RESERVED
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultCloseIncomingChannelSafeFunctionPermisson
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultInitiateOutgoingChannelClosureSafeFunctionPermisson
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultFundChannelMultiFunctionPermisson
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultSetCommitmentSafeFunctionPermisson
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultApproveFunctionPermisson
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW  // defaultSendFunctionPermisson
///     ]
/// ```
pub const DEFAULT_CAPABILITY_PERMISSIONS: &str = "010103030303030303030303";

/// Default announcement permissions, where nothing is specified and falls back to the default, as in smart contract
/// ```text
///     [
///       CapabilityPermission.NONE, // defaultRedeemTicketSafeFunctionPermisson
///       CapabilityPermission.NONE, // RESERVED
///       CapabilityPermission.NONE, // defaultCloseIncomingChannelSafeFunctionPermisson
///       CapabilityPermission.NONE, // defaultInitiateOutgoingChannelClosureSafeFunctionPermisson
///       CapabilityPermission.NONE, // defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson
///       CapabilityPermission.NONE, // defaultFundChannelMultiFunctionPermisson
///       CapabilityPermission.NONE, // defaultSetCommitmentSafeFunctionPermisson
///       CapabilityPermission.NONE, // defaultApproveFunctionPermisson
///       CapabilityPermission.NONE  // defaultSendFunctionPermisson
///     ]
/// ```
pub const DEFAULT_ANNOUNCEMENT_PERMISSIONS: &str = "010003000000000000000000";

/// Default node permissions, where nothing is specified and falls back to the default, as in smart contract
/// ```text
///     [
///       CapabilityPermission.NONE, // defaultRedeemTicketSafeFunctionPermisson
///       CapabilityPermission.NONE, // RESERVED
///       CapabilityPermission.NONE, // defaultCloseIncomingChannelSafeFunctionPermisson
///       CapabilityPermission.NONE, // defaultInitiateOutgoingChannelClosureSafeFunctionPermisson
///       CapabilityPermission.NONE, // defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson
///       CapabilityPermission.NONE, // defaultFundChannelMultiFunctionPermisson
///       CapabilityPermission.NONE, // defaultSetCommitmentSafeFunctionPermisson
///       CapabilityPermission.NONE, // defaultApproveFunctionPermisson
///       CapabilityPermission.NONE  // defaultSendFunctionPermisson
///     ]
/// ```
pub const DEFAULT_NODE_PERMISSIONS: &str = "010303000000000000000000";

/// Safe compatibility fallback handler contract deployed for v1.4.0, as in <https://github.com/safe-global/safe-deployments/blob/b707e5e2994e6f86d76ff7ffade0445c4e49ae9a/src/assets/v1.4.0/compatibility_fallback_handler.json>
pub const SAFE_COMPATIBILITYFALLBACKHANDLER_ADDRESS: &str = "2a15DE4410d4c8af0A7b6c12803120f43C42B820";
/// Safe implementation contract deployed for v1.4.1, as in <https://github.com/safe-global/safe-deployments/blob/8c504f44d148f4c898fee02749c88372bae6609a/src/assets/v1.4.1/safe.json>
pub const SAFE_SAFE_ADDRESS: &str = "41675C099F32341bf84BFc5382aF534df5C7461a";
/// Safe proxy factory contract deployed for v1.4.0, <https://github.com/safe-global/safe-deployments/blob/b707e5e2994e6f86d76ff7ffade0445c4e49ae9a/src/assets/v1.4.0/safe_proxy_factory.json>
pub const SAFE_SAFEPROXYFACTORY_ADDRESS: &str = "4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67";
/// Safe multisend contract deployed for v1.4.0,  <https://github.com/safe-global/safe-deployments/blob/b707e5e2994e6f86d76ff7ffade0445c4e49ae9a/src/assets/v1.4.0/multi_send.json>
pub const SAFE_MULTISEND_ADDRESS: &str = "38869bf66a61cF6bDB996A6aE40D5853Fd43B526";

/// Topic hash for `NewHoprNodeStakeModule` event
pub const NEW_HOPR_NODE_STAKE_MODULE_TOPIC: &str = "0x813d391dc490d6c1dae7d3fdd555f337533d1da2c908c6efd36d4cf557a63206";
/// Topic hash for `NewHoprNodeStakeSafe` event
pub const NEW_HOPR_NODE_STAKE_SAFE_TOPIC: &str = "0x8231d169f416b666ae7fa43faa24a18899738075a53f32c97617d173b189e386";

/// Starting point for a linked list in safe contract, as in <https://github.com/safe-global/safe-smart-account/blob/2278f7ccd502878feb5cec21dd6255b82df374b5/contracts/base/OwnerManager.sol#L14>
pub const SENTINEL_OWNERS: &str = "0x0000000000000000000000000000000000000001";
/// Safe transaction type hash, as in <https://github.com/safe-global/safe-smart-account/blob/2278f7ccd502878feb5cec21dd6255b82df374b5/contracts/Safe.sol#L59>
pub const SAFE_TX_TYPEHASH: &str = "bb8310d486368db6bd6f849402fdd73ad53d316b5a4b2644ad6efe0f941286d8";

/// Safe domain separator typehash, as in <https://github.com/safe-global/safe-smart-account/blob/2278f7ccd502878feb5cec21dd6255b82df374b5/contracts/Safe.sol#L54>
pub const DOMAIN_SEPARATOR_TYPEHASH: &str = "47e79534a245952e8b16893a336b85a3d9ea9fa8c573f3d803afb92a79469218";

/// Topic hash for `ExecutionSuccess` event, as in <https://github.com/safe-global/safe-smart-account/blob/2278f7ccd502878feb5cec21dd6255b82df374b5/contracts/interfaces/ISafe.sol#L18>
pub const SAFE_EXECUTION_SUCCESS: &str = "0x442e715f626346e8c54381002da614f62bee8d27386535b2521ec8540898556e";

/// Implement ethers-rs `get_create2_address` function
/// Returns the CREATE2 address of a smart contract as specified in
/// [EIP1014](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1014.md)
///
/// keccak256( 0xff ++ senderAddress ++ salt ++ keccak256(init_code))[12..]
pub fn get_create2_address(from: Address, salt: impl AsRef<[u8]>, init_code: impl AsRef<[u8]>) -> Address {
    let salt = salt.as_ref();
    let init_code_hash = keccak256(init_code.as_ref());

    let mut bytes = Vec::with_capacity(1 + 20 + salt.len() + init_code_hash.len());
    bytes.push(0xff);
    bytes.extend_from_slice(from.as_slice());
    bytes.extend_from_slice(salt);
    bytes.extend_from_slice(&init_code_hash.0);

    let hash = keccak256(bytes);

    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&hash[12..]);
    Address::from(bytes)
}

// define basic safe abi
sol!(
    #![sol(abi)]
    #![sol(rpc)]
    // #[allow(dead_code)]
    contract SafeContract {
        function nonce() view returns (uint256);
        function getTransactionHash( address to, uint256 value, bytes calldata data, uint8 operation, uint256 safeTxGas, uint256 baseGas, uint256 gasPrice, address gasToken, address refundReceiver, uint256 _nonce) public view returns (bytes32);
        function execTransaction(address to, uint256 value, bytes calldata data, uint8 operation, uint256 safeTxGas, uint256 baseGas, uint256 gasPrice, address gasToken, address payable refundReceiver, bytes memory signatures) public returns (bool);
    }
);

lazy_static::lazy_static! {
    static ref MINTER_ROLE_VALUE: primitives::FixedBytes<32> = keccak256("MINTER_ROLE");
}

/// Creates local Anvil instance.
///
/// Used for testing. When block time is given, new blocks are mined periodically.
/// Otherwise, a new block is mined per transaction.
///
/// Uses a fixed mnemonic to make generated accounts deterministic.
pub fn create_anvil(block_time: Option<std::time::Duration>) -> alloy::node_bindings::AnvilInstance {
    let mut anvil = alloy::node_bindings::Anvil::new()
        .mnemonic("gentle wisdom move brush express similar canal dune emotion series because parrot");

    if let Some(bt) = block_time {
        anvil = anvil.block_time(bt.as_secs());
    }

    anvil.spawn()
}

/// Mints specified amount of HOPR tokens to the contract deployer wallet.
/// Assumes that the `hopr_token` contract is associated with a RPC client that also deployed the contract.
/// Returns the block number at which the minting transaction was confirmed.
pub async fn mint_tokens<P, N>(hopr_token: HoprTokenInstance<P, N>, amount: U256) -> ContractResult<Option<u64>>
where
    P: alloy::contract::private::Provider<N>,
    N: alloy::providers::Network,
{
    let deployer = hopr_token
        .provider()
        .get_accounts()
        .await
        .expect("client must have a signer")[0];

    hopr_token
        .grantRole(*MINTER_ROLE_VALUE, deployer)
        .send()
        .await?
        .watch()
        .await?;

    let tx_receipt = hopr_token
        .mint(deployer, amount, Bytes::new(), Bytes::new())
        .send()
        .await?
        .get_receipt()
        .await?;

    Ok(tx_receipt.block_number())
}

/// Creates a transaction that transfers the given `amount` of native tokens to the
/// given destination.
pub fn create_native_transfer<N>(to: hopr_primitive_types::prelude::Address, amount: U256) -> N::TransactionRequest
where
    N: alloy::providers::Network,
{
    N::TransactionRequest::default().with_to(a2h(to)).with_value(amount)
}

/// Funds the given wallet address with specified amount of native tokens and HOPR tokens.
/// These must be present in the client's wallet.
pub async fn fund_node<P, N>(
    node: hopr_primitive_types::prelude::Address,
    native_token: U256,
    hopr_token: U256,
    hopr_token_contract: HoprTokenInstance<P, N>,
) -> ContractResult<()>
where
    P: alloy::contract::private::Provider<N>,
    N: alloy::providers::Network,
{
    let native_transfer_tx = N::TransactionRequest::default()
        .with_to(a2h(node))
        .with_value(native_token);

    // let native_transfer_tx = Eip1559TransactionRequest::new()
    //     .to(NameOrAddress::Address(node.into()))
    //     .value(native_token);

    let provider = hopr_token_contract.provider();

    provider.send_transaction(native_transfer_tx).await?.watch().await?;

    hopr_token_contract
        .transfer(a2h(node), hopr_token)
        .send()
        .await?
        .watch()
        .await?;
    Ok(())
}

/// Funds the channel to the counterparty with the given amount of HOPR tokens.
/// The amount must be present in the wallet of the client.
pub async fn fund_channel<P, N>(
    counterparty: hopr_primitive_types::prelude::Address,
    hopr_token: HoprTokenInstance<P, N>,
    hopr_channels: HoprChannelsInstance<P, N>,
    amount: U256,
) -> ContractResult<()>
where
    P: alloy::contract::private::Provider<N>,
    N: alloy::providers::Network,
{
    hopr_token
        .approve(*hopr_channels.address(), amount)
        .send()
        .await?
        .watch()
        .await?;

    hopr_channels
        .fundChannel(a2h(counterparty), aliases::U96::from(amount))
        .send()
        .await?
        .watch()
        .await?;

    Ok(())
}

/// Funds the channel to the counterparty with the given amount of HOPR tokens, from a different client
/// The amount must be present in the wallet of the client.
pub async fn fund_channel_from_different_client<P, N>(
    counterparty: hopr_primitive_types::prelude::Address,
    hopr_token_address: hopr_primitive_types::prelude::Address,
    hopr_channels_address: hopr_primitive_types::prelude::Address,
    amount: U256,
    new_client: P,
) -> ContractResult<()>
where
    P: alloy::contract::private::Provider<N> + Clone,
    N: alloy::providers::Network,
{
    let hopr_token_with_new_client: HoprTokenInstance<P, N> =
        HoprTokenInstance::new(a2h(hopr_token_address), new_client.clone());
    let hopr_channels_with_new_client = HoprChannelsInstance::new(a2h(hopr_channels_address), new_client.clone());
    hopr_token_with_new_client
        .approve(a2h(hopr_channels_address), amount)
        .send()
        .await?
        .watch()
        .await?;

    hopr_channels_with_new_client
        .fundChannel(a2h(counterparty), aliases::U96::from(amount))
        .send()
        .await?
        .watch()
        .await?;

    Ok(())
}

/// Prepare a safe transaction
pub async fn get_safe_tx<P, N>(
    safe_contract: SafeContractInstance<P, N>,
    target: hopr_primitive_types::prelude::Address,
    inner_tx_data: Bytes,
    wallet: PrivateKeySigner,
) -> anyhow::Result<N::TransactionRequest>
where
    P: alloy::contract::private::Provider<N>,
    N: alloy::providers::Network,
{
    let nonce = safe_contract.nonce().call().await?;

    let data_hash = safe_contract
        .getTransactionHash(
            a2h(target),
            U256::ZERO,
            inner_tx_data.clone(),
            0,
            U256::ZERO,
            U256::ZERO,
            U256::ZERO,
            primitives::Address::default(),
            wallet.address(),
            nonce,
        )
        .call()
        .await?;

    let signed_data_hash = wallet.sign_hash(&data_hash).await?;

    let safe_tx_data = SafeContract::execTransactionCall {
        to: a2h(target),
        value: U256::ZERO,
        data: inner_tx_data,
        operation: 0,
        safeTxGas: U256::ZERO,
        baseGas: U256::ZERO,
        gasPrice: U256::ZERO,
        gasToken: primitives::Address::default(),
        refundReceiver: wallet.address(),
        signatures: Bytes::from(signed_data_hash.as_bytes()),
    }
    .abi_encode();

    // Outer tx payload: execute as safe tx
    let safe_tx = N::TransactionRequest::default()
        .with_to(*safe_contract.address())
        .with_input(safe_tx_data);

    Ok(safe_tx)
}

/// Send a Safe transaction to the module to include node to the module
pub async fn include_node_to_module_by_safe<P, N>(
    provider: P,
    safe_address: hopr_primitive_types::prelude::Address,
    module_address: hopr_primitive_types::prelude::Address,
    node_address: hopr_primitive_types::prelude::Address,
    deployer: &ChainKeypair, // also node address
) -> anyhow::Result<()>
where
    P: alloy::contract::private::Provider<N> + Clone,
    N: alloy::providers::Network,
{
    // prepare default permission for node.
    // - Clearance: Function
    // - TargetType: SEND
    // - TargetPermission: allow all
    // - NodeDefaultPermission: None
    let node_target_permission = format!("{node_address:?}010203000000000000000000");

    // Inner tx payload: include node to the module
    let inner_tx_data = HoprNodeManagementModule::includeNodeCall {
        nodeDefaultTarget: U256::from_str(&node_target_permission)?,
    }
    .abi_encode();

    let safe_contract = SafeContract::new(a2h(safe_address), provider.clone());
    let wallet = PrivateKeySigner::from_slice(deployer.secret().as_ref()).expect("failed to construct wallet");
    let safe_tx = get_safe_tx(safe_contract, module_address, inner_tx_data.into(), wallet).await?;

    provider
        .send_transaction(safe_tx)
        .await
        .map_err(|e| ChainTypesError::ContractError(e.into()))?
        .watch()
        .await
        .map_err(|e| ChainTypesError::ContractError(e.into()))?;

    Ok(())
}

/// Send a Safe transaction to the module to include annoucement to the module
pub async fn add_announcement_as_target<P, N>(
    provider: P,
    safe_address: hopr_primitive_types::prelude::Address,
    module_address: hopr_primitive_types::prelude::Address,
    announcement_contract_address: hopr_primitive_types::prelude::Address,
    deployer: &ChainKeypair, // also node address
) -> ContractResult<()>
where
    P: alloy::contract::private::Provider<N> + Clone,
    N: alloy::providers::Network,
{
    // prepare default permission for announcement.
    // - Clearance: Function
    // - TargetType: TOKEN
    // - TargetPermission: allow all
    // - NodeDefaultPermission: None
    let announcement_target_permission = format!("{announcement_contract_address:?}010003000000000000000000");

    // Inner tx payload: include node to the module
    let inner_tx_data = HoprNodeManagementModule::scopeTargetTokenCall {
        defaultTarget: U256::from_str(&announcement_target_permission).unwrap(),
    }
    .abi_encode();

    let safe_contract = SafeContract::new(a2h(safe_address), provider.clone());
    let wallet = PrivateKeySigner::from_slice(deployer.secret().as_ref()).expect("failed to construct wallet");
    let safe_tx = get_safe_tx(safe_contract, module_address, inner_tx_data.into(), wallet)
        .await
        .unwrap();

    provider.send_transaction(safe_tx).await?.watch().await?;

    Ok(())
}

/// Send a Safe transaction to the token contract, to approve channels on behalf of safe.
pub async fn approve_channel_transfer_from_safe<P, N>(
    provider: P,
    safe_address: hopr_primitive_types::prelude::Address,
    token_address: hopr_primitive_types::prelude::Address,
    channel_address: hopr_primitive_types::prelude::Address,
    deployer: &ChainKeypair, // also node address
) -> ContractResult<()>
where
    P: alloy::contract::private::Provider<N> + Clone,
    N: alloy::providers::Network,
{
    // Inner tx payload: include node to the module
    let inner_tx_data = HoprToken::approveCall {
        spender: a2h(channel_address),
        value: U256::MAX,
    }
    .abi_encode();

    let safe_contract = SafeContract::new(a2h(safe_address), provider.clone());
    let wallet = PrivateKeySigner::from_slice(deployer.secret().as_ref()).expect("failed to construct wallet");
    let safe_tx = get_safe_tx(safe_contract, token_address, inner_tx_data.into(), wallet)
        .await
        .unwrap();

    provider.send_transaction(safe_tx).await?.watch().await?;

    Ok(())
}

/// Deploy a safe instance and a module instance.
///
/// Notice that to complete the on-boarding process,
/// 1) node should be included to the module
/// 2) announcement contract should be a target in the module
///
/// Notice that to be able to open channels, the deployed safe should have HOPR tokens and approve token transfer for
/// Channels contract on the token contract.
///
/// Returns (module address, safe address)
pub async fn deploy_one_safe_one_module_and_setup_for_testing<P>(
    instances: &ContractInstances<P>,
    provider: P,
    deployer: &ChainKeypair,
) -> ContractResult<(
    hopr_primitive_types::prelude::Address,
    hopr_primitive_types::prelude::Address,
)>
where
    P: alloy::providers::Provider + Clone,
{
    // Get deployer address
    let self_address: hopr_primitive_types::prelude::Address = deployer.public().to_address();

    // Check if safe suite has been deployed. If so, skip this step
    let code = provider
        .get_code_at(address!("914d7Fec6aaC8cd542e72Bca78B30650d45643d7"))
        .await?;

    // only deploy contracts when needed
    if code.is_empty() {
        debug!("deploying safe code");
        // Deploy Safe diamond deployment proxy singleton
        let safe_diamond_proxy_address = {
            // Fund Safe singleton deployer 0.01 anvil-eth and deploy Safe singleton
            let tx = TransactionRequest::default()
                .with_to(address!("E1CB04A0fA36DdD16a06ea828007E35e1a3cBC37"))
                .with_value(U256::from(10000000000000000u128));

            provider.send_transaction(tx).await?.watch().await?;

            let tx_receipt = provider
                .send_raw_transaction(&crate::constants::SAFE_DIAMOND_PROXY_SINGLETON_DEPLOY_CODE)
                .await?
                .get_receipt()
                .await?;
            tx_receipt.contract_address().unwrap()
        };
        debug!(%safe_diamond_proxy_address, "Safe diamond proxy singleton");

        // Deploy minimum Safe suite
        {
            // 1. Safe proxy factory deploySafeProxyFactory();
            let _tx_safe_proxy_factory = TransactionRequest::default()
                .with_to(safe_diamond_proxy_address)
                .with_input(crate::constants::SAFE_PROXY_FACTORY_DEPLOY_CODE);

            // 2. Handler: only CompatibilityFallbackHandler and omit TokenCallbackHandler as it's not used now
            let _tx_safe_compatibility_fallback_handler = TransactionRequest::default()
                .with_to(safe_diamond_proxy_address)
                .with_input(crate::constants::SAFE_COMPATIBILITY_FALLBACK_HANDLER_DEPLOY_CODE);
            // 3. Library: only MultiSendCallOnly and omit MultiSendCall
            let _tx_safe_multisend_call_only = TransactionRequest::default()
                .with_to(safe_diamond_proxy_address)
                .with_input(crate::constants::SAFE_MULTISEND_CALL_ONLY_DEPLOY_CODE);
            // 4. Safe singleton deploySafe();
            let _tx_safe_singleton = TransactionRequest::default()
                .with_to(safe_diamond_proxy_address)
                .with_input(crate::constants::SAFE_SINGLETON_DEPLOY_CODE);
            // other omitted libs: SimulateTxAccessor, CreateCall, and SignMessageLib
            // broadcast those transactions
            provider.send_transaction(_tx_safe_proxy_factory).await?.watch().await?;
            provider
                .send_transaction(_tx_safe_compatibility_fallback_handler)
                .await?
                .watch()
                .await?;
            provider
                .send_transaction(_tx_safe_multisend_call_only)
                .await?
                .watch()
                .await?;
            provider.send_transaction(_tx_safe_singleton).await?.watch().await?;
        }
    }

    // create a salt from the nonce
    let curr_nonce = provider
        .get_transaction_count(a2h(self_address))
        .pending()
        //  Some(BlockNumber::Pending.into()))
        .await
        .unwrap();
    debug!(%curr_nonce, "curr_nonce");

    // FIXME: Check if this is the correct way to get the nonce
    let nonce = keccak256((a2h(self_address), U256::from(curr_nonce)).abi_encode_packed());
    let default_target = format!("{:?}010103020202020202020202", instances.channels.address());

    debug!(%self_address, "self_address");
    debug!("nonce {:?}", U256::from_be_bytes(nonce.0).to_string());
    debug!("default_target in bytes {:?}", default_target.bytes());
    debug!("default_target in u256 {:?}", U256::from_str(&default_target).unwrap());

    let typed_tx = HoprNodeStakeFactory::cloneCall {
        moduleSingletonAddress: *instances.module_implementation.address(),
        admins: vec![a2h(self_address)],
        nonce: nonce.into(),
        defaultTarget: U256::from_str(&default_target).unwrap().into(),
    }
    .abi_encode();

    debug!("typed_tx {:?}", typed_tx);

    // deploy one safe and one module
    let instance_deployment_tx_receipt = instances
        .stake_factory
        .clone(
            *instances.module_implementation.address(),
            vec![a2h(self_address)],
            nonce.into(),
            U256::from_str(&default_target).unwrap().into(),
        )
        .send()
        .await?
        .get_receipt()
        .await?;

    // decode logs
    let maybe_module_tx_log =
        instance_deployment_tx_receipt.decoded_log::<HoprNodeStakeFactory::NewHoprNodeStakeModule>();
    let deployed_module_address: primitives::Address = if let Some(module_tx_log) = maybe_module_tx_log {
        let HoprNodeStakeFactory::NewHoprNodeStakeModule { instance, .. } = module_tx_log.data;
        instance
    } else {
        return Err(ContractError::ContractNotDeployed);
    };

    let maybe_safe_tx_log = instance_deployment_tx_receipt.decoded_log::<HoprNodeStakeFactory::NewHoprNodeStakeSafe>();
    let deployed_safe_address: primitives::Address = if let Some(safe_tx_log) = maybe_safe_tx_log {
        let HoprNodeStakeFactory::NewHoprNodeStakeSafe { instance } = safe_tx_log.data;
        instance
    } else {
        return Err(ContractError::ContractNotDeployed);
    };

    debug!("instance_deployment_tx module instance {:?}", deployed_module_address);
    debug!("instance_deployment_tx safe instance {:?}", deployed_safe_address);

    Ok((h2a(deployed_module_address), h2a(deployed_safe_address)))
}
