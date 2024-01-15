use crate::{
    process::{
        child_process_call_foundry_set_eligibility, child_process_call_foundry_sync_eligibility, set_process_path_env,
    },
    utils::{Cmd, HelperErrors},
};
use clap::Parser;
use hopr_crypto::types::ToChecksum;
use log::{error, log, Level};
use std::{env, iter, str::FromStr};
use primitive_types::primitives::Address;

#[derive(clap::ValueEnum, Debug, Clone, PartialEq, Eq)]
pub enum SyncNetworkRegistryType {
    ForcedSync,
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
    #[clap(help = "Network name. E.g. monte_rosa", long)]
    network: String,

    #[clap(
        help = "Specify path pointing to the contracts root",
        long,
        short,
        default_value = None
    )]
    contracts_root: Option<String>,

    #[clap(
        value_enum,
        long,
        short = 't',
        help_heading = "Sync type",
        help = "Type of syncing eligibility: `forced` or `normal`"
    )]
    pub sync_type: SyncNetworkRegistryType,

    #[clap(help = "Comma separated Ethereum addresses of safes", long, short)]
    safe_addresses: String,

    #[clap(help = "Desired eligibility in forced sync", long)]
    eligibility: Option<bool>,
}

impl SyncNetworkRegistryArgs {
    fn execute_sync_eligibility(self) -> Result<(), HelperErrors> {
        let SyncNetworkRegistryArgs {
            network,
            contracts_root,
            sync_type,
            safe_addresses,
            eligibility,
        } = self;

        // 1. `PRIVATE_KEY` - Private key is required to send on-chain transactions
        if env::var("PRIVATE_KEY").is_err() {
            return Err(HelperErrors::UnableToReadPrivateKey);
        }

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
