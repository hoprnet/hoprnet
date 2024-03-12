//! `hopli` is a collection of commands to help with identity creation, funding, registration, etc. for HOPR nodes

use crate::faucet::FaucetArgs;
use crate::identity::IdentitySubcommands;
use crate::network_registry::NetworkRegistryArgs;
use crate::safe_module::SafeModuleSubcommands;
use crate::utils::{Cmd, HelperErrors};
use clap::{Parser, Subcommand};
pub mod environment_config;
pub mod faucet;
pub mod identity;
pub mod key_pair;
pub mod methods;
pub mod network_registry;
pub mod safe_module;
pub mod utils;

#[derive(Parser, Debug)]
#[clap(name = "hopli")]
struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Helper for running your HOPR nodes
#[derive(Subcommand, Debug)]
#[clap(
    name = "HOPR ethereum package helper",
    author = "HOPR <tech@hoprnet.org>",
    version = "0.1",
    about = "Helper to create node identities, fund nodes, etc."
)]
enum Commands {
    /// Commands around identity
    #[command(visible_alias = "id")]
    Identity {
        #[command(subcommand)]
        command: IdentitySubcommands,
    },

    /// Fund given address and/or addressed derived from identity files native tokens or HOPR tokens
    #[clap(about = "Fund given address and/or addressed derived from identity files native tokens or HOPR tokens")]
    Faucet(FaucetArgs),

    /// Use a manager account to registry some nodes Ethereum address with its staking account address onto the network registry contract
    #[clap(
        about = "Registry some nodes Ethereum address with its staking account address onto the network registry contract. It requires a manager account to perform this action."
    )]
    NetworkRegistry(NetworkRegistryArgs),

    /// Commands around safe module
    #[command(visible_alias = "se")]
    SafeModule {
        #[command(subcommand)]
        command: SafeModuleSubcommands,
    },
}

#[async_std::main]
async fn main() -> Result<(), HelperErrors> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Identity { command } => {
            command.run()?;
        }
        Commands::Faucet(opt) => {
            opt.async_run().await?;
        }
        Commands::NetworkRegistry(opt) => {
            opt.async_run().await?;
        }
        Commands::SafeModule { command } => {
            command.async_run().await?;
        }
    }

    Ok(())
}
