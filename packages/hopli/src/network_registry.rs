use crate::{
    identity_input::LocalIdentityArgs,
    key_pair::read_identities,
    password::PasswordArgs,
    process::{child_process_call_foundry_self_register, set_process_path_env},
    utils::{Cmd, HelperErrors},
};
use clap::Parser;
use log::{log, Level};
use std::env;
use utils_types::traits::PeerIdLike;

/// CLI arguments for `hopli register-in-network-registry`
#[derive(Parser, Default, Debug)]
pub struct RegisterInNetworkRegistryArgs {
    #[clap(help = "Network name. E.g. monte_rosa", long)]
    network: String,

    #[clap(
        help = "Comma sperated node peer ids",
        long,
        short,
        default_value = None
    )]
    peer_ids: Option<String>,

    #[clap(flatten)]
    local_identity: LocalIdentityArgs,

    #[clap(flatten)]
    password: PasswordArgs,

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
            peer_ids,
            password,
            contracts_root,
        } = self;

        // `PRIVATE_KEY` - Private key is required to send on-chain transactions
        if let Err(_) = env::var("PRIVATE_KEY") {
            return Err(HelperErrors::UnableToReadPrivateKey);
        }

        // collect all the peer ids
        let mut all_peer_ids = Vec::new();
        // add peer_ids from CLI, if there's one
        if let Some(provided_peer_ids) = peer_ids {
            all_peer_ids.push(provided_peer_ids);
        }

        // read all the identities from the directory
        let local_files = local_identity.get_files();
        // get peer ids and stringinfy them
        if local_files.len() > 0 {
            // check if password is provided
            let pwd = match password.read_password() {
                Ok(read_pwd) => read_pwd,
                Err(e) => return Err(e),
            };

            // read all the identities from the directory
            match read_identities(local_files, &pwd) {
                Ok(node_identities) => {
                    all_peer_ids.extend(node_identities.iter().map(|ni| ni.chain_key.1.to_peerid_str()));
                }
                Err(e) => {
                    println!("error {:?}", e);
                    return Err(e);
                }
            }
        }

        log!(target: "network_registry", Level::Info, "merged peer_ids {:?}", all_peer_ids.join(","));

        // set directory and environment variables
        if let Err(e) = set_process_path_env(&contracts_root, &network) {
            return Err(e);
        }

        // iterate and collect execution result. If error occurs, the entire operation failes.
        child_process_call_foundry_self_register(&network, &all_peer_ids.join(","))
    }
}

impl Cmd for RegisterInNetworkRegistryArgs {
    /// Run the execute_self_register function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_self_register()
    }
}
