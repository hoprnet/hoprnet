use crate::key_pair::read_identities;
use crate::process::{child_process_call_foundry_faucet, set_process_path_env};
use clap::Parser;
use ethers::types::Address;

use crate::utils::{Cmd, HelperErrors};

/// CLI arguments for `hopli faucet`
#[derive(Parser, Default, Debug)]
pub struct FaucetArgs {
    #[clap(help = "Environment name. E.g. monte_rosa", long)]
    environment_name: String,

    #[clap(help = "Environment type. E.g. production", long, short)]
    environment_type: String,

    #[clap(
        help = "Ethereum address of node that will receive funds",
        long,
        short,
        default_value = None
    )]
    address: Option<String>,

    #[clap(
        help = "Password to decrypt identity files",
        long,
        short,
        default_value = ""
    )]
    password: String,

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

    #[clap(
        help = "Hopr amount in ether, e.g. 10",
        long,
        short = 't',
        default_value_t = 2000
    )]
    hopr_amount: u128,

    #[clap(
        help = "Native token amount in ether, e.g. 1",
        long,
        short = 'n',
        default_value_t = 10
    )]
    native_amount: u128,
}

impl FaucetArgs {
    /// Execute the command with given parameters
    fn execute_faucet(self) -> Result<(), HelperErrors> {
        let FaucetArgs {
            environment_name,
            environment_type,
            address,
            password,
            use_local_identities,
            identity_directory,
            identity_prefix,
            make_root,
            private_key,
            hopr_amount,
            native_amount,
        } = self;

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
            // read all the files from the directory
            if let Some(id_dir) = identity_directory {
                match read_identities(&id_dir.as_str(), &password, &identity_prefix) {
                    Ok(addresses_from_identities) => {
                        addresses_all.extend(addresses_from_identities);
                    }
                    Err(e) => return Err(HelperErrors::UnableToReadIdentitiesFromPath(e)),
                }
            }
        }

        println!("All the addresses: {:?}", addresses_all);

        // TODO: by default, use faucet to fund both native tokens and HOPR tokens

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
        addresses_all
            .into_iter()
            .map(|a| {
                child_process_call_foundry_faucet(
                    &environment_name,
                    &environment_type,
                    &a,
                    &hopr_amount,
                    &native_amount,
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
