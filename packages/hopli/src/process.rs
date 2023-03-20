use ethers::types::Address;
use std::{
    env,
    ffi::OsStr,
    io::{self, Write},
    path::Path,
    process::Command,
};

use crate::environment_config;
use crate::utils::HelperErrors;

pub fn build_path(environment_name: &str, environment_type: &str) -> String {
    let new_path = vec!["./", environment_name, "/", &environment_type.to_string()].concat();
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
/// * `environment_name` - Name of the environment that nodes run in
pub fn set_process_path_env(
    contracts_root: &Option<String>,
    environment_name: &String,
) -> Result<(), HelperErrors> {
    // run in the repo where the make target is saved
    if let Some(new_root) = contracts_root {
        let root = Path::new(OsStr::new(&new_root));
        match env::set_current_dir(&root) {
            Ok(_) => println!(
                "Successfully changed working directory to {}!",
                root.display()
            ),
            Err(_) => return Err(HelperErrors::UnableToSetFoundryRoot),
        }
    }

    // get environment_type and set it as FOUNDRY_PROFILE
    if let Ok(foundry_profile) = environment_config::get_environment_type_from_name(
        &env::current_dir().unwrap(),
        environment_name,
    ) {
        env::set_var("FOUNDRY_PROFILE", foundry_profile.to_string());
    } else {
        return Err(HelperErrors::UnableToSetFoundryRoot);
    }

    // use cmd to call process
    env::set_var("ENVIRONMENT_NAME", environment_name);
    Ok(())
}

/// Launch a child process to call foundry faucet command
///
/// # Arguments
///
/// * `environment_name` - Name of the environment that nodes run in
/// * `address` - Address that the tool fund
/// * `hopr_amount` - Amount of HOPR tokens to be funded
/// * `native_amount` - Amount of native tokens to be funded
pub fn child_process_call_foundry_faucet(
    environment_name: &str,
    address: &Address,
    hopr_amount: &str,
    native_amount: &str,
) -> Result<(), HelperErrors> {
    let hopr_amount_str = hopr_amount.to_string();
    let native_amount_str = native_amount.to_string();
    let addresses_str = format!("{:#x}", &address);

    let faucet_args = vec![
        "script",
        "script/SingleAction.s.sol:SingleActionFromPrivateKeyScript",
        "--broadcast",
        "--sig",
        "transferOrMintHoprAndSendNative(address,uint256,uint256)",
        &addresses_str,
        &hopr_amount_str,
        &native_amount_str,
    ];

    child_process_call_foundry(environment_name, &faucet_args)
}

/// Launch a child process to call foundry self-register command
///
/// # Arguments
///
/// * `environment_name` - Name of the environment that nodes run in
/// * `environment_type` - Type of the environment that nodes run in
/// * `peer_id` - Peer Ids of HOPR nodes to be registered under the caller
pub fn child_process_call_foundry_self_register(
    environment_name: &str,
    peer_ids: &String,
) -> Result<(), HelperErrors> {
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

    child_process_call_foundry(environment_name, &self_register_args)
}

/// Launch a child process to call a foundry script
///
/// # Arguments
///
/// * `environment_name` - Name of the environment that nodes run in
/// * `forge_args` - arguments to be passed to `forge`
pub fn child_process_call_foundry<T>(
    environment_name: &str,
    forge_args: &[T],
) -> Result<(), HelperErrors>
where
    T: AsRef<OsStr>,
{
    // check environment is set
    let envrionment_check = environment_config::ensure_environment_is_set(
        &env::current_dir().unwrap(),
        environment_name,
        &env::var("FOUNDRY_PROFILE").unwrap(),
    )
    .unwrap();
    if !envrionment_check {
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
        return Ok(());
    } else {
        return Err(HelperErrors::ErrorInRunningFoundry);
    }
}
