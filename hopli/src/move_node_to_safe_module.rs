use crate::{
    identity::IdentityFileArgs,
    process::{child_process_call_foundry_move_node_to_safe_module, set_process_path_env},
    utils::{Cmd, HelperErrors},
};
use clap::Parser;
use hopr_crypto_types::types::ToChecksum;
use hopr_primitive_types::primitives::Address;
use log::{log, Level};
use std::{env, str::FromStr};

/// CLI arguments for `hopli move-node-to-safe-module`
#[derive(Parser, Default, Debug)]
pub struct MoveNodeToSafeModuleArgs {
    #[clap(help = "Network name. E.g. monte_rosa", long)]
    network: String,

    #[clap(flatten)]
    local_identity: IdentityFileArgs,

    #[clap(
        help = "Specify path pointing to the contracts root",
        long,
        short,
        default_value = None
    )]
    contracts_root: Option<String>,

    #[clap(help = "Ethereum address of safe", long, short)]
    safe_address: String,

    #[clap(help = "Ethereum address of node management module", long, short)]
    module_address: String,
}

impl MoveNodeToSafeModuleArgs {
    /// 1. Include node to the new module
    /// 2. Deregister the old node-safe from node-safe registry
    /// 3. Registerr the new node-safe from node-safe registry
    /// 4. Remove node from network registry
    /// 5. Include node to network registry
    fn execute_moving_safe_module(self) -> Result<(), HelperErrors> {
        let MoveNodeToSafeModuleArgs {
            network,
            local_identity,
            contracts_root,
            safe_address,
            module_address,
        } = self;

        // 1. `PRIVATE_KEY` - Private key is required to send on-chain transactions
        if env::var("PRIVATE_KEY").is_err() {
            return Err(HelperErrors::UnableToReadPrivateKey);
        }

        // 2. Calculate addresses from the identity file
        let all_node_addresses: Vec<String> = local_identity
            .to_addresses()
            .unwrap()
            .into_iter()
            .map(|adr| adr.to_string())
            .collect();

        log!(target: "move_node_to_safe_module", Level::Info, "NodeAddresses {:?}", all_node_addresses.join(","));

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

        log!(target: "move_node_to_safe_module", Level::Debug, "Calling foundry...");
        // iterate and collect execution result. If error occurs, the entire operation failes.
        child_process_call_foundry_move_node_to_safe_module(
            &network,
            &format!("[{}]", &&all_node_addresses.join(",")),
            &checksumed_safe_addr,
            &checksumed_module_addr,
        )
    }
}

impl Cmd for MoveNodeToSafeModuleArgs {
    /// Run the execute_moving_safe_module function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_moving_safe_module()
    }
}
