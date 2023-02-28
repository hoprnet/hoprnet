use crate::faucet::FaucetArgs;
use crate::utils::{Cmd, HelperErrors};
use clap::{Parser, Subcommand};
// use ethers::types::Address;
pub mod environment_config;
pub mod faucet;
pub mod key_pair;
pub mod process;
pub mod utils;

#[derive(Parser, Debug)]
#[clap(name = "foundry-tool")]
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
    // // related to file structure. TODO: Extend this command to store deployment files, if needed
    // #[clap(
    //     about = "Print path to the folder that should store deployment files for given envirionment and type."
    // )]
    // Files {
    //     #[clap(short, long)]
    //     list: bool,
    // },
    #[clap(
        about = "Fund given address and/or addressed derived from identity files native tokens or HOPR tokens"
    )]
    Faucet(FaucetArgs),
}

fn main() -> Result<(), HelperErrors> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Faucet(opt) => {
            opt.run()?;
        }
    }

    Ok(())
}
