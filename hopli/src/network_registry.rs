//! This module contains arguments and functions to register node-staking safe pairs to the Network Registry contract.
//!
//! Note the currently only manager wallet can register node-safe pairs
use crate::{
    identity::{IdentityFileArgs, PrivateKeyArgs},
    process::{
        child_process_call_foundry_manager_register, child_process_call_foundry_self_register, set_process_path_env,
    },
    utils::{Cmd, HelperErrors},
};
use clap::Parser;
use log::{log, Level};

/// CLI arguments for `hopli register-in-network-registry`
#[derive(Parser, Default, Debug)]
pub struct RegisterInNetworkRegistryArgs {
    /// Name of the network that the node is running on
    #[clap(help = "Network name. E.g. monte_rosa", long)]
    network: String,

    /// node addresses
    #[clap(
        help = "Comma separated node Ethereum addresses",
        long,
        short,
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
    #[clap(flatten)]
    local_identity: IdentityFileArgs,

    /// Path to the root of foundry project (etehereum/contracts), where all the contracts and `contracts-addresses.json` are stored
    #[clap(
        help = "Specify path pointing to the contracts root",
        long,
        short,
        default_value = None
    )]
    contracts_root: Option<String>,

    /// Access to the private key of a manager of Network Registry contract
    #[clap(flatten)]
    pub private_key: PrivateKeyArgs,
}

impl RegisterInNetworkRegistryArgs {
    /// Execute command to register a node and its stakign account without manager privilege
    ///
    /// Node self register with given parameters
    /// `PRIVATE_KEY` env variable is required to send on-chain transactions
    pub fn execute_self_register(self) -> Result<(), HelperErrors> {
        let RegisterInNetworkRegistryArgs {
            network,
            local_identity,
            node_address,
            safe_address: _,
            contracts_root,
            private_key,
        } = self;

        // `PRIVATE_KEY` - Private key is required to send on-chain transactions
        private_key.read()?;

        // collect all the peer ids
        let mut all_chain_addrs = Vec::new();
        // add node_address from CLI, if there's one
        if let Some(provided_chain_addrs) = node_address {
            all_chain_addrs.push(provided_chain_addrs);
        }

        // // read all the identities from the directory
        all_chain_addrs.extend(
            local_identity
                .to_addresses()
                .unwrap()
                .into_iter()
                .map(|adr| adr.to_string()),
        );

        log!(target: "network_registry", Level::Info, "merged node_address {:?}", all_chain_addrs.join(","));

        // set directory and environment variables
        set_process_path_env(&contracts_root, &network)?;

        // iterate and collect execution result. If error occurs, the entire operation failes.
        child_process_call_foundry_self_register(&network, &all_chain_addrs.join(","))
    }

    /// Execute command to register a node and its stakign account with manager privilege
    ///
    /// Manager wallet registers nodes with associated staking accounts
    /// `PRIVATE_KEY` env variable is required to send on-chain transactions
    pub fn execute_manager_register(self) -> Result<(), HelperErrors> {
        let RegisterInNetworkRegistryArgs {
            network,
            local_identity,
            node_address,
            safe_address,
            contracts_root,
            private_key,
        } = self;

        // `PRIVATE_KEY` - Private key is required to send on-chain transactions
        private_key.read()?;

        // collect all the peer ids
        let mut all_chain_addrs = Vec::new();
        // add node_address from CLI, if there's one
        if let Some(provided_chain_addrs) = node_address {
            all_chain_addrs.push(provided_chain_addrs);
        }

        // // read all the identities from the directory
        all_chain_addrs.extend(
            local_identity
                .to_addresses()
                .unwrap()
                .into_iter()
                .map(|adr| adr.to_string()),
        );

        log!(target: "network_registry", Level::Info, "merged node_address {:?}", all_chain_addrs.join(","));

        // set directory and environment variables
        set_process_path_env(&contracts_root, &network)?;

        // iterate and collect execution result. If error occurs, the entire operation failes.
        child_process_call_foundry_manager_register(&network, &safe_address.unwrap(), &all_chain_addrs.join(","))
    }
}

impl Cmd for RegisterInNetworkRegistryArgs {
    /// Run the execute_register function.
    /// By default, registration is done by manager wallet
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_manager_register()
    }

    async fn async_run(self) -> Result<(), HelperErrors> {
        Ok(())
    }
}
