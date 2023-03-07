use crate::faucet::FaucetArgs;
use crate::identity::IdentityArgs;
use crate::network_registry::RegisterInNetworkRegistryArgs;
use crate::utils::{Cmd, HelperErrors};
use clap::{Parser, Subcommand};
// use ethers::types::Address;
pub mod environment_config;
pub mod faucet;
pub mod identity;
pub mod key_pair;
pub mod network_registry;
pub mod password;
pub mod process;
pub mod utils;

#[derive(Parser, Debug)]
#[clap(name = "hopli")]
struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
#[clap(
    name = "HOPR ethereum package helper",
    author = "HOPR <tech@hoprnet.org>",
    version = "0.1",
    about = "Helper to create node identities, fund nodes, etc."
)]
enum Commands {
    #[clap(about = "Create and store identity files")]
    Identity(IdentityArgs),
    #[clap(
        about = "Fund given address and/or addressed derived from identity files native tokens or HOPR tokens"
    )]
    Faucet(FaucetArgs),
    #[clap(about = "Registry some nodes peer ids to the network registery contract")]
    RegisterInNetworkRegistry(RegisterInNetworkRegistryArgs),
}

fn main() -> Result<(), HelperErrors> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Identity(opt) => {
            opt.run()?;
        }
        Commands::Faucet(opt) => {
            opt.run()?;
        }
        Commands::RegisterInNetworkRegistry(opt) => {
            opt.run()?;
        }
    }

    Ok(())
}
