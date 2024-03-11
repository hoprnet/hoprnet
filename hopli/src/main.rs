//! `hopli` is a collection of commands to help with identity creation, funding, registration, etc. for HOPR nodes

use crate::faucet::FaucetArgs;
use crate::identity::IdentityArgs;
use crate::migrate_safe_module::MigrateSafeModuleArgs;
use crate::move_node_to_safe_module::MoveNodeToSafeModuleArgs;
use crate::network_registry::NetworkRegistryArgs;
use crate::safe_module::SafeModuleSubcommands;
use crate::utils::{Cmd, HelperErrors};
use clap::{Parser, Subcommand};
pub mod environment_config;
pub mod faucet;
pub mod identity;
pub mod key_pair;
pub mod methods;
pub mod migrate_safe_module;
pub mod move_node_to_safe_module;
pub mod network_registry;
pub mod process;
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
    /// Create or read the node's identity file(s)
    #[clap(about = "Create and store identity files")]
    Identity(IdentityArgs),

    /// Fund given address and/or addressed derived from identity files native tokens or HOPR tokens
    #[clap(about = "Fund given address and/or addressed derived from identity files native tokens or HOPR tokens")]
    Faucet(FaucetArgs),

    /// Use a manager account to registry some nodes Ethereum address with its staking account address onto the network registry contract
    #[clap(
        about = "Registry some nodes Ethereum address with its staking account address onto the network registry contract. It requires a manager account to perform this action."
    )]
    NetworkRegistry(NetworkRegistryArgs),

    // /// Perform all the necessary steps before staring hopd.
    // /// - Create a Safe proxy instance and a node management instance. Include nodes to module
    // /// - Configure default permissions (for HOPR- Token, Channels, and Announcement contracts)
    // /// - Approve token transfer for the Safe proxy
    // /// - Fund Safe with tokens and fund nodes with xDAI
    // /// - Use the manager account to include the created Safe and provided node address to the Network Registry contract.
    // #[clap(
    //     about = "Create a safe proxy instance and a node management module instance, include nodes to the created module, configure default permissions, fund, register it to the Network Registry. It requires access to a manager account."
    // )]
    // CreateSafeModule(CreateSafeModuleArgs),
    /// Commands around safe module
    #[command(visible_alias = "se")]
    SafeModule {
        #[command(subcommand)]
        command: SafeModuleSubcommands,
    },

    /// Given existing node(s), safe and module, migrate them to a different network.
    /// It requires a manager account to perform this action.
    #[clap(
        about = "Migrate an exising set of node(d) with safe and module to a different network, with default permissions. It requires access to a manager account."
    )]
    MigrateSafeModule(MigrateSafeModuleArgs),

    /// Move nodes that are associated to an old safe to a new safe.
    /// It requires a manager account to perform this action.
    #[clap(about = "Move a registered node to a new safe and module pair. It requires access to a manager account.")]
    MoveNodeToSafeModule(MoveNodeToSafeModuleArgs),
}

#[async_std::main]
async fn main() -> Result<(), HelperErrors> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Identity(opt) => {
            opt.run()?;
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
        Commands::MigrateSafeModule(opt) => {
            opt.run()?;
        }
        Commands::MoveNodeToSafeModule(opt) => {
            opt.run()?;
        }
    }

    Ok(())
}
