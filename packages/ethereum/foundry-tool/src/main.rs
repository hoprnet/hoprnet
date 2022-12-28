use clap::{Parser, Subcommand};
use ethers::types::Address;
use std::path::Path;
use std::process::Command;

mod helper_errors;
mod key_pair;
mod process;
use helper_errors::HelperErrors;

#[derive(Parser, Default, Debug)]
#[clap(
    name = "HOPR ethereum package helper",
    author = "HOPR <tech@hoprnet.org>",
    version = "0.1",
    about = "Helper to store deployment files and fund nodes"
)]
struct Cli {
    #[clap(long)]
    environment_name: String,

    #[clap(long, short)]
    environment_type: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    // related to file structure. TODO: Extend this command to store deployment files, if needed
    #[clap(
        about = "Print path to the folder that should store deployment files for given envirionment and type."
    )]
    Files {
        #[clap(short, long)]
        list: bool,
    },
    #[clap(
        about = "Fund given address and/or addressed derived from identity files native tokens or HOPR tokens"
    )]
    Faucet {
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
            help = "Make faucet script access and extract addresses from local identity files",
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
            help = "Specify the type of token to be sent ('hopr' or 'native'), if needed. Defaut value means sending both tokens",
            long,
            short,
            default_value = None
        )]
        token_type: Option<String>,

        #[clap(
            help = "Specify path pointing to the faucet make target",
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
    },
}

fn main() -> Result<(), HelperErrors> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Files { list }) => {
            if list {
                let new_p = process::build_path(&cli.environment_name, &cli.environment_type);
                println!("check if path {} is created", new_p);
            }
            Ok(())
        }
        Some(Commands::Faucet {
            address,
            password,
            use_local_identities,
            identity_directory,
            identity_prefix,
            token_type: _,
            make_root,
            private_key,
        }) => {
            // Include provided address
            let mut addresses_all = Vec::new();
            if let Some(addr) = address {
                // parse provided address string into `Address` type
                match addr.parse::<Address>() {
                    Ok(parsed_addr) => addresses_all.push(parsed_addr),
                    // TODO: Consider accept peer id here
                    Err(_) => return Err(HelperErrors::UnableToParseAddress(addr)),
                }
            }

            // Check if local identity files should be used. Push all the read identities.
            if use_local_identities {
                // read all the files from the directory
                if let Some(id_dir) = identity_directory {
                    match key_pair::read_identities(&id_dir.as_str(), &password, &identity_prefix) {
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
            if let Err(e) = process::set_process_path_env(&make_root, &private_key) {
                return Err(e);
            }

            // iterate and collect execution result. If error occurs, the entire operation failes.
            addresses_all
                .into_iter()
                .map(|a| {
                    process::child_process_call_make(
                        &cli.environment_name,
                        &cli.environment_type,
                        &a,
                    )
                })
                .collect()
        }
        None => Ok(()),
    }
}
