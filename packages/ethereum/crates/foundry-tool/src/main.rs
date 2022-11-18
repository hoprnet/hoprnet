use clap::{Parser, Subcommand};
use std::path::{Path};
use ethers::types::Address;

mod key_pair;
use key_pair::HelperErrors;

#[derive(Parser, Default, Debug)]
#[clap(
    name = "HOPR ethereum package helper",
    author = "HOPR <tech@hoprnet.org>",
    version = "0.1",
    about = "Helper to store deployment files and fund nodes",
)]
struct Cli {
    #[clap(long)]
    environment_name: String,

    #[clap(long, short, default_value_t = 0)]
    environment_type: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    // related to file structure. TODO: Extend this command to store deployment files, if needed
    #[clap(about = "Print path to the folder that should store deployment files for given envirionment and type.")]
    Files {
        #[clap(short, long)]
        list: bool,
    },
    #[clap(about = "Fund given address and/or addressed derived from identity files native tokens or HOPR tokens")]
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
            help = "specify the type of token to be sent ('hopr' or 'native'), if needed. Defaut value means sending both tokens",
            long,
            short,
            default_value = None
        )]
        token_type: Option<String>,
    }
}

fn main()-> Result<(), HelperErrors> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Files { list }) => {
            if list {
                let new_p = build_path(&cli.environment_name, &cli.environment_type);
                println!("check if path {} is created", new_p);
            }
            Ok(())
        },
        Some(Commands::Faucet { address, password, use_local_identities, identity_directory, identity_prefix, token_type }) => {
            // Include provided address
            let mut addresses_all = Vec::new();
            if let Some(addr) = address {
                if let Ok(parsed_addr) = addr.parse::<Address>() {
                    addresses_all.push(parsed_addr)
                }
                match addr.parse::<Address>() {
                    Ok(parsed_addr) => addresses_all.push(parsed_addr),
                    Err(_) => return Err(HelperErrors::UnableToParseAddress(addr))
                }
            }
            
            // Check if local identity files should be used. Push all the read identities.
            if use_local_identities {
                // read all the files from the directory
                if let Some(id_dir) = identity_directory {
                    match key_pair::read_identities(&id_dir.as_str(), &password, &identity_prefix) {
                        Ok(addresses_from_identities) => {
                            addresses_all.extend(addresses_from_identities);
                        },
                        Err(e) => return Err(e)
                    }
                }
            }

            println!("All the addresses {:?}", addresses_all);
            Ok(())
        },
        None => {
            Ok(())
        }
    }
}


// fn saveFileToDeployments() -> std::io::Result<()> {}

fn build_path(environment_name: &str, environment_type: &u8) -> String {
    let new_path = vec!["./", environment_name, "/", &environment_type.to_string()].concat();
    match Path::new(&new_path).to_str() {
        None => panic!("new path is not a valid UTF-8 sequence"),
        Some(s) => {
            println!("new path is {}", s);
            s.to_string()
        },
    }
}
