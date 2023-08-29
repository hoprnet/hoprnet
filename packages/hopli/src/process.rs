use std::{
    env,
    ffi::OsStr,
    io::{self, Write},
    path::Path,
    process::Command,
};

use crate::environment_config;
use crate::utils::HelperErrors;

pub fn build_path(network: &str, environment_type: &str) -> String {
    let new_path = vec!["./", network, "/", &environment_type.to_string()].concat();
    match Path::new(&new_path).to_str() {
        None => panic!("new path is not a valid UTF-8 sequence"),
        Some(s) => {
            println!("new path is {}", s);
            s.to_string()
        }
    }
}

/// Set environment variables
///
/// # Arguments
///
/// * `contracts_root` - Directory to the foundry project
/// * `foundry_profile` - Value of FOUNDRY_PROFILE variable
/// * `network` - Name of the network that nodes run in
pub fn set_process_path_env(contracts_root: &Option<String>, network: &String) -> Result<(), HelperErrors> {
    // run in the repo where the make target is saved
    if let Some(new_root) = contracts_root {
        let root = Path::new(OsStr::new(&new_root));
        match env::set_current_dir(&root) {
            Ok(_) => println!("Successfully changed working directory to {}!", root.display()),
            Err(_) => return Err(HelperErrors::UnableToSetFoundryRoot),
        }
    }

    // get environment_type and set it as FOUNDRY_PROFILE
    if let Ok(foundry_profile) =
        environment_config::get_environment_type_from_name(&env::current_dir().unwrap(), network)
    {
        env::set_var("FOUNDRY_PROFILE", foundry_profile.to_string());
    } else {
        return Err(HelperErrors::UnableToSetFoundryRoot);
    }

    // use cmd to call process
    env::set_var("NETWORK", network);
    Ok(())
}

/// Launch a child process to call foundry faucet command
///
/// # Arguments
///
/// * `network` - Name of the environment that nodes run in
/// * `address` - Address that the tool fund
/// * `hopr_amount` - Amount of HOPR tokens to be funded
/// * `native_amount` - Amount of native tokens to be funded
pub fn child_process_call_foundry_faucet(
    network: &str,
    address: &String,
    hopr_amount: &str,
    native_amount: &str,
) -> Result<(), HelperErrors> {
    let hopr_amount_str = hopr_amount.to_string();
    let native_amount_str = native_amount.to_string();
    // let addresses_str = format!("{:#x}", &address);

    let faucet_args = vec![
        "script",
        "script/SingleAction.s.sol:SingleActionFromPrivateKeyScript",
        "--broadcast",
        "--sig",
        "transferOrMintHoprAndSendNativeToAmount(address,uint256,uint256)",
        &address,
        &hopr_amount_str,
        &native_amount_str,
    ];

    child_process_call_foundry(network, &faucet_args)
}

/// Launch a child process to call foundry self-register command
///
/// # Arguments
///
/// * `network` - Name of the network that nodes run in
/// * `environment_type` - Type of the environment that nodes run in
/// * `chain_keys` - Ethereum addresses of Chain keys of HOPR nodes to be registered under the caller (comma separated)
pub fn child_process_call_foundry_express_initialization(
    network: &str,
    ethereum_address: &String,
    hopr_amount: &str,
    native_amount: &str,
    chain_keys: &String,
) -> Result<(), HelperErrors> {
    // add brackets to around the string
    let chain_addresses_str = vec!["[", &chain_keys, "]"].concat();
    let self_register_args = vec![
        "script",
        "script/SingleAction.s.sol:SingleActionFromPrivateKeyScript",
        "--broadcast",
        "--sig",
        "expressInitialization(address[],uint256,uint256,string[])",
        &ethereum_address,
        &hopr_amount,
        &native_amount,
        &chain_addresses_str,
    ];

    child_process_call_foundry(network, &self_register_args)
}

/// Launch a child process to call foundry migrate-safe-module command
///
/// # Arguments
///
/// * `network` - Name of the network that nodes run in
/// * `environment_type` - Type of the environment that nodes run in
/// * `node_address` - Addresses of HOPR nodes to be included in the module
pub fn child_process_call_foundry_migrate_safe_module(
    network: &str,
    ethereum_address: &String,
    safe_address: &str,
    module_address: &str,
) -> Result<(), HelperErrors> {
    // add brackets to around the string
    let self_register_args = vec![
        "script",
        "script/SingleAction.s.sol:SingleActionFromPrivateKeyScript",
        "--broadcast",
        "--sig",
        "migrateSafeModule(address[],address,addressuint256,uint256)",
        &ethereum_address,
        &safe_address,
        &module_address,
    ];

    child_process_call_foundry(network, &self_register_args)
}

/// Launch a child process to call foundry express-setup-safe-module command
///
/// # Arguments
///
/// * `network` - Name of the network that nodes run in
/// * `environment_type` - Type of the environment that nodes run in
/// * `node_address` - Addresses of HOPR nodes to be included in the module
pub fn child_process_call_foundry_express_setup_safe_module(
    network: &str,
    ethereum_address: &String,
    hopr_amount: &str,
    native_amount: &str,
) -> Result<(), HelperErrors> {
    // add brackets to around the string
    let self_register_args = vec![
        "script",
        "script/SingleAction.s.sol:SingleActionFromPrivateKeyScript",
        "--broadcast",
        "--sig",
        "expressSetupSafeModule(address[],uint256,uint256)",
        &ethereum_address,
        &hopr_amount,
        &native_amount,
    ];

    child_process_call_foundry(network, &self_register_args)
}

/// Launch a child process to call foundry  command
///
/// # Arguments
///
/// * `network` - Name of the network that nodes run in
/// * `environment_type` - Type of the environment that nodes run in
/// * `peer_id` - Peer Ids of HOPR nodes to be registered under the caller
pub fn child_process_call_foundry_self_register(network: &str, peer_ids: &String) -> Result<(), HelperErrors> {
    // add brackets to around the string
    let peer_id_string = vec!["[", &peer_ids, "]"].concat();
    let self_register_args = vec![
        "script",
        "script/SingleAction.s.sol:SingleActionFromPrivateKeyScript",
        "--broadcast",
        "--sig",
        "selfRegisterNodes(string[])",
        &peer_id_string,
    ];

    child_process_call_foundry(network, &self_register_args)
}

/// Launch a child process to call a foundry script
///
/// # Arguments
///
/// * `network` - Name of the network that nodes run in
/// * `forge_args` - arguments to be passed to `forge`
pub fn child_process_call_foundry<T>(network: &str, forge_args: &[T]) -> Result<(), HelperErrors>
where
    T: AsRef<OsStr>,
{
    // check environment is set
    let environment_check = environment_config::ensure_environment_and_network_are_set(
        &env::current_dir().unwrap(),
        network,
        &env::var("FOUNDRY_PROFILE").unwrap(),
    )
    .unwrap();
    if !environment_check {
        return Err(HelperErrors::EnvironmentInfoMismatch);
    }

    // building the command
    let faucet_output = Command::new("forge")
        .args(forge_args)
        .output()
        .expect("forge command failed to start");
    io::stdout().write_all(&faucet_output.stdout).unwrap();
    io::stderr().write_all(&faucet_output.stderr).unwrap();

    println!("Foundry command execution status: {}", faucet_output.status);

    if faucet_output.status.success() {
        Ok(())
    } else {
        Err(HelperErrors::ErrorInRunningFoundry)
    }
}
