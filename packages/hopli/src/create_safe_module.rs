use crate::{
    identity_input::LocalIdentityArgs,
    key_pair::read_identities,
    password::PasswordArgs,
    process::{child_process_call_foundry_express_setup_safe_module, set_process_path_env},
    utils::{Cmd, HelperErrors},
};
use clap::Parser;
use core_crypto::keypairs::Keypair;
use log::{log, Level};
use std::env;

/// CLI arguments for `hopli create-safe-module`
#[derive(Parser, Default, Debug)]
pub struct CreateSafeModuleArgs {
    #[clap(help = "Network name. E.g. monte_rosa", long)]
    network: String,

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

impl CreateSafeModuleArgs {
    /// 1. Create a safe instance and a node management module instance:
    /// 2. Set default permissions for the module
    /// 3. Include node as a member with restricted permission on sending assets
    fn execute_safe_module_creation(self) -> Result<(), HelperErrors> {
        let CreateSafeModuleArgs {
            network,
            local_identity,
            password,
            contracts_root,
        } = self;

        // 1. `PRIVATE_KEY` - Private key is required to send on-chain transactions
        if let Err(_) = env::var("PRIVATE_KEY") {
            return Err(HelperErrors::UnableToReadPrivateKey);
        }

        // 2. Calculate addresses from the identity file
        // collect all the peer ids
        let all_node_addresses: Vec<String>;
        // check if password is provided
        let pwd = match password.read_password() {
            Ok(read_pwd) => read_pwd,
            Err(e) => return Err(e),
        };

        // read all the identities from the directory
        let files = local_identity.get_files();
        match read_identities(files, &pwd) {
            Ok(node_identities) => {
                all_node_addresses = node_identities
                    .values()
                    .map(|ni| ni.chain_key.public().to_address().to_string())
                    .collect();
            }
            Err(e) => {
                println!("error {:?}", e);
                return Err(e);
            }
        }
        log!(target: "create_safe_module", Level::Info, "NodeAddresses {:?}", all_node_addresses.join(","));

        // set directory and environment variables
        if let Err(e) = set_process_path_env(&contracts_root, &network) {
            return Err(e);
        }

        log!(target: "create_safe_module", Level::Debug, "Calling foundry...");
        // iterate and collect execution result. If error occurs, the entire operation failes.
        child_process_call_foundry_express_setup_safe_module(&network, &format!("[{}]", &&all_node_addresses.join(",")))
    }
}

impl Cmd for CreateSafeModuleArgs {
    /// Run the execute_safe_module_creation function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_safe_module_creation()
    }
}
