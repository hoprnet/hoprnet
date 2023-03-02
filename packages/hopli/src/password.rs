use crate::utils::HelperErrors;
use clap::{Parser, ValueHint};
use std::{env, fs::read_to_string, path::PathBuf};

/// Verification provider arguments
#[derive(Debug, Clone, Parser)]
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

impl Default for PasswordArgs {
    fn default() -> Self {
        PasswordArgs {
            password_path: None,
        }
    }
}

impl PasswordArgs {
    fn read_password(self) -> Result<String> {
        match self.password_path {
            Some(ref password_path) => {
                // read password from file
                let pwd_from_file =
                    read_to_string(constructor_args_path).expect("Fail to read password file");
                println!("pwd_from_file:\n{pwd_from_file}");
                Ok(pwd_from_file)
            }
            None => {
                // read password from environment variable
                let pwd_from_env = env::var("IDENTITY_PASSWORD")
                    .expect("Fail to read password from environment variable");
                println!("pwd_from_env:\n{pwd_from_env}");
                Ok(pwd_from_env)
            }
        }
    }
}
