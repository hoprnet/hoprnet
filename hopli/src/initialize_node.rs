use crate::{
    identity_input::LocalIdentityArgs,
    key_pair::read_identities,
    password::PasswordArgs,
    process::{child_process_call_foundry_express_initialization, set_process_path_env},
    utils::{Cmd, HelperErrors},
};
use clap::Parser;
use core_crypto::keypairs::Keypair;
use ethers::{
    types::U256,
    utils::parse_units, //, types::U256, utils::format_units, ParseUnits
};
use log::{log, Level};
use std::env;

/// CLI arguments for `hopli register-in-network-registry`
#[derive(Parser, Default, Debug)]
pub struct InitializeNodeArgs {
    #[clap(help = "Network name. E.g. monte_rosa", long)]
    network: String,

    #[clap(flatten)]
    local_identity: LocalIdentityArgs,

    #[clap(flatten)]
    password: PasswordArgs,

    #[clap(
        help = "Specify path pointing to the contracts root",
        long,
        short,
        default_value = None
    )]
    contracts_root: Option<String>,

    #[clap(
        help = "Hopr amount in ether, to be funded for each identity, e.g. 10",
        long,
        short = 't',
        value_parser = clap::value_parser!(f64),
        default_value_t = 10.0
    )]
    hopr_amount: f64,

    #[clap(
        help = "Native token amount in ether, to be funded for each identity, e.g. 1",
        long,
        short = 'n',
        value_parser = clap::value_parser!(f64),
        default_value_t = 10.0
    )]
    native_amount: f64,
}

impl InitializeNodeArgs {
    /// Stake + Register + Fund Express initialization:
    /// 1. `PRIVATE_KEY` env variable is required to send on-chain transactions
    /// 2. calculate the peer ID from the identity file
    /// 3. check if the peer ID is registered in network registry
    /// 4. if not, do all the necessary actions to register it
    /// 5. check if the corresponding wallet has at least 0.1 native and 10 tokens.
    /// 6. if not, fund it with the minimum amount
    fn execute_express_initialization(self) -> Result<(), HelperErrors> {
        let InitializeNodeArgs {
            network,
            local_identity,
            password,
            contracts_root,
            hopr_amount,
            native_amount,
        } = self;

        // 1. `PRIVATE_KEY` - Private key is required to send on-chain transactions
        if let Err(_) = env::var("PRIVATE_KEY") {
            return Err(HelperErrors::UnableToReadPrivateKey);
        }

        // 2. Calculate the peerID and addresses from the identity file
        // collect all the peer ids
        let all_node_addresses: Vec<String>;
        // check if password is provided
        let pwd = match password.read_password() {
            Ok(read_pwd) => read_pwd,
            Err(e) => return Err(e),
        };

        // read all the identities from the directory
        let files = local_identity.get_files();
        match read_identities(files, &pwd) {
            Ok(node_identities) => {
                all_node_addresses = node_identities
                    .values()
                    .map(|ni| ni.chain_key.public().0.to_address().to_string())
                    .collect();
            }
            Err(e) => {
                println!("error {:?}", e);
                return Err(e);
            }
        }
        log!(target: "initialize_node", Level::Info, "NodeAddresses {:?}", all_node_addresses.join(","));

        // set directory and environment variables
        if let Err(e) = set_process_path_env(&contracts_root, &network) {
            return Err(e);
        }

        // convert hopr_amount and native_amount from f64 to uint256 string
        let hopr_amount_uint256 = parse_units(hopr_amount, "ether").unwrap();
        let hopr_amount_uint256_string = U256::from(hopr_amount_uint256).to_string();
        let native_amount_uint256 = parse_units(native_amount, "ether").unwrap();
        let native_amount_uint256_string = U256::from(native_amount_uint256).to_string();

        log!(target: "initialize_node", Level::Debug, "Calling foundry...");
        // iterate and collect execution result. If error occurs, the entire operation failes.
        child_process_call_foundry_express_initialization(
            &network,
            &format!("[{}]", &&all_node_addresses.join(",")),
            &hopr_amount_uint256_string,
            &native_amount_uint256_string,
            &all_node_addresses.join(","),
        )
    }
}

impl Cmd for InitializeNodeArgs {
    /// Run the execute_express_initialization function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_express_initialization()
    }
}
