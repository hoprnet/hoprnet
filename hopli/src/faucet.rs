use crate::{
    identity::{IdentityFileArgs, PrivateKeyArgs},
    process::{child_process_call_foundry_faucet, set_process_path_env},
    utils::{Cmd, HelperErrors},
};
use clap::Parser;
use ethers::{types::U256, utils::parse_units};
use hopr_crypto_types::types::ToChecksum;
use hopr_primitive_types::primitives::Address;
use log::{log, Level};
use std::str::FromStr;

/// CLI arguments for `hopli faucet`
#[derive(Parser, Default, Debug)]
pub struct FaucetArgs {
    #[clap(help = "Network name. E.g. monte_rosa", long)]
    network: String,

    #[clap(
        help = "Comma-separated Ethereum addresses of nodes that will receive funds",
        long,
        short,
        default_value = None
    )]
    address: Option<String>,

    #[clap(flatten)]
    local_identity: IdentityFileArgs,

    #[clap(
        help = "Specify path pointing to the contracts root",
        long,
        short,
        default_value = None
    )]
    contracts_root: Option<String>,

    #[clap(
        help = "Hopr amount in ether, e.g. 10",
        long,
        short = 't',
        value_parser = clap::value_parser!(f64),
        default_value_t = 2000.0
    )]
    hopr_amount: f64,

    #[clap(
        help = "Native token amount in ether, e.g. 1",
        long,
        short = 'n',
        value_parser = clap::value_parser!(f64),
        default_value_t = 10.0
    )]
    native_amount: f64,

    #[clap(flatten)]
    pub private_key: PrivateKeyArgs,
}

impl FaucetArgs {
    /// Execute the command with given parameters
    /// `PRIVATE_KEY` env variable is required to send on-chain transactions
    fn execute_faucet(self) -> Result<(), HelperErrors> {
        let FaucetArgs {
            network,
            address,
            local_identity,
            contracts_root,
            hopr_amount,
            native_amount,
            private_key,
        } = self;

        // `PRIVATE_KEY` - Private key is required to send on-chain transactions
        private_key.read()?;

        // Include provided address
        let mut addresses_all = Vec::new();

        // validate and arrayfy provided list of addresses
        if let Some(addresses) = address {
            let provided_addresses: Vec<String> = addresses
                .split(',')
                .map(|addr| Address::from_str(addr).unwrap().to_checksum())
                .collect();
            addresses_all.extend(provided_addresses);
        }

        // if local identity dirs/path is provided, read addresses from identity files
        addresses_all.extend(
            local_identity
                .to_addresses()
                .unwrap()
                .into_iter()
                .map(|adr| adr.to_string()),
        );

        log!(target: "faucet", Level::Info, "All the addresses: {:?}", addresses_all);

        // set directory and environment variables
        set_process_path_env(&contracts_root, &network)?;

        // convert hopr_amount and native_amount from f64 to uint256 string
        let hopr_amount_uint256 = parse_units(hopr_amount, "ether").unwrap();
        let hopr_amount_uint256_string = U256::from(hopr_amount_uint256).to_string();
        let native_amount_uint256 = parse_units(native_amount, "ether").unwrap();
        let native_amount_uint256_string = U256::from(native_amount_uint256).to_string();

        // iterate and collect execution result. If error occurs, the entire operation failes.
        addresses_all.into_iter().try_for_each(|a| {
            child_process_call_foundry_faucet(&network, &a, &hopr_amount_uint256_string, &native_amount_uint256_string)
        })
    }
}

impl Cmd for FaucetArgs {
    /// Run the execute_faucet function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_faucet()
    }
}
