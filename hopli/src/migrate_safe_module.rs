//! This module contains arguments and functions to migrate existing HOPR node(s), Safe,
//! and module to be migrated to a different network.
//!
//! It performs the following steps:
//! - add the Channel contract of the new network to the module as target and set default permissions.
//! - add the Announcement contract as target to the module
//! - approve HOPR tokens of the Safe proxy to be transferred by the new Channels contract
//! - Use the manager wlalet to add nodes and Safes to the Network Registry contract of the new network.
use crate::{
    identity::{IdentityFileArgs, PrivateKeyArgs},
    process::{child_process_call_foundry_migrate_safe_module, set_process_path_env},
    utils::{Cmd, HelperErrors},
};
use clap::Parser;
use hopr_crypto_types::types::ToChecksum;
use hopr_primitive_types::primitives::Address;
use log::{log, Level};
use std::str::FromStr;

/// CLI arguments for `hopli migrate-safe-module`
#[derive(Parser, Default, Debug)]
pub struct MigrateSafeModuleArgs {
    /// Name of the network that the node is running on
    #[clap(help = "Network name. E.g. monte_rosa", long)]
    network: String,

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

    /// Address of the safe proxy instance
    #[clap(help = "Ethereum address of safe", long, short)]
    safe_address: String,

    /// Address of the node management module proxy instance
    #[clap(help = "Ethereum address of node management module", long, short)]
    module_address: String,

    /// Access to the private key, of which the wallet is the manager of Network Registry contract
    #[clap(flatten)]
    pub private_key: PrivateKeyArgs,
}

impl MigrateSafeModuleArgs {
    /// Execute the command to migrate node, safe, and module to the network.
    fn execute_safe_module_migration(self) -> Result<(), HelperErrors> {
        let MigrateSafeModuleArgs {
            network,
            local_identity,
            contracts_root,
            safe_address,
            module_address,
            private_key,
        } = self;

        // 1. `PRIVATE_KEY` - Private key is required to send on-chain transactions
        private_key.read()?;

        // 2. Calculate addresses from the identity file
        let all_node_addresses: Vec<String> = local_identity
            .to_addresses()
            .unwrap()
            .into_iter()
            .map(|adr| adr.to_string())
            .collect();

        log!(target: "migrate_safe_module", Level::Info, "NodeAddresses {:?}", all_node_addresses.join(","));

        // 3. parse safe and module address
        let parsed_safe_addr = if safe_address.starts_with("0x") {
            safe_address.strip_prefix("0x").unwrap_or(&safe_address)
        } else {
            &safe_address
        };
        let parsed_module_addr = if module_address.starts_with("0x") {
            module_address.strip_prefix("0x").unwrap_or(&module_address)
        } else {
            &module_address
        };
        let checksumed_safe_addr = Address::from_str(parsed_safe_addr).unwrap().to_checksum();
        let checksumed_module_addr = Address::from_str(parsed_module_addr).unwrap().to_checksum();

        // set directory and environment variables
        set_process_path_env(&contracts_root, &network)?;

        log!(target: "migrate_safe_module", Level::Debug, "Calling foundry...");
        // iterate and collect execution result. If error occurs, the entire operation failes.
        child_process_call_foundry_migrate_safe_module(
            &network,
            &format!("[{}]", &&all_node_addresses.join(",")),
            &checksumed_safe_addr,
            &checksumed_module_addr,
        )
    }
}

impl Cmd for MigrateSafeModuleArgs {
    /// Run the execute_safe_module_migration function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_safe_module_migration()
    }
}
