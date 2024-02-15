//! This module contains arguments and functions to sync eligibility node-staking safe pairs on the Network Registry contract.
use crate::{
    identity::PrivateKeyArgs,
    process::{
        child_process_call_foundry_set_eligibility, child_process_call_foundry_sync_eligibility, set_process_path_env,
    },
    utils::{Cmd, HelperErrors},
};
use clap::Parser;
use hopr_crypto_types::types::ToChecksum;
use hopr_primitive_types::primitives::Address;
use log::{error, log, Level};
use std::{iter, str::FromStr};

/// Two types of syncing the eligibility on the Network Registry
#[derive(clap::ValueEnum, Debug, Clone, PartialEq, Eq)]
pub enum SyncNetworkRegistryType {
    /// Forced sync by a privileged manager wallet
    ForcedSync,
    /// Normal sync by a wallet
    NormalSync,
}

impl FromStr for SyncNetworkRegistryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "f" | "forced" => Ok(SyncNetworkRegistryType::ForcedSync),
            "n" | "normal" => Ok(SyncNetworkRegistryType::NormalSync),
            _ => Err(format!("Unknown network registry sync type: {s}")),
        }
    }
}

/// CLI arguments for `hopli manage-network-registry`
#[derive(Parser, Clone, Debug)]
pub struct SyncNetworkRegistryArgs {
    /// Name of the network that the node is running on
    #[clap(help = "Network name. E.g. monte_rosa", long)]
    network: String,

    /// Path to the root of foundry project (etehereum/contracts), where all the contracts and `contracts-addresses.json` are stored
    #[clap(
        help = "Specify path pointing to the contracts root",
        long,
        short,
        default_value = None
    )]
    contracts_root: Option<String>,

    /// Type of sync. Either force or normal sync
    #[clap(
        value_enum,
        long,
        short = 't',
        help_heading = "Sync type",
        help = "Type of syncing eligibility: `forced` or `normal`"
    )]
    pub sync_type: SyncNetworkRegistryType,

    /// Address of the safe proxy instance
    #[clap(help = "Comma separated Ethereum addresses of safes", long, short)]
    safe_addresses: String,

    /// Desired eligibility of nodes to be synced
    #[clap(help = "Desired eligibility in forced sync", long)]
    eligibility: Option<bool>,

    /// Private key to execute the sync action. If the sync is forcely done, the private key must be a manager of Network Registry contract
    #[clap(flatten)]
    pub private_key: PrivateKeyArgs,
}

impl SyncNetworkRegistryArgs {
    /// Execute the command of syncing eligibility of node/safes on the Network Regsitry
    fn execute_sync_eligibility(self) -> Result<(), HelperErrors> {
        let SyncNetworkRegistryArgs {
            network,
            contracts_root,
            sync_type,
            safe_addresses,
            eligibility,
            private_key,
        } = self;

        // 1. `PRIVATE_KEY` - Private key is required to send on-chain transactions
        private_key.read()?;

        // 2. Read addresses of safes
        let all_safes_addresses: Vec<String> = safe_addresses
            .split(',')
            .map(|addr| Address::from_str(addr).unwrap().to_checksum())
            .collect();

        log!(target: "sync_eligibility", Level::Info, "Safe addresses {:?}", all_safes_addresses);

        // set directory and environment variables
        set_process_path_env(&contracts_root, &network)?;

        // Prepare payload and call function according to differnt types of sync
        match sync_type {
            SyncNetworkRegistryType::ForcedSync => {
                if let Some(desired_eligibility) = eligibility {
                    let all_eligibilities: Vec<String> = iter::repeat(desired_eligibility.to_string())
                        .take(all_safes_addresses.len())
                        .collect();
                    log!(target: "sync_eligibility", Level::Info, "Eligibilities {:?}", all_eligibilities);

                    log!(target: "sync_eligibility::forced", Level::Debug, "Calling foundry...");
                    // iterate and collect execution result. If error occurs, the entire operation failes.
                    child_process_call_foundry_set_eligibility(
                        &network,
                        &format!("[{}]", &&all_safes_addresses.join(",")),
                        &format!("[{}]", &&all_eligibilities.join(",")),
                    )
                } else {
                    error!("Eligibility must be specified");
                    Err(HelperErrors::MissingParameter("eligibility".to_string()))
                }
            }
            SyncNetworkRegistryType::NormalSync => {
                log!(target: "sync_eligibility::normal", Level::Debug, "Calling foundry...");
                child_process_call_foundry_sync_eligibility(&network, &format!("[{}]", &&all_safes_addresses.join(",")))
            }
        }
    }
}

impl Cmd for SyncNetworkRegistryArgs {
    /// Run the execute_sync_eligibility function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_sync_eligibility()
    }
}
