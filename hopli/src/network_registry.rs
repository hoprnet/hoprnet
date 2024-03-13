//! This module contains arguments and functions to interact with the Network Registry contract for a previledged account.
//! To participate in the HOPR network, a node must be included in the network registry contract.
//! Nodes and the staking account (Safe) that manages them should be registered as a pair in the Network registry contrat.
//! Nodes and safes can be registered by either a manager or by the staking account itself.
//!
//! Note the currently only manager wallet can register node-safe pairs. Node runners cannot self-register their nodes.
//!
//! A manager (i.e. an account with `MANAGER_ROLE` role), can perform the following actions with `hopli network-registry`,
//! by specifying the subcommand:
//! A manager account can register nodes and safes with `manager-regsiter`
//! A manager account can deregister nodes with `manager-deregsiter`
//! A manager account can set eligibility of staking accounts with `manager-force-sync`
use crate::key_pair::PrivateKeyReader;
use crate::{
    environment_config::NetworkProviderArgs,
    key_pair::{IdentityFileArgs, PrivateKeyArgs},
    methods::{
        deregister_nodes_from_network_registry, force_sync_safes_on_network_registry,
        register_safes_and_nodes_on_network_registry,
    },
    utils::{Cmd, HelperErrors},
};
use bindings::hopr_network_registry::HoprNetworkRegistry;
use clap::Parser;
use ethers::types::H160;
use log::info;
use std::str::FromStr;

/// CLI arguments for `hopli network-registry`
#[derive(Clone, Debug, Parser)]
pub enum NetworkRegistrySubcommands {
    // Register nodes and safes with a manager account
    #[command(visible_alias = "mr")]
    ManagerRegister {
        /// Network name, contracts config file root, and customized provider, if available
        #[command(flatten)]
        network_provider: NetworkProviderArgs,

        /// node addresses
        #[clap(
            help = "Comma separated node Ethereum addresses",
            long,
            short = 'o',
            default_value = None
        )]
        node_address: Option<String>,

        /// Addresses of the safe proxy instances
        #[clap(
            help = "Comma separated Safe Ethereum addresses",
            long,
            short,
            default_value = None
        )]
        safe_address: Option<String>,

        /// Arguments to locate identity file(s) of HOPR node(s)
        #[command(flatten)]
        local_identity: IdentityFileArgs,

        /// Access to the private key of a manager of Network Registry contract
        #[command(flatten)]
        private_key: PrivateKeyArgs,
    },

    /// Remove nodes and safes with a manager account
    #[command(visible_alias = "md")]
    ManagerDeregister {
        /// Network name, contracts config file root, and customized provider, if available
        #[command(flatten)]
        network_provider: NetworkProviderArgs,

        /// node addresses
        #[clap(
            help = "Comma separated node Ethereum addresses",
            long,
            short = 'o',
            default_value = None
        )]
        node_address: Option<String>,

        /// Arguments to locate identity file(s) of HOPR node(s)
        #[command(flatten)]
        local_identity: IdentityFileArgs,

        /// Access to the private key of a manager of Network Registry contract
        #[command(flatten)]
        private_key: PrivateKeyArgs,
    },

    /// Force sync the eligibility of safe accounts
    #[command(visible_alias = "ms")]
    ManagerForceSync {
        /// Network name, contracts config file root, and customized provider, if available
        #[command(flatten)]
        network_provider: NetworkProviderArgs,

        /// Addresses of the safe proxy instances
        #[clap(
            help = "Comma separated Safe Ethereum addresses",
            long,
            short,
            default_value = None
        )]
        safe_address: Option<String>,

        /// Access to the private key of a manager of Network Registry contract
        #[command(flatten)]
        private_key: PrivateKeyArgs,

        /// Eligibility of safes when calling `hopli network-registry -a manager-force-sync`
        #[clap(
            help = "Desired eligibility of safes",
            long,
            short,
            default_value = None
        )]
        eligibility: Option<bool>,
    },
}

impl NetworkRegistrySubcommands {
    /// Execute command to register a node and its staking account (safe) with manager privilege and make the safe eligible.
    ///
    /// Manager wallet registers nodes with associated staking accounts
    pub async fn execute_manager_register(
        network_provider: NetworkProviderArgs,
        local_identity: IdentityFileArgs,
        node_address: Option<String>,
        safe_address: Option<String>,
        private_key: PrivateKeyArgs,
    ) -> Result<(), HelperErrors> {
        // read all the node addresses
        let mut node_eth_addresses: Vec<H160> = Vec::new();
        if let Some(addresses) = node_address {
            node_eth_addresses.extend(addresses.split(',').map(|addr| H160::from_str(addr).unwrap()));
        }
        // if local identity dirs/path is provided, read addresses from identity files
        node_eth_addresses.extend(local_identity.to_addresses().unwrap().into_iter().map(H160::from));

        // read all the safe addresses
        let mut safe_eth_addresses: Vec<H160> = Vec::new();
        if let Some(addresses) = safe_address {
            safe_eth_addresses.extend(addresses.split(',').map(|addr| H160::from_str(addr).unwrap()));
        }

        // read private key. The provided env
        let signer_private_key = private_key.read("MANAGER_PRIVATE_KEY")?;

        // get RPC provider for the given network and environment
        let rpc_provider = network_provider.get_provider_with_signer(&signer_private_key).await?;
        let contract_addresses = network_provider.get_network_details_from_name()?;

        let hopr_network_registry =
            HoprNetworkRegistry::new(contract_addresses.addresses.network_registry, rpc_provider.clone());

        // get registered safe of all the nodes
        // check if any of the node has been registered to a different address than the given safe address.
        // if the node has been registered to the given safe address, skip any action on it
        // if the node has not been registered to any safe address, register it.
        // if the node has been registered to a different safe address, remove the old safe and register the new one
        let (removed_pairs_num, added_pairs_num) =
            register_safes_and_nodes_on_network_registry(hopr_network_registry, safe_eth_addresses, node_eth_addresses)
                .await?;
        info!(
            "{:?} pairs have been removed and {:?} pairs have been added to the network registry.",
            removed_pairs_num, added_pairs_num
        );
        Ok(())
    }

    /// Execute command to deregister a node and its staking account with manager privilege
    ///
    /// This action does not need to provide safe_address
    /// Manager wallet deregisters nodes from associated staking accounts
    pub async fn execute_manager_deregister(
        network_provider: NetworkProviderArgs,
        local_identity: IdentityFileArgs,
        node_address: Option<String>,
        private_key: PrivateKeyArgs,
    ) -> Result<(), HelperErrors> {
        // read all the node addresses
        let mut node_eth_addresses: Vec<H160> = Vec::new();
        if let Some(addresses) = node_address {
            node_eth_addresses.extend(addresses.split(',').map(|addr| H160::from_str(addr).unwrap()));
        }
        // if local identity dirs/path is provided, read addresses from identity files
        node_eth_addresses.extend(local_identity.to_addresses().unwrap().into_iter().map(H160::from));
        info!(
            "Will deregister {:?} nodes from the network registry",
            node_eth_addresses.len()
        );

        // read private key
        let signer_private_key = private_key.read("MANAGER_PRIVATE_KEY")?;

        // get RPC provider for the given network and environment
        let rpc_provider = network_provider.get_provider_with_signer(&signer_private_key).await?;
        let contract_addresses = network_provider.get_network_details_from_name()?;

        let hopr_network_registry =
            HoprNetworkRegistry::new(contract_addresses.addresses.network_registry, rpc_provider.clone());

        // deregister all the given nodes from the network registry
        let removed_pairs_num =
            deregister_nodes_from_network_registry(hopr_network_registry, node_eth_addresses).await?;
        info!(
            "{:?} pairs have been removed from the network registry.",
            removed_pairs_num
        );
        Ok(())
    }

    /// Execute command to force sync eligibility of staking accounts with manager privilege
    ///
    /// This action does not need to provide node_address
    /// Manager wallet sync eligibility of staking accounts to a given value
    pub async fn execute_manager_force_sync(
        network_provider: NetworkProviderArgs,
        safe_address: Option<String>,
        private_key: PrivateKeyArgs,
        eligibility: Option<bool>,
    ) -> Result<(), HelperErrors> {
        // read all the safe addresses
        let mut safe_eth_addresses: Vec<H160> = Vec::new();
        if let Some(addresses) = safe_address {
            safe_eth_addresses.extend(addresses.split(',').map(|addr| H160::from_str(addr).unwrap()));
        }

        info!(
            "Will force sync {:?} safes in the network registry",
            safe_eth_addresses.len()
        );

        // read private key
        let signer_private_key = private_key.read("MANAGER_PRIVATE_KEY")?;

        // get RPC provider for the given network and environment
        let rpc_provider = network_provider.get_provider_with_signer(&signer_private_key).await?;
        let contract_addresses = network_provider.get_network_details_from_name()?;

        let hopr_network_registry =
            HoprNetworkRegistry::new(contract_addresses.addresses.network_registry, rpc_provider.clone());

        // deregister all the given nodes from the network registry
        match eligibility {
            Some(safe_eligibility) => {
                force_sync_safes_on_network_registry(
                    hopr_network_registry,
                    safe_eth_addresses.clone(),
                    vec![safe_eligibility; safe_eth_addresses.len()],
                )
                .await?;
                info!(
                    "synced the eligibility of {:?} safes in the network registry to {:?}",
                    safe_eth_addresses.len(),
                    safe_eligibility
                );
                Ok(())
            }
            None => Err(HelperErrors::MissingParameter("eligibility".to_string())),
        }
    }
}

impl Cmd for NetworkRegistrySubcommands {
    /// Run the execute_register function.
    /// By default, registration is done by manager wallet
    fn run(self) -> Result<(), HelperErrors> {
        Ok(())
    }

    async fn async_run(self) -> Result<(), HelperErrors> {
        match self {
            NetworkRegistrySubcommands::ManagerRegister {
                network_provider,
                local_identity,
                node_address,
                safe_address,
                private_key,
            } => {
                NetworkRegistrySubcommands::execute_manager_register(
                    network_provider,
                    local_identity,
                    node_address,
                    safe_address,
                    private_key,
                )
                .await?;
            }
            NetworkRegistrySubcommands::ManagerDeregister {
                network_provider,
                local_identity,
                node_address,
                private_key,
            } => {
                NetworkRegistrySubcommands::execute_manager_deregister(
                    network_provider,
                    local_identity,
                    node_address,
                    private_key,
                )
                .await?;
            }
            NetworkRegistrySubcommands::ManagerForceSync {
                network_provider,
                safe_address,
                private_key,
                eligibility,
            } => {
                NetworkRegistrySubcommands::execute_manager_force_sync(
                    network_provider,
                    safe_address,
                    private_key,
                    eligibility,
                )
                .await?;
            }
        }
        Ok(())
    }
}
