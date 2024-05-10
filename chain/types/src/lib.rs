//! This crate contains various on-chain related modules and types.

use bindings::hopr_announcements::HoprAnnouncements;
use bindings::hopr_channels::HoprChannels;
use bindings::hopr_dummy_proxy_for_network_registry::HoprDummyProxyForNetworkRegistry;
use bindings::hopr_network_registry::HoprNetworkRegistry;
use bindings::hopr_node_management_module::HoprNodeManagementModule;
use bindings::hopr_node_safe_registry::HoprNodeSafeRegistry;
use bindings::hopr_node_stake_factory::HoprNodeStakeFactory;
use bindings::hopr_safe_proxy_for_network_registry::HoprSafeProxyForNetworkRegistry;
use bindings::hopr_ticket_price_oracle::HoprTicketPriceOracle;
use bindings::hopr_token::HoprToken;
use ethers::abi::Token;
use ethers::prelude::*;
use hex_literal::hex;
use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
use hopr_primitive_types::primitives::Address;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

pub mod actions;
pub mod chain_events;

// Various (mostly testing related) utility functions
pub mod utils;

pub use ethers::core::types::transaction::eip2718::TypedTransaction;

/// Short-hand for creating new EIP1559 transaction object.
pub fn create_eip1559_transaction() -> TypedTransaction {
    TypedTransaction::Eip1559(Eip1559TransactionRequest::new())
}

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
    /// Stake factory contract
    pub stake_factory: Address,
    /// Node management module contract (can be zero if safe is not used)
    pub module_implementation: Address,
}

#[derive(Debug, Clone)]
pub enum NetworkRegistryProxy<M: Middleware> {
    Dummy(HoprDummyProxyForNetworkRegistry<M>),
    Safe(HoprSafeProxyForNetworkRegistry<M>),
}

impl<M: Middleware> NetworkRegistryProxy<M> {
    pub fn address(&self) -> Address {
        match self {
            NetworkRegistryProxy::Dummy(c) => c.address().into(),
            NetworkRegistryProxy::Safe(c) => c.address().into(),
        }
    }
}

/// Holds instances to contracts.
#[derive(Debug)]
pub struct ContractInstances<M: Middleware> {
    pub token: HoprToken<M>,
    pub channels: HoprChannels<M>,
    pub announcements: HoprAnnouncements<M>,
    pub network_registry: HoprNetworkRegistry<M>,
    pub network_registry_proxy: NetworkRegistryProxy<M>,
    pub safe_registry: HoprNodeSafeRegistry<M>,
    pub price_oracle: HoprTicketPriceOracle<M>,
    pub stake_factory: HoprNodeStakeFactory<M>,
    pub module_implementation: HoprNodeManagementModule<M>,
}

impl<M: Middleware> Clone for ContractInstances<M> {
    fn clone(&self) -> Self {
        // Requires manual clone implementation, because `M` is usually never `Clone`
        let client = self.token.client();
        Self {
            token: HoprToken::new(self.token.address(), client.clone()),
            channels: HoprChannels::new(self.channels.address(), client.clone()),
            announcements: HoprAnnouncements::new(self.announcements.address(), client.clone()),
            network_registry: HoprNetworkRegistry::new(self.network_registry.address(), client.clone()),
            network_registry_proxy: match &self.network_registry_proxy {
                NetworkRegistryProxy::Dummy(nr) => {
                    NetworkRegistryProxy::Dummy(HoprDummyProxyForNetworkRegistry::new(nr.address(), client.clone()))
                }
                NetworkRegistryProxy::Safe(nr) => {
                    NetworkRegistryProxy::Safe(HoprSafeProxyForNetworkRegistry::new(nr.address(), client.clone()))
                }
            },
            safe_registry: HoprNodeSafeRegistry::new(self.safe_registry.address(), client.clone()),
            price_oracle: HoprTicketPriceOracle::new(self.price_oracle.address(), client.clone()),
            stake_factory: HoprNodeStakeFactory::new(self.stake_factory.address(), client.clone()),
            module_implementation: HoprNodeManagementModule::new(self.module_implementation.address(), client.clone()),
        }
    }
}

impl<M: Middleware> ContractInstances<M> {
    pub fn new(contract_addresses: &ContractAddresses, provider: Arc<M>, use_dummy_nr: bool) -> Self {
        Self {
            token: HoprToken::new(contract_addresses.token, provider.clone()),
            channels: HoprChannels::new(contract_addresses.channels, provider.clone()),
            announcements: HoprAnnouncements::new(contract_addresses.announcements, provider.clone()),
            network_registry: HoprNetworkRegistry::new(contract_addresses.network_registry, provider.clone()),
            network_registry_proxy: if use_dummy_nr {
                NetworkRegistryProxy::Dummy(HoprDummyProxyForNetworkRegistry::new(
                    contract_addresses.network_registry_proxy,
                    provider.clone(),
                ))
            } else {
                NetworkRegistryProxy::Safe(HoprSafeProxyForNetworkRegistry::new(
                    contract_addresses.network_registry_proxy,
                    provider.clone(),
                ))
            },
            safe_registry: HoprNodeSafeRegistry::new(contract_addresses.safe_registry, provider.clone()),
            price_oracle: HoprTicketPriceOracle::new(contract_addresses.price_oracle, provider.clone()),
            stake_factory: HoprNodeStakeFactory::new(contract_addresses.stake_factory, provider.clone()),
            module_implementation: HoprNodeManagementModule::new(
                contract_addresses.module_implementation,
                provider.clone(),
            ),
        }
    }

    /// Deploys testing environment (with dummy network registry proxy) via the given provider.
    pub async fn deploy_for_testing(provider: Arc<M>, deployer: &ChainKeypair) -> Result<Self, ContractError<M>> {
        {
            // Fund 1820 deployer and deploy ERC1820Registry
            let mut tx = Eip1559TransactionRequest::new();
            tx = tx.to(H160::from_str(crate::utils::ERC_1820_DEPLOYER).unwrap());
            tx = tx.value(crate::utils::ETH_VALUE_FOR_ERC1820_DEPLOYER);

            provider
                .send_transaction(tx, None)
                .await
                .map_err(|e| ContractError::MiddlewareError { e })?
                .await?;

            provider.send_raw_transaction(
                hex!("f90a388085174876e800830c35008080b909e5608060405234801561001057600080fd5b506109c5806100206000396000f3fe608060405234801561001057600080fd5b50600436106100a5576000357c010000000000000000000000000000000000000000000000000000000090048063a41e7d5111610078578063a41e7d51146101d4578063aabbb8ca1461020a578063b705676514610236578063f712f3e814610280576100a5565b806329965a1d146100aa5780633d584063146100e25780635df8122f1461012457806365ba36c114610152575b600080fd5b6100e0600480360360608110156100c057600080fd5b50600160a060020a038135811691602081013591604090910135166102b6565b005b610108600480360360208110156100f857600080fd5b5035600160a060020a0316610570565b60408051600160a060020a039092168252519081900360200190f35b6100e06004803603604081101561013a57600080fd5b50600160a060020a03813581169160200135166105bc565b6101c26004803603602081101561016857600080fd5b81019060208101813564010000000081111561018357600080fd5b82018360208201111561019557600080fd5b803590602001918460018302840111640100000000831117156101b757600080fd5b5090925090506106b3565b60408051918252519081900360200190f35b6100e0600480360360408110156101ea57600080fd5b508035600160a060020a03169060200135600160e060020a0319166106ee565b6101086004803603604081101561022057600080fd5b50600160a060020a038135169060200135610778565b61026c6004803603604081101561024c57600080fd5b508035600160a060020a03169060200135600160e060020a0319166107ef565b604080519115158252519081900360200190f35b61026c6004803603604081101561029657600080fd5b508035600160a060020a03169060200135600160e060020a0319166108aa565b6000600160a060020a038416156102cd57836102cf565b335b9050336102db82610570565b600160a060020a031614610339576040805160e560020a62461bcd02815260206004820152600f60248201527f4e6f7420746865206d616e616765720000000000000000000000000000000000604482015290519081900360640190fd5b6103428361092a565b15610397576040805160e560020a62461bcd02815260206004820152601a60248201527f4d757374206e6f7420626520616e204552433136352068617368000000000000604482015290519081900360640190fd5b600160a060020a038216158015906103b85750600160a060020a0382163314155b156104ff5760405160200180807f455243313832305f4143434550545f4d4147494300000000000000000000000081525060140190506040516020818303038152906040528051906020012082600160a060020a031663249cb3fa85846040518363ffffffff167c01000000000000000000000000000000000000000000000000000000000281526004018083815260200182600160a060020a0316600160a060020a031681526020019250505060206040518083038186803b15801561047e57600080fd5b505afa158015610492573d6000803e3d6000fd5b505050506040513d60208110156104a857600080fd5b5051146104ff576040805160e560020a62461bcd02815260206004820181905260248201527f446f6573206e6f7420696d706c656d656e742074686520696e74657266616365604482015290519081900360640190fd5b600160a060020a03818116600081815260208181526040808320888452909152808220805473ffffffffffffffffffffffffffffffffffffffff19169487169485179055518692917f93baa6efbd2244243bfee6ce4cfdd1d04fc4c0e9a786abd3a41313bd352db15391a450505050565b600160a060020a03818116600090815260016020526040812054909116151561059a5750806105b7565b50600160a060020a03808216600090815260016020526040902054165b919050565b336105c683610570565b600160a060020a031614610624576040805160e560020a62461bcd02815260206004820152600f60248201527f4e6f7420746865206d616e616765720000000000000000000000000000000000604482015290519081900360640190fd5b81600160a060020a031681600160a060020a0316146106435780610646565b60005b600160a060020a03838116600081815260016020526040808220805473ffffffffffffffffffffffffffffffffffffffff19169585169590951790945592519184169290917f605c2dbf762e5f7d60a546d42e7205dcb1b011ebc62a61736a57c9089d3a43509190a35050565b600082826040516020018083838082843780830192505050925050506040516020818303038152906040528051906020012090505b92915050565b6106f882826107ef565b610703576000610705565b815b600160a060020a03928316600081815260208181526040808320600160e060020a031996909616808452958252808320805473ffffffffffffffffffffffffffffffffffffffff19169590971694909417909555908152600284528181209281529190925220805460ff19166001179055565b600080600160a060020a038416156107905783610792565b335b905061079d8361092a565b156107c357826107ad82826108aa565b6107b85760006107ba565b815b925050506106e8565b600160a060020a0390811660009081526020818152604080832086845290915290205416905092915050565b6000808061081d857f01ffc9a70000000000000000000000000000000000000000000000000000000061094c565b909250905081158061082d575080155b1561083d576000925050506106e8565b61084f85600160e060020a031961094c565b909250905081158061086057508015155b15610870576000925050506106e8565b61087a858561094c565b909250905060018214801561088f5750806001145b1561089f576001925050506106e8565b506000949350505050565b600160a060020a0382166000908152600260209081526040808320600160e060020a03198516845290915281205460ff1615156108f2576108eb83836107ef565b90506106e8565b50600160a060020a03808316600081815260208181526040808320600160e060020a0319871684529091529020549091161492915050565b7bffffffffffffffffffffffffffffffffffffffffffffffffffffffff161590565b6040517f01ffc9a7000000000000000000000000000000000000000000000000000000008082526004820183905260009182919060208160248189617530fa90519096909550935050505056fea165627a7a72305820377f4a2d4301ede9949f163f319021a6e9c687c292a5e2b2c4734c126b524e6c00291ba01820182018201820182018201820182018201820182018201820182018201820a01820182018201820182018201820182018201820182018201820182018201820")
                    .into()).await.map_err(|e| ContractError::MiddlewareError {e})?.await?;
        }

        // Get deployer address
        let self_address: types::Address = deployer.public().to_address().into();

        let stake_factory = HoprNodeStakeFactory::deploy(provider.clone(), ())?.send().await?;
        let module_implementation = HoprNodeManagementModule::deploy(provider.clone(), ())?.send().await?;
        let safe_registry = HoprNodeSafeRegistry::deploy(provider.clone(), ())?.send().await?;
        let price_oracle = HoprTicketPriceOracle::deploy(
            provider.clone(),
            (self_address, ethers::types::U256::from(100000000000000000_u128)),
        )?
        .send()
        .await?;
        let token = HoprToken::deploy(provider.clone(), ())?.send().await?;
        let network_registry_proxy = HoprDummyProxyForNetworkRegistry::deploy(provider.clone(), self_address)?
            .send()
            .await?;
        let channels = HoprChannels::deploy(
            provider.clone(),
            Token::Tuple(vec![
                Token::Address(token.address()),
                Token::Uint(1_u32.into()),
                Token::Address(safe_registry.address()),
            ]),
        )?
        .send()
        .await?;
        let announcements = HoprAnnouncements::deploy(provider.clone(), Token::Address(safe_registry.address()))?
            .send()
            .await?;
        let network_registry = HoprNetworkRegistry::deploy(
            provider.clone(),
            (
                ethers::types::Address::from(network_registry_proxy.address()),
                self_address,
                self_address,
            ),
        )?
        .send()
        .await?;

        // Disable network registry in local environment
        network_registry.disable_registry().send().await?.await?;

        Ok(Self {
            token,
            channels,
            announcements,
            network_registry,
            network_registry_proxy: NetworkRegistryProxy::Dummy(network_registry_proxy),
            safe_registry,
            price_oracle,
            stake_factory,
            module_implementation,
        })
    }
}

impl<M: Middleware> From<&ContractInstances<M>> for ContractAddresses {
    fn from(value: &ContractInstances<M>) -> Self {
        Self {
            token: value.token.address().into(),
            channels: value.channels.address().into(),
            announcements: value.announcements.address().into(),
            network_registry: value.network_registry.address().into(),
            network_registry_proxy: value.network_registry_proxy.address(),
            safe_registry: value.safe_registry.address().into(),
            price_oracle: value.price_oracle.address().into(),
            stake_factory: value.stake_factory.address().into(),
            module_implementation: value.module_implementation.address().into(),
        }
    }
}

/// Creates local Anvil instance.
///
/// Used for testing. When block time is given, new blocks are mined periodically.
/// Otherwise, a new block is mined per transaction.
pub fn create_anvil(block_time: Option<std::time::Duration>) -> ethers::utils::AnvilInstance {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = std::path::Path::new(std::str::from_utf8(&output).unwrap().trim());
    let workspace_dir = cargo_path.parent().unwrap().to_path_buf();

    let mut anvil = ethers::utils::Anvil::new().path(workspace_dir.join(".foundry/bin/anvil"));

    if let Some(bt) = block_time {
        anvil = anvil.block_time(bt.as_secs());
    }

    anvil.spawn()
}
