use std::path::{Path};
use std::env;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::process::Command;
use ethers::types::Address;

use crate::helper_errors::HelperErrors;

pub fn child_process_call_make (make_root: &Option<String>, private_key: &String, environment_name: &str, environment_type: &str, address: &Address) -> Result<(), HelperErrors> {
     // run in the repo where the make target is saved
     if let Some(new_root) = make_root {
        let root = Path::new(OsStr::new(&new_root));
        match env::set_current_dir(&root) {
            Ok(_) => println!("Successfully changed working directory to {}!", root.display()),
            Err(_) => return Err(HelperErrors::UnableToSetFoundryRoot)
        }
    }

    // use cmd to call process
    env::set_var("PRIVATE_KEY", private_key);
    let env_name_arg = &vec!["environment-name=", environment_name].concat();
    let env_type_arg = &vec!["environment-type=", environment_type].concat();
    let recipient_arg = &vec!["recipient=", &format!("{:#x}", &address)].concat(); // format is necessary due to fixed-hash crate's default behavior of eliding the middle part
    
    // building the command
    let faucet_output = Command::new("make").args(["faucet", env_name_arg, env_type_arg, recipient_arg]).output().expect("sh command failed to start");
    println!("Foundry command execution status: {}", faucet_output.status);
    io::stdout().write_all(&faucet_output.stdout).unwrap();
    io::stderr().write_all(&faucet_output.stderr).unwrap();

    if faucet_output.status.success() {
        return Ok(());
    } else {
        return Err(HelperErrors::ErrorInRunningFoundry);
    }
}
