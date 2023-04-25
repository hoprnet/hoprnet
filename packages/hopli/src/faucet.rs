use crate::key_pair::read_identities;
use crate::password::PasswordArgs;
use crate::process::{child_process_call_foundry_faucet, set_process_path_env};
use clap::Parser;
use ethers::{
    types::{Address, U256},
    utils::parse_units, //, types::U256, utils::format_units, ParseUnits
};
use std::env;

use crate::utils::{Cmd, HelperErrors};

/// CLI arguments for `hopli faucet`
#[derive(Parser, Default, Debug)]
pub struct FaucetArgs {
    #[clap(help = "Environment name. E.g. monte_rosa", long)]
    network_id: String,

    #[clap(
        help = "Ethereum address of node that will receive funds",
        long,
        short,
        default_value = None
    )]
    address: Option<String>,

    #[clap(flatten)]
    password: PasswordArgs,

    #[clap(
        help = "Forge faucet script access and extract addresses from local identity files",
        long,
        short,
        default_value = "false"
    )]
    use_local_identities: bool,

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

    #[clap(
        help = "Specify path pointing to the contracts root",
        long,
        short,
        default_value = None
    )]
    contracts_root: Option<String>,

    #[clap(
        help = "Hopr amount in ether, e.g. 10",
        long,
        short = 't',
        value_parser = clap::value_parser!(f64),
        default_value_t = 2000.0
    )]
    hopr_amount: f64,

    #[clap(
        help = "Native token amount in ether, e.g. 1",
        long,
        short = 'n',
        value_parser = clap::value_parser!(f64),
        default_value_t = 10.0
    )]
    native_amount: f64,
}

impl FaucetArgs {
    /// Execute the command with given parameters
    /// `PRIVATE_KEY` env variable is required to send on-chain transactions
    fn execute_faucet(self) -> Result<(), HelperErrors> {
        let FaucetArgs {
            network_id,
            address,
            password,
            use_local_identities,
            identity_directory,
            identity_prefix,
            contracts_root,
            hopr_amount,
            native_amount,
        } = self;

        // `PRIVATE_KEY` - Private key is required to send on-chain transactions
        if let Err(_) = env::var("PRIVATE_KEY") {
            return Err(HelperErrors::UnableToReadPrivateKey);
        }

        // Include provided address
        let mut addresses_all = Vec::new();
        if let Some(addr) = address {
            // parse provided address string into `Address` type
            match addr.parse::<Address>() {
                Ok(parsed_addr) => addresses_all.push(parsed_addr),
                // TODO: Consider accept peer id here
                Err(_) => return Err(HelperErrors::UnableToParseAddress(addr.to_string())),
            }
        }

        // Check if local identity files should be used. Push all the read identities.
        if use_local_identities {
            // check if password is provided
            let pwd = match password.read_password() {
                Ok(read_pwd) => read_pwd,
                Err(e) => return Err(e),
            };

            // read all the files from the directory
            if let Some(id_dir) = identity_directory {
                match read_identities(&id_dir.as_str(), &pwd, &identity_prefix) {
                    Ok(addresses_from_identities) => {
                        addresses_all.extend(addresses_from_identities);
                    }
                    Err(e) => return Err(HelperErrors::UnableToReadIdentitiesFromPath(e)),
                }
            }
        }

        println!("All the addresses: {:?}", addresses_all);

        // set directory and environment variables
        if let Err(e) = set_process_path_env(&contracts_root, &network_id) {
            return Err(e);
        }

        // convert hopr_amount and native_amount from f64 to uint256 string
        let hopr_amount_uint256 = parse_units(hopr_amount, "ether").unwrap();
        let hopr_amount_uint256_string = U256::from(hopr_amount_uint256).to_string();
        let native_amount_uint256 = parse_units(native_amount, "ether").unwrap();
        let native_amount_uint256_string = U256::from(native_amount_uint256).to_string();

        // iterate and collect execution result. If error occurs, the entire operation failes.
        addresses_all
            .into_iter()
            .map(|a| {
                child_process_call_foundry_faucet(
                    &network_id,
                    &a,
                    &hopr_amount_uint256_string,
                    &native_amount_uint256_string,
                )
            })
            .collect()
    }
}

impl Cmd for FaucetArgs {
    /// Run the execute_faucet function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_faucet()
    }
}
