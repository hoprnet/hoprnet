//! This module contains arguments and functions to manage safe and module.
//! [SafeModuleSubcommands] defines three subcommands: create, move, and migrate.
//! - [SafeModuleSubcommands::Create] creates staking wallets (safe and node management module)
//! and execute necessary on-chain transactions to setup a HOPR node.
//! Detailed breakdown of the steps:
//!     - create a Safe proxy instance and HOPR node management module proxy instance
//!     - include nodes configure default permissions on the created module proxy
//!     - fund the node and Safe with some native tokens and HOPR tokens respectively
//!     - approve HOPR tokens to be transferred from the Safe proxy instaces by Channels contract
//!     - Use manager wallet to add nodes and staking safes to the Network Registry contract
//! - [SafeModuleSubcommands::Move] moves a node from to an existing Safe.
//! Note that the Safe should has a node management module attached and configured.
//! Note that the admin key of the old and new safes are the same. This command does not support
//! moving nodes to safes controled by a different admin key.
//! Note that all the safes involved (old and new) should have a threshold of 1
//! Detailed breakdown of the steps:
//!     - use old safes to deregister nodes from Node-safe registry
//!     - use the new safe to include nodes to the module
//!     - use manager wallet to deregister nodes from the network registry
//!     - use maanger wallet to register nodes with new safes to the network regsitry
//! - [SafeModuleSubcommands::Migrate] migrates a node to a different network.
//! It performs the following steps:
//!     - add the Channel contract of the new network to the module as target and set default permissions.
//!     - add the Announcement contract as target to the module
//!     - approve HOPR tokens of the Safe proxy to be transferred by the new Channels contract
//!     - Use the manager wlalet to add nodes and Safes to the Network Registry contract of the new network.
use crate::{
    environment_config::NetworkProviderArgs,
    identity::{IdentityFileArgs, PrivateKeyArgs},
    methods::{
        deploy_safe_module_with_targets_and_nodes, deregister_nodes_from_node_safe_registry_and_remove_from_module,
        include_nodes_to_module, migrate_nodes, safe_singleton,
    },
    utils::{Cmd, HelperErrors},
};
use bindings::{hopr_node_safe_registry::HoprNodeSafeRegistry, hopr_node_stake_factory::HoprNodeStakeFactory};
use clap::{builder::RangedU64ValueParser, Parser};
use ethers::{
    types::{H160, U256},
    utils::parse_units,
};
use hopr_crypto_types::keypairs::Keypair;
use log::info;
use safe_singleton::SafeSingleton;
use std::str::FromStr;

/// CLI arguments for `hopli safe-module`
#[derive(Clone, Debug, Parser)]
pub enum SafeModuleSubcommands {
    /// Create safe and module proxy if nothing exists
    #[command(visible_alias = "cr")]
    Create {
        /// Network name, contracts config file root, and customized provider, if available
        #[clap(flatten)]
        network_provider: NetworkProviderArgs,

        /// Arguments to locate identity file(s) of HOPR node(s)
        #[command(flatten)]
        local_identity: Option<IdentityFileArgs>,

        /// node addresses
        #[clap(
            help = "Comma separated node Ethereum addresses",
            long,
            short = 'o',
            default_value = None
        )]
        node_address: Option<String>,

        /// admin addresses
        #[clap(
            help = "Comma separated node Ethereum addresses",
            long,
            short = 'a',
            default_value = None
        )]
        admin_address: Option<String>,

        /// Threshold for the generated safe
        #[clap(
            help = "Threshold for the generated safe, e.g. 1",
            long,
            short,
            value_parser = RangedU64ValueParser::<u32>::new().range(1..),
            default_value_t = 1
        )]
        threshold: u32,

        /// Allowance of the channel contract to manage HOPR tokens on behalf of deployed safe
        #[clap(
            help = "Provide the allowance of the channel contract to manage HOPR tokens on behalf of deployed safe. Value in ether, e.g. 10",
            long,
            short = 'l',
            value_parser = clap::value_parser!(f64),
        )]
        allowance: Option<f64>,

        /// Access to the private key, of which the wallet either contains sufficient assets
        /// as the source of funds or it can mint necessary tokens
        /// This wallet is also the manager of Network Registry contract
        #[command(flatten)]
        private_key: PrivateKeyArgs,
    },

    /// Migrate safe and module to a new network
    #[command(visible_alias = "mg")]
    Migrate {
        /// Network name, contracts config file root, and customized provider, if available
        #[clap(flatten)]
        network_provider: NetworkProviderArgs,

        /// Arguments to locate identity file(s) of HOPR node(s)
        #[command(flatten)]
        local_identity: Option<IdentityFileArgs>,

        /// node addresses
        #[clap(
             help = "Comma separated node Ethereum addresses",
             long,
             short = 'o',
             default_value = None
         )]
        node_address: Option<String>,

        /// safe address that the nodes move to
        #[clap(help = "New managing safe to which all the nodes move", long, short = 's')]
        safe_address: String,

        /// module address that the nodes move to
        #[clap(help = "New managing module to which all the nodes move", long, short = 'm')]
        module_address: String,

        /// Allowance of the channel contract to manage HOPR tokens on behalf of deployed safe
        #[clap(
            help = "Provide the allowance of the channel contract to manage HOPR tokens on behalf of deployed safe. Value in ether, e.g. 10",
            long,
            short = 'l',
            value_parser = clap::value_parser!(f64),
        )]
        allowance: Option<f64>,

        /// Access to the private key, of which the wallet either contains sufficient assets
        /// as the source of funds or it can mint necessary tokens
        /// This wallet is also the manager of Network Registry contract
        #[command(flatten)]
        private_key: PrivateKeyArgs,
    },

    /// Move nodes to one single safe and module pair
    #[command(visible_alias = "mv")]
    Move {
        /// Network name, contracts config file root, and customized provider, if available
        #[clap(flatten)]
        network_provider: NetworkProviderArgs,

        /// Arguments to locate identity file(s) of HOPR node(s)
        #[command(flatten)]
        local_identity: Option<IdentityFileArgs>,

        /// node addresses
        #[clap(
             help = "Comma separated node Ethereum addresses",
             long,
             short = 'o',
             default_value = None
         )]
        node_address: Option<String>,

        /// old module addresses
        #[clap(help = "Comma separated old module addresses", long, short = 'u')]
        old_module_address: String,

        /// safe address that the nodes move to
        #[clap(help = "New managing safe to which all the nodes move", long, short = 's')]
        new_safe_address: String,

        /// module address that the nodes move to
        #[clap(help = "New managing module to which all the nodes move", long, short = 'm')]
        new_module_address: String,

        /// Access to the private key, of which the wallet either contains sufficient assets
        /// as the source of funds or it can mint necessary tokens
        /// This wallet is also the manager of Network Registry contract
        #[command(flatten)]
        private_key: PrivateKeyArgs,
    },
}

impl SafeModuleSubcommands {
    /// Execute the command, which quickly create necessary staking wallets
    /// and execute necessary on-chain transactions to setup a HOPR node.
    ///
    /// 1. Create a safe instance and a node management module instance:
    /// 2. Set default permissions for the module
    /// 3. Include node as a member with restricted permission on sending assets
    pub async fn execute_safe_module_creation(
        network_provider: NetworkProviderArgs,
        local_identity: Option<IdentityFileArgs>,
        node_address: Option<String>,
        admin_address: Option<String>,
        threshold: u32,
        allowance: Option<f64>,
        private_key: PrivateKeyArgs,
    ) -> Result<(), HelperErrors> {
        // read all the node addresses
        let mut node_eth_addresses: Vec<H160> = Vec::new();
        if let Some(addresses) = node_address {
            node_eth_addresses.extend(addresses.split(',').map(|addr| H160::from_str(addr).unwrap()));
        }
        // if local identity dirs/path is provided, read addresses from identity files
        if let Some(local_identity_arg) = local_identity {
            node_eth_addresses.extend(local_identity_arg.to_addresses().unwrap().into_iter().map(H160::from));
        }
        let node_addresses = if node_eth_addresses.is_empty() {
            None
        } else {
            Some(node_eth_addresses)
        };

        // get allowance
        let token_allowance = match allowance {
            Some(allw) => U256::from(parse_units(allw, "ether").unwrap()),
            None => U256::max_value(),
        };

        // read private key
        let signer_private_key = private_key.read()?;
        // get RPC provider for the given network and environment
        let rpc_provider = network_provider.get_provider_with_signer(&signer_private_key).await?;
        let contract_addresses = network_provider.get_network_details_from_name()?;

        // read all the admin addresses
        let admin_eth_addresses: Vec<H160> = match admin_address {
            Some(admin_address_str) => admin_address_str
                .split(',')
                .map(|addr| H160::from_str(addr).unwrap())
                .collect(),
            None => vec![signer_private_key.clone().public().to_address().into()],
        };

        // within one multicall, as an owner of the safe
        // deploy a safe proxy instance and a module proxy instance with multicall as an owner
        // add announcement as a permitted target in the deployed module proxy
        // approve token transfer to be done for the safe by channel contracts
        // if node addresses are known, include nodes to the module by safe
        // transfer safe ownership to actual admins
        // set desired threshold
        let hopr_stake_factory =
            HoprNodeStakeFactory::new(contract_addresses.addresses.node_stake_v2_factory, rpc_provider.clone());

        let (safe, node_module) = deploy_safe_module_with_targets_and_nodes(
            hopr_stake_factory,
            contract_addresses.addresses.token.into(),
            contract_addresses.addresses.channels.into(),
            contract_addresses.addresses.module_implementation.into(),
            contract_addresses.addresses.announcements.into(),
            token_allowance,
            node_addresses,
            admin_eth_addresses,
            U256::from(threshold),
        )
        .await
        .unwrap();

        println!("safe {:?}", safe.address());
        println!("node_module {:?}", node_module.address());

        // TODO: FIXME: action around network registry

        Ok(())
    }

    /// Execute the command, which moves nodes to a new managing safe and module pair
    /// Note that it does not register the node with the new safe on NodeSafeRegistry,
    /// because it is an action that nodes need to do on-start.
    pub async fn execute_safe_module_moving(
        network_provider: NetworkProviderArgs,
        local_identity: Option<IdentityFileArgs>,
        node_address: Option<String>,
        old_module_address: String,
        new_safe_address: String,
        new_module_address: String,
        private_key: PrivateKeyArgs,
    ) -> Result<(), HelperErrors> {
        // read all the node addresses
        let mut node_eth_addresses: Vec<H160> = Vec::new();
        if let Some(addresses) = node_address {
            node_eth_addresses.extend(addresses.split(',').map(|addr| H160::from_str(addr).unwrap()));
        }
        // if local identity dirs/path is provided, read addresses from identity files
        if let Some(local_identity_arg) = local_identity {
            node_eth_addresses.extend(local_identity_arg.to_addresses().unwrap().into_iter().map(H160::from));
        }

        // parse safe and module addresses
        let safe_addr = H160::from_str(&new_safe_address).unwrap();
        let module_addr = H160::from_str(&new_module_address).unwrap();
        let old_module_addr: Vec<H160> = old_module_address
            .split(',')
            .map(|addr| H160::from_str(addr).unwrap())
            .collect();

        // read private key
        let signer_private_key = private_key.read()?;
        // get RPC provider for the given network and environment
        let rpc_provider = network_provider.get_provider_with_signer(&signer_private_key).await?;
        let contract_addresses = network_provider.get_network_details_from_name()?;

        // 1. Deregister the old node-safe from node-safe registry
        // 2. Remove nodes from the old module
        // 3. Include node to the new module
        // 4. Remove node from network registry
        // 5. Include node to network registry
        let hopr_node_safe_registry =
            HoprNodeSafeRegistry::new(contract_addresses.addresses.node_safe_registry, rpc_provider.clone());
        let safe = SafeSingleton::new(safe_addr, rpc_provider.clone());

        if !node_eth_addresses.is_empty() {
            // first deregister nodes from their old safe
            deregister_nodes_from_node_safe_registry_and_remove_from_module(
                hopr_node_safe_registry.clone(),
                node_eth_addresses.clone(),
                old_module_addr,
                signer_private_key.clone(),
            )
            .await
            .unwrap();

            info!("Nodes are deregistered from old modules");
            // then include nodes to module
            include_nodes_to_module(safe, node_eth_addresses, module_addr, signer_private_key)
                .await
                .unwrap();
            info!("Nodes are included to the new module");
        };

        // TODO: FIXME: action around network registry

        Ok(())
    }

    /// Execute the command, which migrates nodes to a new network
    /// Note that it does not register the node with the new safe on NodeSafeRegistry,
    /// because it is an action that nodes need to do on-start.
    pub async fn execute_safe_module_migration(
        network_provider: NetworkProviderArgs,
        local_identity: Option<IdentityFileArgs>,
        node_address: Option<String>,
        safe_address: String,
        module_address: String,
        allowance: Option<f64>,
        private_key: PrivateKeyArgs,
    ) -> Result<(), HelperErrors> {
        // read all the node addresses
        let mut node_eth_addresses: Vec<H160> = Vec::new();
        if let Some(addresses) = node_address {
            node_eth_addresses.extend(addresses.split(',').map(|addr| H160::from_str(addr).unwrap()));
        }
        // if local identity dirs/path is provided, read addresses from identity files
        if let Some(local_identity_arg) = local_identity {
            node_eth_addresses.extend(local_identity_arg.to_addresses().unwrap().into_iter().map(H160::from));
        }

        // get allowance
        let token_allowance = match allowance {
            Some(allw) => U256::from(parse_units(allw, "ether").unwrap()),
            None => U256::max_value(),
        };

        // parse safe and module addresses
        let safe_addr = H160::from_str(&safe_address).unwrap();
        let module_addr = H160::from_str(&module_address).unwrap();

        // read private key
        let signer_private_key = private_key.read()?;
        // get RPC provider for the given network and environment
        let rpc_provider = network_provider.get_provider_with_signer(&signer_private_key).await?;
        let contract_addresses = network_provider.get_network_details_from_name()?;

        let safe = SafeSingleton::new(safe_addr, rpc_provider.clone());

        // Create a Safe tx to Multisend contract,
        // 1. scope the Channel contract of the new network to the module as target and set default permissions.
        // 2. scope the Announcement contract as target to the module
        // 3. approve HOPR tokens of the Safe proxy to be transferred by the new Channels contract
        migrate_nodes(
            safe,
            module_addr,
            contract_addresses.addresses.channels.into(),
            contract_addresses.addresses.token.into(),
            contract_addresses.addresses.announcements.into(),
            token_allowance,
            signer_private_key,
        )
        .await
        .unwrap();
        info!("a new network has been included due to the migration");

        // TODO: FIXME: action around network registry
        // if !node_eth_addresses.is_empty() {}
        Ok(())
    }
}

impl Cmd for SafeModuleSubcommands {
    /// Run the execute_safe_module_creation function
    fn run(self) -> Result<(), HelperErrors> {
        // self.execute_safe_module_creation()
        Ok(())
    }

    async fn async_run(self) -> Result<(), HelperErrors> {
        match self {
            SafeModuleSubcommands::Create {
                network_provider,
                local_identity,
                node_address,
                admin_address,
                threshold,
                allowance,
                private_key,
            } => {
                SafeModuleSubcommands::execute_safe_module_creation(
                    network_provider,
                    local_identity,
                    node_address,
                    admin_address,
                    threshold,
                    allowance,
                    private_key,
                )
                .await
                .unwrap();
            }
            SafeModuleSubcommands::Move {
                network_provider,
                local_identity,
                node_address,
                old_module_address,
                new_safe_address,
                new_module_address,
                private_key,
            } => {
                SafeModuleSubcommands::execute_safe_module_moving(
                    network_provider,
                    local_identity,
                    node_address,
                    old_module_address,
                    new_safe_address,
                    new_module_address,
                    private_key,
                )
                .await
                .unwrap();
            }
            SafeModuleSubcommands::Migrate {
                network_provider,
                local_identity,
                node_address,
                safe_address,
                module_address,
                allowance,
                private_key,
            } => {
                SafeModuleSubcommands::execute_safe_module_migration(
                    network_provider,
                    local_identity,
                    node_address,
                    safe_address,
                    module_address,
                    allowance,
                    private_key,
                )
                .await
                .unwrap();
            }
        }
        Ok(())
    }
}
