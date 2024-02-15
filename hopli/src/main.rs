// TODO: docs are missing
use crate::create_safe_module::CreateSafeModuleArgs;
use crate::faucet::FaucetArgs;
use crate::identity::IdentityArgs;
use crate::initialize_node::InitializeNodeArgs;
use crate::migrate_safe_module::MigrateSafeModuleArgs;
use crate::move_node_to_safe_module::MoveNodeToSafeModuleArgs;
use crate::network_registry::RegisterInNetworkRegistryArgs;
use crate::sync_network_registry::SyncNetworkRegistryArgs;
use crate::utils::{Cmd, HelperErrors};
use clap::{Parser, Subcommand};
use tracing_subscriber::layer::SubscriberExt;
pub mod create_safe_module;
pub mod environment_config;
pub mod faucet;
pub mod identity;
pub mod identity_input;
pub mod initialize_node;
pub mod key_pair;
pub mod migrate_safe_module;
pub mod move_node_to_safe_module;
pub mod network_registry;
pub mod password;
pub mod process;
pub mod sync_network_registry;
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
    #[clap(about = "Fund given address and/or addressed derived from identity files native tokens or HOPR tokens")]
    Faucet(FaucetArgs),
    #[clap(about = "Registry some nodes peer ids to the network registery contract")]
    RegisterInNetworkRegistry(RegisterInNetworkRegistryArgs),
    #[clap(about = "Necessary steps to initiate a node (network registery, stake, fund)")]
    InitializeNode(InitializeNodeArgs),
    #[clap(about = "Create a safe instance and a node management instance, configure default permissions")]
    CreateSafeModule(CreateSafeModuleArgs),
    #[clap(
        about = "Migrate an exising set of node(d) with safe and module to a new network, with default permissions"
    )]
    MigrateSafeModule(MigrateSafeModuleArgs),
    #[clap(about = "Move a registered node to a new safe and module pair")]
    MoveNodeToSafeModule(MoveNodeToSafeModuleArgs),
    #[clap(about = "Sync eligibility of safes on network registry")]
    SyncNetworkRegistry(SyncNetworkRegistryArgs),
}

fn main() -> Result<(), HelperErrors> {
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
        Commands::Identity(opt) => {
            opt.run()?;
        }
        Commands::Faucet(opt) => {
            opt.run()?;
        }
        Commands::RegisterInNetworkRegistry(opt) => {
            opt.run()?;
        }
        Commands::InitializeNode(opt) => {
            opt.run()?;
        }
        Commands::CreateSafeModule(opt) => {
            opt.run()?;
        }
        Commands::MigrateSafeModule(opt) => {
            opt.run()?;
        }
        Commands::MoveNodeToSafeModule(opt) => {
            opt.run()?;
        }
        Commands::SyncNetworkRegistry(opt) => {
            opt.run()?;
        }
    }

    Ok(())
}
