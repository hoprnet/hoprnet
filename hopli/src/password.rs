use crate::utils::HelperErrors;
use clap::{Parser, ValueHint};
use std::{env, fs::read_to_string, path::PathBuf};

/// Verification provider arguments
#[derive(Debug, Clone, Parser, Default)]
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

/// Verification provider arguments
#[derive(Debug, Clone, Parser, Default)]
pub struct PasswordsArgs {
    #[clap(
        long,
        help = "The path to a file containing the old password",
        long_help = "The path to read the old password. If not specified, the IDENTITY_PASSWORD environment variable.",
        value_hint = ValueHint::FilePath,
        name = "old_password",
        value_name = "OLD_PASSWORD"
    )]
    old_password_path: Option<PathBuf>,

    #[clap(
        long,
        help = "The path to a file containing the new password",
        long_help = "The path to read the new password. If not specified, the NEW_IDENTITY_PASSWORD environment variable.",
        value_hint = ValueHint::FilePath,
        name = "new_password",
        value_name = "NEW_PASSWORD"
    )]
    new_password_path: Option<PathBuf>,
}

impl PasswordsArgs {
    fn read_password(&self, password_path: &Option<PathBuf>, default_key: &str) -> Result<String, HelperErrors> {
        match password_path {
            Some(ref password_path) => {
                // read password from file
                if let Ok(pwd_from_file) = read_to_string(password_path) {
                    Ok(pwd_from_file)
                } else {
                    println!("Cannot read from password_path {}", password_path.display());
                    Err(HelperErrors::UnableToReadPassword)
                }
            }
            None => {
                // read password from environment variable
                if let Ok(pwd_from_env) = env::var(default_key) {
                    Ok(pwd_from_env)
                } else {
                    println!("Cannot read from env var for {}", default_key);
                    Err(HelperErrors::UnableToReadPassword)
                }
            }
        }
    }
    pub fn read_old_password(&self) -> Result<String, HelperErrors> {
        self.read_password(&self.old_password_path, "IDENTITY_PASSWORD")
    }

    pub fn read_new_password(&self) -> Result<String, HelperErrors> {
        self.read_password(&self.new_password_path, "NEW_IDENTITY_PASSWORD")
    }
}
