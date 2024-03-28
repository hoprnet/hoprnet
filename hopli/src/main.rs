//! `hopli` is a collection of commands to help with identity creation, funding, registration, etc. for HOPR nodes

use crate::faucet::FaucetArgs;
use crate::identity::IdentitySubcommands;
use crate::network_registry::NetworkRegistrySubcommands;
use crate::safe_module::SafeModuleSubcommands;
use crate::utils::{Cmd, HelperErrors};
use clap::{Parser, Subcommand};
use tracing_subscriber::layer::SubscriberExt;
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
    about = "Helper to create node identities, fund nodes, manage network registry, manage HOPR staking safes and modules, etc."
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

    /// Commands around network registry.
    #[command(visible_alias = "nr")]
    NetworkRegistry {
        #[command(subcommand)]
        command: NetworkRegistrySubcommands,
    },

    /// Commands around safe module
    #[command(visible_alias = "sm")]
    SafeModule {
        #[command(subcommand)]
        command: SafeModuleSubcommands,
    },
}

#[async_std::main]
async fn main() -> Result<(), HelperErrors> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let format = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(false);

    let subscriber = tracing_subscriber::Registry::default().with(env_filter).with(format);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    let cli = Cli::parse();

    match cli.command {
        Commands::Identity { command } => {
            command.run()?;
        }
        Commands::Faucet(opt) => {
            opt.async_run().await?;
        }
        Commands::NetworkRegistry { command } => {
            command.async_run().await?;
        }
        Commands::SafeModule { command } => {
            command.async_run().await?;
        }
    }

    Ok(())
}
