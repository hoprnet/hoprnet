//! This module contains arguments and functions to quickly create necessary staking wallets
//! and execute necessary on-chain transactions to setup a HOPR node.
//!
//! Detailed breakdown of the steps:
//! - create a Safe proxy instance and HOPR node management module proxy instance
//! - include nodes configure default permissions on the created module proxy
//! - fund the node and Safe with some native tokens and HOPR tokens respectively
//! - approve HOPR tokens to be transferred from the Safe proxy instaces by Channels contract
//! - Use manager wallet to add nodes and staking safes to the Network Registry contract
use crate::{
    identity::{IdentityFileArgs, PrivateKeyArgs},
    process::{child_process_call_foundry_express_setup_safe_module, set_process_path_env},
    utils::{Cmd, HelperErrors},
};
use clap::Parser;
use ethers::{types::U256, utils::parse_units};
use log::{log, Level};

/// CLI arguments for `hopli create-safe-module`
#[derive(Parser, Default, Debug)]
pub struct CreateSafeModuleArgs {
    /// Name of the network that the node is running on
    #[clap(help = "Network name. E.g. monte_rosa", long)]
    network: String,

    /// Arguments to locate identity file(s) of HOPR node(s)
    #[clap(flatten)]
    local_identity: IdentityFileArgs,

    /// Path to the root of foundry project (etehereum/contracts), where all the contracts and `contracts-addresses.json` are stored
    #[clap(
        help = "Specify path pointing to the contracts root",
        long,
        short,
        default_value = None
    )]
    contracts_root: Option<String>,

    /// The amount of HOPR tokens (in floating number) to be funded per wallet
    #[clap(
        help = "Hopr amount in ether, e.g. 10",
        long,
        short = 't',
        value_parser = clap::value_parser!(f64),
        default_value_t = 2000.0
    )]
    hopr_amount: f64,

    /// The amount of native tokens (in floating number) to be funded per wallet
    #[clap(
        help = "Native token amount in ether, e.g. 1",
        long,
        short = 'n',
        value_parser = clap::value_parser!(f64),
        default_value_t = 10.0
    )]
    native_amount: f64,

    /// Access to the private key, of which the wallet either contains sufficient assets
    /// as the source of funds or it can mint necessary tokens
    /// This wallet is also the manager of Network Registry contract
    #[clap(flatten)]
    pub private_key: PrivateKeyArgs,
}

impl CreateSafeModuleArgs {
    /// Execute the command, which quickly create necessary staking wallets
    /// and execute necessary on-chain transactions to setup a HOPR node.
    ///
    /// 1. Create a safe instance and a node management module instance:
    /// 2. Set default permissions for the module
    /// 3. Include node as a member with restricted permission on sending assets
    fn execute_safe_module_creation(self) -> Result<(), HelperErrors> {
        let CreateSafeModuleArgs {
            network,
            local_identity,
            contracts_root,
            hopr_amount,
            native_amount,
            private_key,
        } = self;

        // 1. `PRIVATE_KEY` - Private key is required to send on-chain transactions
        private_key.read()?;

        // 2. Calculate addresses from the identity file
        let all_node_addresses: Vec<String> = local_identity
            .to_addresses()
            .unwrap()
            .into_iter()
            .map(|adr| adr.to_string())
            .collect();

        log!(target: "create_safe_module", Level::Info, "NodeAddresses {:?}", all_node_addresses.join(","));

        // set directory and environment variables
        set_process_path_env(&contracts_root, &network)?;

        // convert hopr_amount and native_amount from f64 to uint256 string
        let hopr_amount_uint256 = parse_units(hopr_amount, "ether").unwrap();
        let hopr_amount_uint256_string = U256::from(hopr_amount_uint256).to_string();
        let native_amount_uint256 = parse_units(native_amount, "ether").unwrap();
        let native_amount_uint256_string = U256::from(native_amount_uint256).to_string();

        log!(target: "create_safe_module", Level::Debug, "Calling foundry...");
        // iterate and collect execution result. If error occurs, the entire operation failes.
        child_process_call_foundry_express_setup_safe_module(
            &network,
            &format!("[{}]", &&all_node_addresses.join(",")),
            &hopr_amount_uint256_string,
            &native_amount_uint256_string,
        )
    }
}

impl Cmd for CreateSafeModuleArgs {
    /// Run the execute_safe_module_creation function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_safe_module_creation()
    }

    async fn async_run(self) -> Result<(), HelperErrors> {
        Ok(())
    }
}
