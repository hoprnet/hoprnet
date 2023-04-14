use crate::{
    key_pair::read_identities,
    password::PasswordArgs,
    process::{child_process_call_foundry_express_initialization, set_process_path_env},
};
use clap::Parser;
use ethers::{
    types::U256,
    utils::parse_units, //, types::U256, utils::format_units, ParseUnits
};
use std::env;

use crate::utils::{Cmd, HelperErrors};

/// CLI arguments for `hopli register-in-network-registry`
#[derive(Parser, Default, Debug)]
pub struct InitializeNodeArgs {
    #[clap(help = "Environment name. E.g. monte_rosa", long)]
    environment_name: String,

    #[clap(
        help = "Path to the directory that stores identity files",
        long,
        short = 'd',
        default_value = "/tmp"
    )]
    identity_directory: Option<String>,

    #[clap(
        help = "Only use identity files with prefix",
        long,
        short = 'x',
        default_value = None
    )]
    identity_prefix: Option<String>,

    #[clap(flatten)]
    password: PasswordArgs,

    #[clap(
        help = "Specify path pointing to the contracts root",
        long,
        short,
        default_value = None
    )]
    contracts_root: Option<String>,

    #[clap(
        help = "Hopr amount in ether, to be funded for each identity, e.g. 10",
        long,
        short = 't',
        value_parser = clap::value_parser!(f64),
        default_value_t = 2000.0
    )]
    hopr_amount: f64,

    #[clap(
        help = "Native token amount in ether, to be funded for each identity, e.g. 1",
        long,
        short = 'n',
        value_parser = clap::value_parser!(f64),
        default_value_t = 10.0
    )]
    native_amount: f64,
}

impl InitializeNodeArgs {
    /// Stake + Register + Fund Express initialization:
    /// 1. `PRIVATE_KEY` env variable is required to send on-chain transactions
    /// 2. calculate the peer ID from the identity file
    /// 3. check if the peer ID is registered in network registry
    /// 4. if not, do all the necessary actions to register it
    /// 5. check if the corresponding wallet has at least 0.1 native and 10 tokens.
    /// 6. if not, fund it with the minimum amount
    fn execute_express_initialization(self) -> Result<(), HelperErrors> {
        let InitializeNodeArgs {
            environment_name,
            identity_directory,
            identity_prefix,
            password,
            contracts_root,
            hopr_amount,
            native_amount,
        } = self;

        // 1. `PRIVATE_KEY` - Private key is required to send on-chain transactions
        if let Err(_) = env::var("PRIVATE_KEY") {
            return Err(HelperErrors::UnableToReadPrivateKey);
        }

        // 2. Calculate the peerID and addresses from the identity file
        // collect all the peer ids
        let mut all_peer_ids = Vec::new();
        let mut all_node_addresses = Vec::new();
        // check if password is provided
        let pwd = match password.read_password() {
            Ok(read_pwd) => read_pwd,
            Err(e) => return Err(e),
        };
        // read all the identities from the directory
        if let Some(id_dir) = identity_directory {
            match read_identities(&id_dir, &pwd, &identity_prefix) {
                Ok(node_identities) => {
                    all_peer_ids = node_identities.iter().map(|ni| ni.peer_id.clone()).collect();
                    all_node_addresses = node_identities.iter().map(|ni| ni.ethereum_address.clone()).collect();
                }
                Err(e) => {
                    println!("error {:?}", e);
                    return Err(e);
                }
            }
        }

        // set directory and environment variables
        if let Err(e) = set_process_path_env(&contracts_root, &environment_name) {
            return Err(e);
        }

        // convert hopr_amount and native_amount from f64 to uint256 string
        let hopr_amount_uint256 = parse_units(hopr_amount, "ether").unwrap();
        let hopr_amount_uint256_string = U256::from(hopr_amount_uint256).to_string();
        let native_amount_uint256 = parse_units(native_amount, "ether").unwrap();
        let native_amount_uint256_string = U256::from(native_amount_uint256).to_string();

        // iterate and collect execution result. If error occurs, the entire operation failes.
        child_process_call_foundry_express_initialization(
            &environment_name,
            &format!("[{}]", &&all_node_addresses.join(",")),
            &hopr_amount_uint256_string,
            &native_amount_uint256_string,
            // &format!("[{}]", &all_peer_ids.join(",")),
            &all_peer_ids.join(","),
        )
    }
}

impl Cmd for InitializeNodeArgs {
    /// Run the execute_express_initialization function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_express_initialization()
    }
}
