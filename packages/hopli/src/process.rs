use ethers::types::Address;
use std::env;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

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
/// * `make_root` - Directory to the foundry project
/// * `private_key` - Private key to send on-chain transactions
/// * `foundry_profile` - Value of FOUNDRY_PROFILE variable
/// * `environment_name` - Name of the environment that nodes run in
pub fn set_process_path_env(
    make_root: &Option<String>,
    private_key: &String,
    foundry_profile: &String,
    environment_name: &String,
) -> Result<(), HelperErrors> {
    // run in the repo where the make target is saved
    if let Some(new_root) = make_root {
        let root = Path::new(OsStr::new(&new_root));
        match env::set_current_dir(&root) {
            Ok(_) => println!(
                "Successfully changed working directory to {}!",
                root.display()
            ),
            Err(_) => return Err(HelperErrors::UnableToSetFoundryRoot),
        }
    }

    // use cmd to call process
    env::set_var("PRIVATE_KEY", private_key);
    env::set_var("FOUNDRY_PROFILE", foundry_profile);
    env::set_var("ENVIRONMENT_NAME", environment_name);
    Ok(())
}

/// Launch a child process to call foundry faucet command
///
/// # Arguments
///
/// * `environment_name` - Name of the environment that nodes run in
/// * `environment_type` - Type of the environment that nodes run in
/// * `address` - Address that the tool fund
/// * `hopr_amount` - Amount of HOPR tokens to be funded
/// * `native_amount` - Amount of native tokens to be funded
pub fn child_process_call_foundry_faucet(
    environment_name: &str,
    environment_type: &str,
    address: &Address,
    hopr_amount: &u128,
    native_amount: &u128,
) -> Result<(), HelperErrors> {
    // check environment is set
    let envrionment_check = environment_config::ensure_environment_is_set(
        &env::current_dir().unwrap(),
        environment_name,
        environment_type,
    )
    .unwrap();
    if !envrionment_check {
        return Err(HelperErrors::EnvironmentInfoMismatch);
    }

    // building the command
    let faucet_output = Command::new("forge")
        .args([
            "script",
            "script/SingleAction.s.sol:SingleActionFromPrivateKeyScript",
            "--broadcast",
            "--sig",
            "mintHoprAndSendNative(address,uint256,uint256)",
            &format!("{:#x}", &address),
            &hopr_amount.to_string(),
            &native_amount.to_string(),
        ])
        .output()
        .expect("forge faucet command failed to start");
    io::stdout().write_all(&faucet_output.stdout).unwrap();
    io::stderr().write_all(&faucet_output.stderr).unwrap();

    println!("Foundry command execution status: {}", faucet_output.status);

    if faucet_output.status.success() {
        return Ok(());
    } else {
        return Err(HelperErrors::ErrorInRunningFoundry);
    }
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
    environment_type: &str,
    peer_ids: &String,
) -> Result<(), HelperErrors> {
    // check environment is set
    let envrionment_check = environment_config::ensure_environment_is_set(
        &env::current_dir().unwrap(),
        environment_name,
        environment_type,
    )
    .unwrap();
    if !envrionment_check {
        return Err(HelperErrors::EnvironmentInfoMismatch);
    }

    // add brackets to around the string
    let peer_id_string = vec!["[", &peer_ids, "]"].concat();

    // building the command
    let faucet_output = Command::new("forge")
        .args([
            "script",
            "script/SingleAction.s.sol:SingleActionFromPrivateKeyScript",
            "--broadcast",
            "--sig",
            "selfRegisterNodes(string[])",
            &peer_id_string,
        ])
        .output()
        .expect("forge faucet command failed to start");
    io::stdout().write_all(&faucet_output.stdout).unwrap();
    io::stderr().write_all(&faucet_output.stderr).unwrap();

    println!("Foundry command execution status: {}", faucet_output.status);

    if faucet_output.status.success() {
        return Ok(());
    } else {
        return Err(HelperErrors::ErrorInRunningFoundry);
    }
}
