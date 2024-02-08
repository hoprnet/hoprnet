use crate::{
    identity::IdentityFileArgs,
    process::{child_process_call_foundry_self_register, set_process_path_env},
    utils::{Cmd, HelperErrors},
};
use clap::Parser;
use log::{log, Level};
use std::env;

/// CLI arguments for `hopli register-in-network-registry`
#[derive(Parser, Default, Debug)]
pub struct RegisterInNetworkRegistryArgs {
    #[clap(help = "Network name. E.g. monte_rosa", long)]
    network: String,

    #[clap(
        help = "Comma separated node peer ids",
        long,
        short,
        default_value = None
    )]
    peer_ids: Option<String>,

    #[clap(flatten)]
    local_identity: IdentityFileArgs,

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
            network,
            local_identity,
            peer_ids: chain_addresses,
            contracts_root,
        } = self;

        // `PRIVATE_KEY` - Private key is required to send on-chain transactions
        if env::var("PRIVATE_KEY").is_err() {
            return Err(HelperErrors::UnableToReadPrivateKey);
        }

        // collect all the peer ids
        let mut all_chain_addrs = Vec::new();
        // add peer_ids from CLI, if there's one
        if let Some(provided_chain_addrs) = chain_addresses {
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

        log!(target: "network_registry", Level::Info, "merged peer_ids {:?}", all_chain_addrs.join(","));

        // set directory and environment variables
        set_process_path_env(&contracts_root, &network)?;

        // iterate and collect execution result. If error occurs, the entire operation failes.
        child_process_call_foundry_self_register(&network, &all_chain_addrs.join(","))
    }
}

impl Cmd for RegisterInNetworkRegistryArgs {
    /// Run the execute_self_register function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_self_register()
    }
}
