use crate::process::{child_process_call_foundry_self_register, set_process_path_env};
use clap::Parser;
use std::env;

use crate::utils::{Cmd, HelperErrors};

/// CLI arguments for `hopli register-in-network-registry`
#[derive(Parser, Default, Debug)]
pub struct RegisterInNetworkRegistryArgs {
    #[clap(help = "Environment name. E.g. monte_rosa", long)]
    environment_name: String,

    #[clap(
        help = "Comma sperated node peer ids",
        long,
        short,
        default_value = None
    )]
    peer_ids: String,

    #[clap(
        help = "Specify path pointing to the contracts root",
        long,
        short,
        default_value = None
    )]
    contracts_root: Option<String>,
}

impl RegisterInNetworkRegistryArgs {
    /// Node self register with given parameters
    /// `PRIVATE_KEY` env variable is required to send on-chain transactions
    fn execute_self_register(self) -> Result<(), HelperErrors> {
        let RegisterInNetworkRegistryArgs {
            environment_name,
            peer_ids,
            contracts_root,
        } = self;

        // `PRIVATE_KEY` - Private key is required to send on-chain transactions
        if let Err(_) = env::var("PRIVATE_KEY") {
            return Err(HelperErrors::UnableToReadPrivateKey);
        }

        // set directory and environment variables
        if let Err(e) = set_process_path_env(&contracts_root, &environment_name) {
            return Err(e);
        }

        // iterate and collect execution result. If error occurs, the entire operation failes.
        child_process_call_foundry_self_register(&environment_name, &peer_ids)
    }
}

impl Cmd for RegisterInNetworkRegistryArgs {
    /// Run the execute_self_register function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_self_register()
    }
}
