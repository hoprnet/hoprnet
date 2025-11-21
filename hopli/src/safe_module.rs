//! This module contains arguments and functions to manage safe and module.
//! [SafeModuleSubcommands] defines three subcommands: create, move, and migrate.
//! - [SafeModuleSubcommands::Create] creates staking wallets (safe and node management module) and execute necessary
//!   on-chain transactions to setup a HOPR node. Detailed breakdown of the steps:
//!     - create a Safe proxy instance and HOPR node management module proxy instance
//!     - include nodes configure default permissions on the created module proxy
//!     - fund the node and Safe with some native tokens and HOPR tokens respectively
//!     - approve HOPR tokens to be transferred from the Safe proxy instaces by Channels contract
//!     - Use manager wallet to add nodes and staking safes to the Network Registry contract
//! - [SafeModuleSubcommands::Move] moves a node from to an existing Safe. Note that the Safe should has a node
//!   management module attached and configured. Note that the admin key of the old and new safes are the same. This
//!   command does not support moving nodes to safes controled by a different admin key. Note that all the safes
//!   involved (old and new) should have a threshold of 1 Detailed breakdown of the steps:
//!     - use old safes to deregister nodes from Node-safe registry
//!     - use the new safe to include nodes to the module
//!     - use manager wallet to deregister nodes from the network registry
//!     - use manager wallet to register nodes with new safes to the network regsitry
//! - [SafeModuleSubcommands::Migrate] migrates a node to a different network. It performs the following steps:
//!     - add the Channel contract of the new network to the module as target and set default permissions.
//!     - add the Announcement contract as target to the module
//!     - approve HOPR tokens of the Safe proxy to be transferred by the new Channels contract
//!     - Use the manager wallet to add nodes and Safes to the Network Registry contract of the new network.
//! - [SafeModuleSubcommands::Debug] goes through a series of checks to debug the setup of a node and safe.
//!
//! It checks the following:
//!     - node xDAI balance
//!     - If node has been included on Network Registry
//!     - If node and safe are associated on Node Safe Registry
//!     - If Safe is owned by the correct owner(s)
//!     - Safe’s wxHOPR balance and allowance
//!     - if the module is enabled
//!     - if node is included in the module
//!     - Get all the targets of the safe (then check if channel and announcement are there)
//!     - Get the owner of the module
//!
//! You need to enable the INFO level of the tracing logger to see the output of the debug command.
//!
//! Some sample commands
//! - Express creation of a safe and a module
//! ```text
//! hopli safe-module create \
//!     --network anvil-localhost \
//!     --contracts-root "../ethereum/contracts" \
//!     --identity-directory "./test" \
//!     --password-path "./test/pwd" \
//!     --admin-address 0x47f2710069F01672D01095cA252018eBf08bF85e,0x0D07Eb66Deb54D48D004765E13DcC028cf56592b \
//!     --allowance 10.5 \
//!     --hopr-amount 10 \
//!     --native-amount 0.1 \
//!     --manager-private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
//!     --private-key 59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d \
//!     --provider-url "http://localhost:8545"
//! ```
//!
//! - Migrate nodes and safe to a new network
//! ```text
//! hopli safe-module migrate \
//!     --network anvil-localhost2 \
//!     --contracts-root "../ethereum/contracts" \
//!     --identity-directory "./test" \
//!     --password-path "./test/pwd" \
//!     --safe-address 0x6a64fe01c3aba5bdcd04b81fef375369ca47326f \
//!     --module-address 0x5d46d0c5279fd85ce7365e4d668f415685922839 \
//!     --manager-private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
//!     --private-key 59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d \
//!     --provider-url "http://localhost:8545"
//! ```
//!
//! - Move registered nodes to a different set of safe and module
//! ```text
//! hopli safe-module move \
//!     --network anvil-localhost \
//!     --contracts-root "../ethereum/contracts"  \
//!     --old-module-address 0x5d46d0c5279fd85ce7365e4d668f415685922839 \
//!     --new-safe-address 0xce66d19a86600f3c6eb61edd6c431ded5cc92b21 \
//!     --new-module-address 0x3086c20265cf742b169b05cd0eae1941455e4e9f \
//!     --node-address 0x93a50B0fFF7b4ED36A3C6445e280E72AC2AEFc51,0x58033D3074D001a32bF379801eaf8969817fFfCf,0xeEDaab91158928647a9270Fe290897eBB1230250 \
//!     --manager-private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
//!     --private-key 59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d \
//!     --provider-url "http://localhost:8545"
//!
//! - Debug nodes and safe
//! ```text
//! hopli safe-module debug \
//!     --network anvil-localhost2 \
//!     --contracts-root "../ethereum/contracts" \
//!     --identity-directory "./test" \
//!     --password-path "./test/pwd" \
//!     --safe-address 0x6a64fe01c3aba5bdcd04b81fef375369ca47326f \
//!     --module-address 0x5d46d0c5279fd85ce7365e4d668f415685922839 \
//!     --provider-url "http://localhost:8545"
//! ```
use std::str::FromStr;

use clap::{Parser, builder::RangedU64ValueParser};
use hopr_bindings::{
    exports::alloy::primitives::{Address, U256, utils::parse_units},
    hopr_node_safe_registry::HoprNodeSafeRegistry,
    hopr_node_stake_factory::HoprNodeStakeFactory,
    hopr_token::HoprToken,
};
use hopr_crypto_types::keypairs::Keypair;
use tracing::{info, warn};

use crate::{
    environment_config::NetworkProviderArgs,
    key_pair::{ArgEnvReader, IdentityFileArgs, ManagerPrivateKeyArgs, PrivateKeyArgs},
    methods::{
        SafeSingleton, debug_node_safe_module_setup_main, debug_node_safe_module_setup_on_balance_and_registries,
        deploy_safe_module_with_targets_and_nodes, deregister_nodes_from_node_safe_registry_and_remove_from_module,
        include_nodes_to_module, migrate_nodes, transfer_native_tokens, transfer_or_mint_tokens,
    },
    utils::{Cmd, HelperErrors, a2h},
};

/// CLI arguments for `hopli safe-module`
#[derive(Clone, Debug, Parser)]
pub enum SafeModuleSubcommands {
    /// Create safe and module proxy if nothing exists
    #[command(visible_alias = "cr")]
    Create {
        /// Network name, contracts config file root, and customized provider, if available
        #[command(flatten)]
        network_provider: NetworkProviderArgs,

        /// Arguments to locate identity file(s) of HOPR node(s)
        #[command(flatten)]
        local_identity: IdentityFileArgs,

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

        /// The amount of HOPR tokens (in floating number) to be funded to the new safe
        #[clap(
            help = "Hopr amount in ether, e.g. 10",
            long,
            short = 'm',
            value_parser = clap::value_parser!(f64),
        )]
        hopr_amount: Option<f64>,

        /// The amount of native tokens (in floating number) to be funded per node
        #[clap(
            help = "Native token amount in ether, e.g. 1",
            long,
            short = 'g',
            value_parser = clap::value_parser!(f64),
        )]
        native_amount: Option<f64>,

        /// Access to the private key, of which the wallet either contains sufficient assets
        /// as the source of funds or it can mint necessary tokens
        #[command(flatten)]
        private_key: PrivateKeyArgs,

        /// Access to the private key, of which the wallet has `MANAGER_ROLE` of network registry
        /// If provided, this wallet will grant the created safe access to the network registry
        #[command(flatten, name = "manager_private_key")]
        manager_private_key: ManagerPrivateKeyArgs,
    },

    /// Migrate safe and module to a new network
    #[command(visible_alias = "mg")]
    Migrate {
        /// Network name, contracts config file root, and customized provider, if available
        #[command(flatten)]
        network_provider: NetworkProviderArgs,

        /// Arguments to locate identity file(s) of HOPR node(s)
        #[command(flatten)]
        local_identity: IdentityFileArgs,

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
        #[command(flatten)]
        private_key: PrivateKeyArgs,

        /// Access to the private key, of which the wallet has `MANAGER_ROLE` of network registry
        /// If provided, this wallet will grant the created safe access to the network registry
        #[command(flatten, name = "manager_private_key")]
        manager_private_key: ManagerPrivateKeyArgs,
    },

    /// Move nodes to one single safe and module pair
    #[command(visible_alias = "mv")]
    Move {
        /// Network name, contracts config file root, and customized provider, if available
        #[command(flatten)]
        network_provider: NetworkProviderArgs,

        /// Arguments to locate identity file(s) of HOPR node(s)
        #[command(flatten)]
        local_identity: IdentityFileArgs,

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
        #[command(flatten)]
        private_key: PrivateKeyArgs,

        /// Access to the private key, of which the wallet has `MANAGER_ROLE` of network registry
        /// If provided, this wallet will grant the created safe access to the network registry
        #[command(flatten, name = "manager_private_key")]
        manager_private_key: ManagerPrivateKeyArgs,
    },

    /// Debug safe and module setup
    #[command(visible_alias = "dg")]
    Debug {
        /// Network name, contracts config file root, and customized provider, if available
        #[command(flatten)]
        network_provider: NetworkProviderArgs,

        /// Arguments to locate identity file(s) of HOPR node(s)
        #[command(flatten)]
        local_identity: IdentityFileArgs,

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
    },
}

impl SafeModuleSubcommands {
    /// Execute the command, which quickly create necessary staking wallets
    /// and execute necessary on-chain transactions to setup a HOPR node.
    ///
    /// 1. Create a safe instance and a node management module instance:
    /// 2. Set default permissions for the module
    /// 3. Include node as a member with restricted permission on sending assets
    /// 4. transfer some HOPR token to the new safe (directly)
    /// 5. transfer some native tokens to nodes
    #[allow(clippy::too_many_arguments)]
    pub async fn execute_safe_module_creation(
        network_provider: NetworkProviderArgs,
        local_identity: IdentityFileArgs,
        node_address: Option<String>,
        admin_address: Option<String>,
        threshold: u32,
        hopr_amount: Option<f64>,
        native_amount: Option<f64>,
        private_key: PrivateKeyArgs,
    ) -> Result<(), HelperErrors> {
        // read all the node addresses
        let mut node_eth_addresses: Vec<Address> = Vec::new();
        if let Some(addresses) = node_address {
            node_eth_addresses.extend(
                addresses
                    .split(',')
                    .map(|addr| {
                        Address::from_str(addr)
                            .map_err(|e| HelperErrors::InvalidAddress(format!("Invalid node address: {e:?}")))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        // if local identity dirs/path is provided, read addresses from identity files
        node_eth_addresses.extend(
            local_identity
                .to_addresses()
                .map_err(|e| HelperErrors::InvalidAddress(format!("Invalid node address: {e:?}")))?
                .into_iter()
                .map(a2h),
        );

        let node_addresses = if node_eth_addresses.is_empty() {
            None
        } else {
            Some(node_eth_addresses.clone())
        };

        // read private key
        let signer_private_key = private_key.read_default()?;
        // get RPC provider for the given network and environment
        let rpc_provider = network_provider.get_provider_with_signer(&signer_private_key).await?;
        let contract_addresses = network_provider.get_network_details_from_name()?;

        // read all the admin addresses
        let admin_eth_addresses: Vec<Address> = match admin_address {
            Some(admin_address_str) => admin_address_str
                .split(',')
                .map(|addr| Address::from_str(addr).unwrap())
                .collect(),
            None => vec![a2h(signer_private_key.clone().public().to_address())],
        };

        // within one multicall, as an owner of the safe
        // deploy a safe proxy instance and a module proxy instance with multicall as an owner
        // add announcement as a permitted target in the deployed module proxy
        // approve token transfer to be done for the safe by channel contracts
        // if node addresses are known, include nodes to the module by safe
        // transfer safe ownership to actual admins
        // set desired threshold
        let hopr_stake_factory = HoprNodeStakeFactory::new(
            a2h(contract_addresses.addresses.node_stake_factory),
            rpc_provider.clone(),
        );

        let (safe, node_module) = deploy_safe_module_with_targets_and_nodes(
            hopr_stake_factory,
            a2h(contract_addresses.addresses.channels),
            node_addresses,
            admin_eth_addresses,
            U256::from(threshold),
        )
        .await?;

        println!("safe {:?}", safe.address());
        println!("node_module {:?}", node_module.address());

        // direct transfer of some HOPR tokens to the safe
        if let Some(hopr_amount_for_safe) = hopr_amount {
            let hopr_token = HoprToken::new(a2h(contract_addresses.addresses.token), rpc_provider.clone());
            let hopr_to_be_transferred: U256 = parse_units(&hopr_amount_for_safe.to_string(), "ether")
                .map_err(|_| HelperErrors::ParseError("Failed to parse HOPR amount units".into()))?
                .into();

            transfer_or_mint_tokens(hopr_token, vec![*safe.address()], vec![hopr_to_be_transferred]).await?;
            info!(
                "safe {:?} has received {:?} HOPR tokens",
                safe.address(),
                hopr_amount_for_safe
            );
        }

        // distribute some native tokens to the nodes
        if let Some(native_amount_for_node) = native_amount {
            let native_to_be_transferred: U256 = parse_units(&native_amount_for_node.to_string(), "ether")
                .map_err(|_| HelperErrors::ParseError("Failed to parse HOPR amount units".into()))?
                .into();
            let native_amounts = vec![native_to_be_transferred; node_eth_addresses.len()];
            transfer_native_tokens(rpc_provider.clone(), node_eth_addresses.clone(), native_amounts).await?;
            info!(
                "each node in {:?} has received {:?} native tokens",
                node_eth_addresses, native_amount_for_node
            );
        }

        Ok(())
    }

    /// Execute the command, which moves nodes to a new managing safe and module pair
    /// Note that it does not register the node with the new safe on NodeSafeRegistry,
    /// because it is an action that nodes need to do on-start.
    #[allow(clippy::too_many_arguments)]
    pub async fn execute_safe_module_moving(
        network_provider: NetworkProviderArgs,
        local_identity: IdentityFileArgs,
        node_address: Option<String>,
        old_module_address: String,
        new_safe_address: String,
        new_module_address: String,
        private_key: PrivateKeyArgs,
    ) -> Result<(), HelperErrors> {
        // read all the node addresses
        let mut node_eth_addresses: Vec<Address> = Vec::new();
        if let Some(addresses) = node_address {
            node_eth_addresses.extend(
                addresses
                    .split(',')
                    .map(|addr| {
                        Address::from_str(addr)
                            .map_err(|e| HelperErrors::InvalidAddress(format!("Invalid node address: {e:?}")))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        // if local identity dirs/path is provided, read addresses from identity files
        node_eth_addresses.extend(
            local_identity
                .to_addresses()
                .map_err(|e| HelperErrors::InvalidAddress(format!("Invalid node address: {e:?}")))?
                .into_iter()
                .map(a2h),
        );

        // parse safe and module addresses
        let safe_addr = Address::from_str(&new_safe_address)
            .map_err(|_| HelperErrors::InvalidAddress(format!("Cannot parse safe address {new_safe_address:?}")))?;
        let module_addr = Address::from_str(&new_module_address)
            .map_err(|_| HelperErrors::InvalidAddress(format!("Cannot parse module address {new_module_address:?}")))?;
        let old_module_addr: Vec<Address> = old_module_address
            .split(',')
            .map(|addr| Address::from_str(addr).unwrap())
            .collect();

        // read private key
        let signer_private_key = private_key.read_default()?;
        // get RPC provider for the given network and environment
        let rpc_provider = network_provider.get_provider_with_signer(&signer_private_key).await?;
        let contract_addresses = network_provider.get_network_details_from_name()?;

        // 1. Deregister the old node-safe from node-safe registry
        // 2. Remove nodes from the old module
        // 3. Include node to the new module
        // 4. Remove node from network registry
        // 5. Include node to network registry
        let hopr_node_safe_registry = HoprNodeSafeRegistry::new(
            a2h(contract_addresses.addresses.node_safe_registry),
            rpc_provider.clone(),
        );
        let safe = SafeSingleton::new(safe_addr, rpc_provider.clone());

        if !node_eth_addresses.is_empty() {
            // first deregister nodes from their old safe
            match deregister_nodes_from_node_safe_registry_and_remove_from_module(
                hopr_node_safe_registry.clone(),
                node_eth_addresses.clone(),
                old_module_addr,
                signer_private_key.clone(),
            )
            .await
            {
                Ok(_) => {
                    info!("Nodes are deregistered from old safes");
                }
                Err(e) => {
                    return Err(e);
                }
            };

            // then include nodes to module
            match include_nodes_to_module(
                safe.clone(),
                node_eth_addresses.clone(),
                module_addr,
                signer_private_key,
            )
            .await
            {
                Ok(_) => {
                    info!("Nodes are included to the new module");
                }
                Err(e) => {
                    return Err(e);
                }
            };
        };

        Ok(())
    }

    /// Execute the command, which migrates nodes to a new network
    /// Note that it does not register the node with the new safe on NodeSafeRegistry,
    /// because it is an action that nodes need to do on-start.
    #[allow(clippy::too_many_arguments)]
    pub async fn execute_safe_module_migration(
        network_provider: NetworkProviderArgs,
        local_identity: IdentityFileArgs,
        node_address: Option<String>,
        safe_address: String,
        module_address: String,
        allowance: Option<f64>,
        private_key: PrivateKeyArgs,
    ) -> Result<(), HelperErrors> {
        // read all the node addresses
        let mut node_eth_addresses: Vec<Address> = Vec::new();
        if let Some(addresses) = node_address {
            node_eth_addresses.extend(
                addresses
                    .split(',')
                    .map(|addr| {
                        Address::from_str(addr)
                            .map_err(|e| HelperErrors::InvalidAddress(format!("Invalid node address: {e:?}")))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        // if local identity dirs/path is provided, read addresses from identity files
        node_eth_addresses.extend(
            local_identity
                .to_addresses()
                .map_err(|e| HelperErrors::InvalidAddress(format!("Invalid node address: {e:?}")))?
                .into_iter()
                .map(a2h),
        );

        // get allowance
        let token_allowance: U256 = match allowance {
            Some(allw) => parse_units(&allw.to_string(), "ether")
                .map_err(|_| HelperErrors::ParseError("Failed to parse allowance units".into()))?
                .into(),
            None => U256::MAX,
        };

        // parse safe and module addresses
        let safe_addr = Address::from_str(&safe_address)
            .map_err(|_| HelperErrors::InvalidAddress(format!("Cannot parse safe address {safe_address:?}")))?;
        let module_addr = Address::from_str(&module_address)
            .map_err(|_| HelperErrors::InvalidAddress(format!("Cannot parse module address {module_address:?}")))?;

        // read private key
        let signer_private_key = private_key.read_default()?;
        // get RPC provider for the given network and environment
        let rpc_provider = network_provider.get_provider_with_signer(&signer_private_key).await?;
        let contract_addresses = network_provider.get_network_details_from_name()?;

        let safe = SafeSingleton::new(safe_addr, rpc_provider.clone());

        // Create a Safe tx to Multisend contract,
        // 1. scope the Channel contract of the new network to the module as target and set default permissions.
        // 2. scope the Announcement contract as target to the module
        // 3. approve HOPR tokens of the Safe proxy to be transferred by the new Channels contract
        migrate_nodes(
            safe.clone(),
            module_addr,
            a2h(contract_addresses.addresses.channels),
            a2h(contract_addresses.addresses.token),
            a2h(contract_addresses.addresses.announcements),
            token_allowance,
            signer_private_key,
        )
        .await?;
        info!("a new network has been included due to the migration");

        Ok(())
    }

    /// Execute the command to debug the following:
    /// 1. node xDAI balance
    /// 2. If node has been included on Network Registry
    /// 3. If node and safe are associated on Node Safe Registry
    /// 4. If Safe is owned by the correct owner(s)
    /// 5. Safe’s wxHOPR balance and allowance
    /// 6. if node is included in the module
    /// 7. If the channel contract is included as a target
    /// 8. If the announce contract is included as a target
    /// 9. If safe is the owner of the module
    #[allow(clippy::too_many_arguments)]
    pub async fn execute_safe_module_debugging(
        network_provider: NetworkProviderArgs,
        local_identity: IdentityFileArgs,
        node_address: Option<String>,
        safe_address: String,
        module_address: String,
    ) -> Result<(), HelperErrors> {
        // read all the node addresses
        info!("Reading all the node addresses...");
        let mut node_eth_addresses: Vec<Address> = Vec::new();
        if let Some(addresses) = node_address {
            node_eth_addresses.extend(
                addresses
                    .split(',')
                    .map(|addr| {
                        Address::from_str(addr)
                            .map_err(|e| HelperErrors::InvalidAddress(format!("Invalid node address: {e:?}")))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        // if local identity dirs/path is provided, read addresses from identity files
        node_eth_addresses.extend(
            local_identity
                .to_addresses()
                .map_err(|e| HelperErrors::InvalidAddress(format!("Invalid node address: {e:?}")))?
                .into_iter()
                .map(a2h),
        );

        // parse safe and module addresses
        let safe_addr = Address::from_str(&safe_address)
            .map_err(|_| HelperErrors::InvalidAddress(format!("Cannot parse safe address {safe_address:?}")))?;
        let module_addr = Address::from_str(&module_address)
            .map_err(|_| HelperErrors::InvalidAddress(format!("Cannot parse module address {module_address:?}")))?;

        // get RPC provider for the given network and environment
        let rpc_provider = network_provider.get_provider_without_signer().await?;
        let contract_addresses = network_provider.get_network_details_from_name()?;

        let hopr_token = HoprToken::new(a2h(contract_addresses.addresses.token), rpc_provider.clone());
        let node_safe_registry = HoprNodeSafeRegistry::new(
            a2h(contract_addresses.addresses.node_safe_registry),
            rpc_provider.clone(),
        );

        // loop through all the nodes and debug
        for node in node_eth_addresses {
            info!("Starting debug checks for node: {:?}", node);
            info!("Checking node registration with safe registry...");
            let registered_safe =
                debug_node_safe_module_setup_on_balance_and_registries(node_safe_registry.clone(), &node)
                    .await
                    .map_err(HelperErrors::MulticallError)?;

            // compare the registered safe with the provided safe
            if registered_safe != safe_addr {
                warn!(
                    "Node {:?} is not registered with the provided safe {:?}",
                    node, safe_addr
                );
            }
            info!("Checking node and safe association in node-safe registry...");
            debug_node_safe_module_setup_main(
                hopr_token.clone(),
                &module_addr,
                &node,
                &safe_addr,
                &a2h(contract_addresses.addresses.channels),
                &a2h(contract_addresses.addresses.announcements),
            )
            .await
            .map_err(HelperErrors::MulticallError)?;
        }
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
                #[allow(unused_variables)]
                allowance,
                hopr_amount,
                native_amount,
                private_key,
                ..
            } => {
                SafeModuleSubcommands::execute_safe_module_creation(
                    network_provider,
                    local_identity,
                    node_address,
                    admin_address,
                    threshold,
                    hopr_amount,
                    native_amount,
                    private_key,
                )
                .await
            }
            SafeModuleSubcommands::Move {
                network_provider,
                local_identity,
                node_address,
                old_module_address,
                new_safe_address,
                new_module_address,
                private_key,
                ..
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
            }
            SafeModuleSubcommands::Migrate {
                network_provider,
                local_identity,
                node_address,
                safe_address,
                module_address,
                allowance,
                private_key,
                ..
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
            }
            SafeModuleSubcommands::Debug {
                network_provider,
                local_identity,
                node_address,
                safe_address,
                module_address,
            } => {
                SafeModuleSubcommands::execute_safe_module_debugging(
                    network_provider,
                    local_identity,
                    node_address,
                    safe_address,
                    module_address,
                )
                .await
            }
        }
    }
}
