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
use crate::{
    environment_config::NetworkProviderArgs,
    identity::{IdentityFileArgs, PrivateKeyArgs},
    methods::deploy_safe_module_with_targets_and_nodes,
    utils::{Cmd, HelperErrors},
};
use bindings::hopr_node_stake_factory::HoprNodeStakeFactory;
use clap::{builder::RangedU64ValueParser, Parser};
use ethers::{
    types::{H160, U256},
    utils::parse_units,
};
use hopr_crypto_types::keypairs::Keypair;
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
        // /// The name of the contract to upload selectors for.
        // #[arg(required_unless_present = "all")]
        // contract: Option<String>,

        // /// Upload selectors for all contracts in the project.
        // #[arg(long, required_unless_present = "contract")]
        // all: bool,

        // #[command(flatten)]
        // project_paths: ProjectPathsArgs,
    },

    /// Move nodes to one single safe and module pair
    #[command(visible_alias = "mv")]
    Move {
        // /// The name of the contract to list selectors for.
        // #[arg(help = "The name of the contract to list selectors for.")]
        // contract: Option<String>,

        // #[command(flatten)]
        // project_paths: ProjectPathsArgs,
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
            SafeModuleSubcommands::Move {} => {}
            SafeModuleSubcommands::Migrate {} => {}
        }
        Ok(())
    }
}
