use crate::{
    identity_input::LocalIdentityArgs,
    key_pair::read_identities,
    password::PasswordArgs,
    process::{child_process_call_foundry_express_setup_safe_module, set_process_path_env},
    utils::{Cmd, HelperErrors},
};
use clap::Parser;
use ethers::{
    types::U256,
    utils::parse_units, //, types::U256, utils::format_units, ParseUnits
};
use hopr_crypto_types::keypairs::Keypair;
use log::{log, Level};
use std::env;

/// CLI arguments for `hopli create-safe-module`
#[derive(Parser, Default, Debug)]
pub struct CreateSafeModuleArgs {
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
}

impl CreateSafeModuleArgs {
    /// 1. Create a safe instance and a node management module instance:
    /// 2. Set default permissions for the module
    /// 3. Include node as a member with restricted permission on sending assets
    fn execute_safe_module_creation(self) -> Result<(), HelperErrors> {
        let CreateSafeModuleArgs {
            network,
            local_identity,
            password,
            contracts_root,
            hopr_amount,
            native_amount,
        } = self;

        // 1. `PRIVATE_KEY` - Private key is required to send on-chain transactions
        if env::var("PRIVATE_KEY").is_err() {
            return Err(HelperErrors::UnableToReadPrivateKey);
        }

        // 2. Calculate addresses from the identity file
        
        let pwd = password.read_password()?;

        // read all the identities from the directory
        let files = local_identity.get_files();
        let all_node_addresses: Vec<String> = read_identities(files, &pwd)?
            .values()
            .map(|ni| ni.chain_key.public().to_address().to_string())
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
}
