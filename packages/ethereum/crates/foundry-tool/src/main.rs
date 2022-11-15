use clap::{Parser, Subcommand};
use std::path::{Path};
use std::fs;

#[derive(Parser, Default, Debug)]
#[command(name = "Contract Deployment Helper")]
#[command(author = "HOPR <tech@hoprnet.org>")]
#[command(version = "0.1")]
#[command(about = "Toolchain to deploy multiple smart contracts, store deployment files and ", long_about = None)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    environment_name: String,

    #[arg(long, short, default_value_t = 0)]
    environment_type: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    // related to file structure
    Files {
        #[arg(short, long)]
        list: bool,
    },
    Faucet {
        #[arg(short = 'a', long)]
        address: Option<String>,
        #[arg(short, long)]
        password: String,
        #[arg(short, long)]
        use_local_identities: bool,
        #[arg(short = 'm', long)]
        amount: String,
        #[arg(short = 'd', long)]
        identity_directory: Option<String>,
        #[arg(short = 'x', long)]
        identity_prefix: Option<String>,
        #[arg(short, long)]
        token_type: String, // 'hopr' | 'native'
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Files { list }) => {
            let new_p = buildPath(&cli.environment_name, &cli.environment_type);
            println!("check if path {} is created", new_p);
        },
        Some(Commands::Faucet { address, password, use_local_identities, amount, identity_directory, identity_prefix, token_type }) => {
            // Check if local identity files should be used. Push all the read identities.
            let file_names;
            if use_local_identities {
                // read all the files from the directory
                match identity_directory {
                    None => println!("identity_directory must be provided"),
                    Some(identity_directory) => {
                        file_names = fs::read_dir(Path::new(&identity_directory)).unwrap();
                        for path in file_names {
                            println!("Name: {}", path.unwrap().path().display())
                        }
                    }
                }
            }
        },
        None => {}
    }
}


// fn saveFileToDeployments() -> std::io::Result<()> {}

fn buildPath(environment_name: &str, environment_type: &u8) -> String {
    let new_path = vec!["./", environment_name, "/", &environment_type.to_string()].concat();
    match Path::new(&new_path).to_str() {
        None => panic!("new path is not a valid UTF-8 sequence"),
        Some(s) => {
            println!("new path is {}", s);
            s.to_string()
        },
    }
}
