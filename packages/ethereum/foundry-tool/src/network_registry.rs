use crate::process::{child_process_call_foundry_self_register, set_process_path_env};
use clap::Parser;

use crate::utils::{Cmd, HelperErrors};

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
        help = "Specify path pointing to the foundry root",
        long,
        short,
        default_value = None
    )]
    make_root: Option<String>,

    #[clap(
        help = "Private key of the caller address, e.g. 0xabc",
        long,
        short = 'k',
        default_value = None
    )]
    private_key: String,
}

impl NetworkRegistryArgs {
    fn execute_self_register(self) -> Result<(), HelperErrors> {
        let NetworkRegistryArgs {
            environment_name,
            environment_type,
            peer_ids,
            make_root,
            private_key,
        } = self;

        // set directory and environment variables
        if let Err(e) = set_process_path_env(
            &make_root,
            &private_key,
            &environment_type,
            &environment_name,
        ) {
            return Err(e);
        }

        // iterate and collect execution result. If error occurs, the entire operation failes.
        child_process_call_foundry_self_register(&environment_name, &environment_type, &peer_ids)
    }
}

impl Cmd for NetworkRegistryArgs {
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_self_register()
    }
}
