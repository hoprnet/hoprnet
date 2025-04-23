//! This crate contains various on-chain related modules and types.

use alloy::{contract::Result as ContractResult, network::TransactionBuilder, primitives};
use constants::{ERC_1820_DEPLOYER, ERC_1820_REGISTRY_DEPLOY_CODE, ETH_VALUE_FOR_ERC1820_DEPLOYER};
use serde::{Deserialize, Serialize};
use utils::{address_from_alloy_primitive, address_to_alloy_primitive};

use hopr_bindings::{
    hoprannouncements::HoprAnnouncements::{self, HoprAnnouncementsInstance},
    hoprchannels::HoprChannels::{self, HoprChannelsInstance},
    hoprdummyproxyfornetworkregistry::HoprDummyProxyForNetworkRegistry::{
        self, HoprDummyProxyForNetworkRegistryInstance,
    },
    hoprnetworkregistry::HoprNetworkRegistry::{self, HoprNetworkRegistryInstance},
    hoprnodemanagementmodule::HoprNodeManagementModule::{self, HoprNodeManagementModuleInstance},
    hoprnodesaferegistry::HoprNodeSafeRegistry::{self, HoprNodeSafeRegistryInstance},
    hoprnodestakefactory::HoprNodeStakeFactory::{self, HoprNodeStakeFactoryInstance},
    hoprsafeproxyfornetworkregistry::HoprSafeProxyForNetworkRegistry::HoprSafeProxyForNetworkRegistryInstance,
    hoprticketpriceoracle::HoprTicketPriceOracle::{self, HoprTicketPriceOracleInstance},
    hoprtoken::HoprToken::{self, HoprTokenInstance},
    hoprwinningprobabilityoracle::HoprWinningProbabilityOracle::{self, HoprWinningProbabilityOracleInstance},
};

// use ethers::abi::Token;
use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
use hopr_primitive_types::primitives::Address;

pub mod actions;
pub mod chain_events;
pub mod constants;
pub mod errors;
// Various (mostly testing related) utility functions
pub mod utils;

/// Holds addresses of all smart contracts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ContractAddresses {
    /// Token contract
    pub token: Address,
    /// Channels contract
    pub channels: Address,
    /// Announcements contract
    pub announcements: Address,
    /// Network registry contract
    pub network_registry: Address,
    /// Network registry proxy contract
    pub network_registry_proxy: Address,
    /// Safe registry contract
    pub safe_registry: Address,
    /// Price oracle contract
    pub price_oracle: Address,
    /// Minimum ticket winning probability contract
    pub win_prob_oracle: Address,
    /// Stake factory contract
    pub stake_factory: Address,
    /// Node management module contract (can be zero if safe is not used)
    pub module_implementation: Address,
}

#[derive(Debug, Clone)]
pub enum NetworkRegistryProxy<T, P, N> {
    Dummy(HoprDummyProxyForNetworkRegistryInstance<T, P, N>),
    Safe(HoprSafeProxyForNetworkRegistryInstance<T, P, N>),
}

impl<T, P, N> NetworkRegistryProxy<T, P, N>
where
    T: alloy::contract::private::Transport + Clone,
    P: alloy::contract::private::Provider<T, N>,
    N: alloy::providers::Network,
{
    pub fn address(&self) -> Address {
        match self {
            NetworkRegistryProxy::Dummy(c) => address_from_alloy_primitive(*c.address()),
            NetworkRegistryProxy::Safe(c) => address_from_alloy_primitive(*c.address()),
        }
    }
}

/// Holds instances to contracts.
#[derive(Debug)]
pub struct ContractInstances<T, P, N> {
    pub token: HoprTokenInstance<T, P, N>,
    pub channels: HoprChannelsInstance<T, P, N>,
    pub announcements: HoprAnnouncementsInstance<T, P, N>,
    pub network_registry: HoprNetworkRegistryInstance<T, P, N>,
    pub network_registry_proxy: NetworkRegistryProxy<T, P, N>,
    pub safe_registry: HoprNodeSafeRegistryInstance<T, P, N>,
    pub price_oracle: HoprTicketPriceOracleInstance<T, P, N>,
    pub win_prob_oracle: HoprWinningProbabilityOracleInstance<T, P, N>,
    pub stake_factory: HoprNodeStakeFactoryInstance<T, P, N>,
    pub module_implementation: HoprNodeManagementModuleInstance<T, P, N>,
}

/// FIXME: This implementation would still require Provider to be Clone.
impl<T, P, N> ContractInstances<T, P, N>
where
    T: alloy::contract::private::Transport + Clone,
    P: alloy::contract::private::Provider<T, N> + Clone,
    N: alloy::providers::Network,
    // where
    //     T: alloy::contract::private::Transport,
    //     P: alloy::contract::private::Provider<T, alloy::network::Ethereum> + Clone,
{
    pub fn new(contract_addresses: &ContractAddresses, provider: P, use_dummy_nr: bool) -> Self {
        Self {
            token: HoprTokenInstance::new(address_to_alloy_primitive(contract_addresses.token), provider.clone()),
            channels: HoprChannelsInstance::new(
                address_to_alloy_primitive(contract_addresses.channels),
                provider.clone(),
            ),
            announcements: HoprAnnouncementsInstance::new(
                address_to_alloy_primitive(contract_addresses.announcements),
                provider.clone(),
            ),
            network_registry: HoprNetworkRegistryInstance::new(
                address_to_alloy_primitive(contract_addresses.network_registry),
                provider.clone(),
            ),
            network_registry_proxy: if use_dummy_nr {
                NetworkRegistryProxy::Dummy(HoprDummyProxyForNetworkRegistryInstance::new(
                    address_to_alloy_primitive(contract_addresses.network_registry_proxy),
                    provider.clone(),
                ))
            } else {
                NetworkRegistryProxy::Safe(HoprSafeProxyForNetworkRegistryInstance::new(
                    address_to_alloy_primitive(contract_addresses.network_registry_proxy),
                    provider.clone(),
                ))
            },
            safe_registry: HoprNodeSafeRegistryInstance::new(
                address_to_alloy_primitive(contract_addresses.safe_registry),
                provider.clone(),
            ),
            price_oracle: HoprTicketPriceOracleInstance::new(
                address_to_alloy_primitive(contract_addresses.price_oracle),
                provider.clone(),
            ),
            win_prob_oracle: HoprWinningProbabilityOracleInstance::new(
                address_to_alloy_primitive(contract_addresses.win_prob_oracle),
                provider.clone(),
            ),
            stake_factory: HoprNodeStakeFactoryInstance::new(
                address_to_alloy_primitive(contract_addresses.stake_factory),
                provider.clone(),
            ),
            module_implementation: HoprNodeManagementModuleInstance::new(
                address_to_alloy_primitive(contract_addresses.module_implementation),
                provider.clone(),
            ),
        }
    }

    /// Deploys testing environment (with dummy network registry proxy) via the given provider.
    pub async fn deploy_for_testing(provider: P, deployer: &ChainKeypair) -> ContractResult<Self> {
        {
            // Fund 1820 deployer and deploy ERC1820Registry
            let tx = N::TransactionRequest::default()
                .with_to(ERC_1820_DEPLOYER)
                .with_value(ETH_VALUE_FOR_ERC1820_DEPLOYER);

            // Sequentially executing the following transactions:
            // 1. Fund the deployer wallet
            provider.send_transaction(tx.clone()).await?.watch().await?;
            // 2. Use the fundedd deployer wallet to deploy ERC1820Registry with a signed txn
            provider
                .send_raw_transaction(&ERC_1820_REGISTRY_DEPLOY_CODE)
                .await?
                .watch()
                .await?;
        }

        // Get deployer address
        let self_address = address_to_alloy_primitive(deployer.public().to_address());

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
            primitives::aliases::U56::from(0xFFFFFFFFFFFFFF_u64), // 0xFFFFFFFFFFFFFF in hex or 72057594037927935 in decimal values
        )
        .await?;
        let token = HoprToken::deploy(provider.clone()).await?;
        let network_registry_proxy = HoprDummyProxyForNetworkRegistry::deploy(provider.clone(), self_address).await?;
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
        let network_registry = HoprNetworkRegistry::deploy(
            provider.clone(),
            primitives::Address::from(network_registry_proxy.address().as_ref()),
            self_address,
            self_address,
        )
        .await?;

        // Disable network registry in local environment and wait for its confirmation
        network_registry.disableRegistry().send().await?.watch().await?;

        Ok(Self {
            token,
            channels,
            announcements,
            network_registry,
            network_registry_proxy: NetworkRegistryProxy::Dummy(network_registry_proxy),
            safe_registry,
            price_oracle,
            win_prob_oracle,
            stake_factory,
            module_implementation,
        })
    }
}

impl<T, P, N> From<&ContractInstances<T, P, N>> for ContractAddresses
where
    T: alloy::contract::private::Transport + Clone,
    P: alloy::contract::private::Provider<T, N>,
    N: alloy::providers::Network,
{
    fn from(instances: &ContractInstances<T, P, N>) -> Self {
        Self {
            token: address_from_alloy_primitive(*instances.token.address()),
            channels: address_from_alloy_primitive(*instances.channels.address()),
            announcements: address_from_alloy_primitive(*instances.announcements.address()),
            network_registry: address_from_alloy_primitive(*instances.network_registry.address()),
            network_registry_proxy: instances.network_registry_proxy.address(),
            safe_registry: address_from_alloy_primitive(*instances.safe_registry.address()),
            price_oracle: address_from_alloy_primitive(*instances.price_oracle.address()),
            win_prob_oracle: address_from_alloy_primitive(*instances.win_prob_oracle.address()),
            stake_factory: address_from_alloy_primitive(*instances.stake_factory.address()),
            module_implementation: address_from_alloy_primitive(*instances.module_implementation.address()),
        }
    }
}
