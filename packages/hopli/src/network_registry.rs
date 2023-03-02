use crate::process::{child_process_call_foundry_self_register, set_process_path_env};
use clap::Parser;
use std::env;

use crate::utils::{Cmd, HelperErrors};

/// CLI arguments for `hopli network-registry`
#[derive(Parser, Default, Debug)]
pub struct NetworkRegistryArgs {
    #[clap(help = "Environment name. E.g. monte_rosa", long)]
    environment_name: String,

    #[clap(help = "Environment type. E.g. production", long, short)]
    environment_type: String,

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

impl NetworkRegistryArgs {
    /// Node self register with given parameters
    /// `PRIVATE_KEY` env variable is required to send on-chain transactions
    fn execute_self_register(self) -> Result<(), HelperErrors> {
        let NetworkRegistryArgs {
            environment_name,
            environment_type,
            peer_ids,
            contracts_root,
        } = self;

        // `PRIVATE_KEY` - Private key is required to send on-chain transactions
        if let Err(_) = env::var("PRIVATE_KEY") {
            return Err(HelperErrors::UnableToReadPrivateKey);
        }

        // set directory and environment variables
        if let Err(e) = set_process_path_env(&contracts_root, &environment_type, &environment_name)
        {
            return Err(e);
        }

        // iterate and collect execution result. If error occurs, the entire operation failes.
        child_process_call_foundry_self_register(&environment_name, &environment_type, &peer_ids)
    }
}

impl Cmd for NetworkRegistryArgs {
    /// Run the execute_self_register function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_self_register()
    }
}
