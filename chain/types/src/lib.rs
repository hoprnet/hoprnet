//! This crate contains various on-chain related modules and types.
use alloy::{
    contract::Result as ContractResult, network::TransactionBuilder, primitives, providers::MULTICALL3_ADDRESS,
    rpc::types::TransactionRequest,
};
use constants::{ERC_1820_DEPLOYER, ERC_1820_REGISTRY_DEPLOY_CODE, ETH_VALUE_FOR_ERC1820_DEPLOYER};
use hopr_bindings::{
    hoprannouncements::HoprAnnouncements::{self, HoprAnnouncementsInstance},
    hoprchannels::HoprChannels::{self, HoprChannelsInstance},
    hoprdummyproxyfornetworkregistry::HoprDummyProxyForNetworkRegistry::{
        self, HoprDummyProxyForNetworkRegistryInstance,
    },
    hoprnetworkregistry::HoprNetworkRegistry::{self},
    hoprnodemanagementmodule::HoprNodeManagementModule::{self, HoprNodeManagementModuleInstance},
    hoprnodesaferegistry::HoprNodeSafeRegistry::{self, HoprNodeSafeRegistryInstance},
    hoprnodestakefactory::HoprNodeStakeFactory::{self, HoprNodeStakeFactoryInstance},
    hoprsafeproxyfornetworkregistry::HoprSafeProxyForNetworkRegistry::{self, HoprSafeProxyForNetworkRegistryInstance},
    hoprticketpriceoracle::HoprTicketPriceOracle::{self, HoprTicketPriceOracleInstance},
    hoprtoken::HoprToken::{self, HoprTokenInstance},
    hoprwinningprobabilityoracle::HoprWinningProbabilityOracle::{self, HoprWinningProbabilityOracleInstance},
};
use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
use hopr_primitive_types::primitives::Address;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::constants::{ETH_VALUE_FOR_MULTICALL3_DEPLOYER, MULTICALL3_DEPLOY_CODE, MULTICALL3_DEPLOYER};

pub mod chain_events;
pub mod constants;
pub mod errors;
pub mod payload;
// Various (mostly testing related) utility functions
pub mod utils;

pub mod prelude {
    pub use super::{
        ContractAddresses, ContractInstances,
        chain_events::ChainEvent,
        payload::{BasicPayloadGenerator, PayloadGenerator, SafePayloadGenerator, SignableTransaction},
    };
}

/// Holds addresses of all smart contracts.
#[serde_with::serde_as]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ContractAddresses {
    /// Token contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub token: Address,
    /// Channels contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub channels: Address,
    /// Announcements contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub announcements: Address,
    /// Safe registry contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub node_safe_registry: Address,
    /// Price oracle contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub ticket_price_oracle: Address,
    /// Minimum ticket winning probability contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub winning_probability_oracle: Address,
    /// Stake factory contract
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub node_stake_v2_factory: Address,
    /// Node management module contract (can be zero if safe is not used)
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub module_implementation: Address,
}

// Used instead of From implementation to avoid alloy being a dependency of the primitive crates
pub fn h2a(h: alloy::primitives::Address) -> Address {
    Address::from(h.0.0)
}

pub fn a2h(a: Address) -> alloy::primitives::Address {
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
    pub fn address(&self) -> Address {
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
                    .with_to(MULTICALL3_DEPLOYER)
                    .with_value(ETH_VALUE_FOR_MULTICALL3_DEPLOYER);
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
