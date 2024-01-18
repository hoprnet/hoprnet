use crate::utils::HelperErrors;
use clap::{Parser, ValueHint};
use std::{env, fs::read_to_string, path::PathBuf};

/// Verification provider arguments
#[derive(Debug, Clone, Parser)]
#[derive(Default)]
pub struct PasswordArgs {
    #[clap(
        long,
        help = "The path to a file containing the password",
        long_help = "The path to read the password. If not specified, the IDENTITY_PASSWORD environment variable.",
        value_hint = ValueHint::FilePath,
        name = "password_path",
        value_name = "PASSWORD_PATH"
    )]
    password_path: Option<PathBuf>,
}



impl PasswordArgs {
    pub fn read_password(self) -> Result<String, HelperErrors> {
        match self.password_path {
            Some(ref password_path) => {
                // read password from file
                if let Ok(pwd_from_file) = read_to_string(password_path) {
                    Ok(pwd_from_file)
                } else {
                    println!("Cannot read from password_path");
                    Err(HelperErrors::UnableToReadPassword)
                }
            }
            None => {
                // read password from environment variable
                if let Ok(pwd_from_env) = env::var("IDENTITY_PASSWORD") {
                    Ok(pwd_from_env)
                } else {
                    println!("Cannot read from env var");
                    Err(HelperErrors::UnableToReadPassword)
                }
            }
        }
    }
}
