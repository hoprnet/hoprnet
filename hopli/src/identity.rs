//! This module contains the arguments and functions around private keys and keystores.
//!
//! Keystore file is often referred as HOPR node identity file, which is an encrypted private key for an Ethereum wallet.
//! This identity file uses password (received from [PasswordArgs]) for encryption.
//!
//! Actions related to identity files are specified in [IdentityActionType].
//!
//! Location of identity files can be provided with [IdentityFileArgs].
//!
//! This module also contains definition of argument for private key, defined in [PrivateKeyArgs].
use clap::{builder::RangedU64ValueParser, Parser, ValueHint};
use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
use hopr_primitive_types::primitives::Address;
use hoprd_keypair::key_pair::HoprKeys;
use log::{debug, error, info};

use crate::key_pair::{create_identity, read_identities};
use crate::utils::{Cmd, HelperErrors};
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    str::FromStr,
};

/// An enum representing different actions around `hopli identiy`
#[derive(clap::ValueEnum, Debug, Clone, PartialEq, Eq)]
pub enum IdentityActionType {
    /// Create a new identity file
    Create,

    /// Read an existing identity file
    Read,
}

impl FromStr for IdentityActionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "c" | "create" => Ok(IdentityActionType::Create),
            "r" | "read" => Ok(IdentityActionType::Read),
            _ => Err(format!("Unknown identity action: {s}")),
        }
    }
}

/// Arguments for private key.
#[derive(Debug, Clone, Parser, Default)]
pub struct PrivateKeyArgs {
    /// Either provide a private key as argument or as an environment variable `PRIVATE_KEY`
    #[clap(
        long,
        short = 'k',
        help = "Private key to unlock the account that broadcasts the transaction",
        name = "private_key",
        value_name = "PRIVATE_KEY"
    )]
    pub private_key: Option<String>,
}

impl PrivateKeyArgs {
    /// Read the private key and return an address string
    pub fn read(self) -> Result<ChainKeypair, HelperErrors> {
        let private_key = if let Some(pk) = self.private_key {
            info!("reading private key from cli");
            pk
        } else {
            info!("reading private key from env PRIVATE_KEY");
            env::var("PRIVATE_KEY").map_err(HelperErrors::UnableToReadPrivateKey)?
        };

        // TODO:
        info!("To validate the private key");

        Ok(ChainKeypair::from_secret(hex::decode(&private_key).unwrap().as_slice()).unwrap())
    }
}

/// Arguments for password.
///
/// Password is used for encrypting an identity file
/// Password can be passed as an environment variable `IDENTITY_PASSWORD`, or
/// in a file of which the path is supplied in `--password_path`
#[derive(Debug, Clone, Parser, Default)]
pub struct PasswordArgs {
    /// The path to a file containing the password that encrypts the identity file
    #[clap(
        short,
        long,
        help = "The path to read the password. If not specified, the IDENTITY_PASSWORD environment variable.",
        value_hint = ValueHint::FilePath,
        name = "password_path",
        value_name = "PASSWORD_PATH"
    )]
    pub password_path: Option<PathBuf>,
}

impl PasswordArgs {
    /// Read the password either from its path or from the environment variable IDENTITY_PASSWORD
    pub fn read(self) -> Result<String, HelperErrors> {
        let pwd = if let Some(pwd_path) = self.password_path {
            info!("reading password from password_path");
            fs::read_to_string(pwd_path).map_err(HelperErrors::UnableToReadFromPath)?
        } else {
            info!("reading password from env IDENTITY_PASSWORD");
            env::var("IDENTITY_PASSWORD").map_err(|_| HelperErrors::UnableToReadPassword)?
        };

        Ok(pwd)
    }
}

/// CLI arguments to specify the directory of one or multiple identity files
#[derive(Debug, Clone, Parser, Default)]
pub struct IdentityFromDirectoryArgs {
    /// Directory to all the identity files
    #[arg(
        help = "Path to the directory that stores identity files",
        long,
        short = 'd',
        value_hint = ValueHint::DirPath,
        required = false
    )]
    pub identity_directory: Option<String>,

    /// Prefix of identity files. Only identity files with the provided are decrypted with the password
    #[arg(
        help = "Only use identity files with prefix",
        long,
        short = 'x',
        default_value = None,
        required = false
    )]
    pub identity_prefix: Option<String>,
}

impl IdentityFromDirectoryArgs {
    /// read files from given directory or file path
    pub fn get_files_from_directory(self) -> Result<Vec<PathBuf>, HelperErrors> {
        let IdentityFromDirectoryArgs {
            identity_directory,
            identity_prefix,
        } = self;
        let id_dir = identity_directory.unwrap();

        debug!(target: "identity_reader_from_directory", "Reading dir {}", &id_dir);

        // early return if failed in reading identity directory
        let directory = fs::read_dir(Path::new(&id_dir))?;
        // read all the files from the directory that contains
        // 1) "id" in its name
        // 2) the provided idetity_prefix
        let files: Vec<PathBuf> = directory
            .into_iter() // read all the files from the directory
            .filter(|r| r.is_ok()) // Get rid of Err variants for Result<DirEntry>
            .map(|r| r.unwrap().path()) // Read all the files from the given directory
            .filter(|r| r.is_file()) // Filter out folders
            .filter(|r| r.to_str().unwrap().contains("id")) // file name should contain "id"
            .filter(|r| match &identity_prefix {
                Some(id_prf) => r.file_stem().unwrap().to_str().unwrap().starts_with(id_prf.as_str()),
                _ => true,
            })
            .collect();
        info!(target: "identity_reader_from_directory", "{} path read from dir", &files.len());
        Ok(files)
    }
}

/// CLI arguments to specify the directory of one or multiple identity files
#[derive(Debug, Clone, Parser, Default)]
pub struct IdentityFileArgs {
    /// Directory that contains one or multiple identity files
    #[clap(help = "Get identity file(s) from a directory", flatten)]
    pub identity_from_directory: Option<IdentityFromDirectoryArgs>,

    /// Path to one identity file
    #[arg(
        short,
        long,
        help = "The path to an identity file",
        value_hint = ValueHint::FilePath,
        name = "identity_from_path"
    )]
    pub identity_from_path: Option<PathBuf>,

    /// Password to encrypt identity file(s)
    #[clap(help = "Password for the identit(ies)", flatten)]
    pub password: PasswordArgs,
}

impl IdentityFileArgs {
    /// read identity files from given directory or file path
    pub fn get_files(self) -> Vec<PathBuf> {
        let IdentityFileArgs {
            identity_from_directory,
            identity_from_path,
            ..
        } = self;
        debug!(target: "identity_location_reader", "Read from dir {}, path {}", &identity_from_directory.is_some(), &identity_from_path.is_some());

        let mut files: Vec<PathBuf> = Vec::new();
        if let Some(id_dir_args) = identity_from_directory {
            files = id_dir_args.get_files_from_directory().unwrap();
        };
        if let Some(id_path) = identity_from_path {
            debug!(target: "identity_location_reader", "Reading path {}", &id_path.as_path().display().to_string());
            if id_path.exists() {
                files.push(id_path);
                info!(target: "identity_location_reader", "path read from path");
            } else {
                error!(target: "identity_location_reader",  "Path {} does not exist.", &id_path.as_path().display().to_string());
            }
        }
        files
    }

    /// read identity files and return their Ethereum addresses
    pub fn to_addresses(self) -> Result<Vec<Address>, HelperErrors> {
        let files = self.clone().get_files();

        // get Ethereum addresses from identity files
        if !files.is_empty() {
            // check if password is provided
            let pwd = self.password.read()?;

            // read all the identities from the directory
            Ok(read_identities(files, &pwd)?
                .values()
                .map(|ni| ni.chain_key.public().0.to_address())
                .collect())
        } else {
            Ok(Vec::<Address>::new())
        }
    }
}

/// CLI arguments for `hopli identity`
#[derive(Parser, Clone, Debug)]
pub struct IdentityArgs {
    /// Possible actions around identity files
    #[clap(
        value_enum,
        long,
        short,
        help_heading = "Identity action",
        help = "Action with identity `create` or `read`"
    )]
    pub action: IdentityActionType,

    /// Arguments to locate one or multiple identity file(s)
    #[clap(help = "Action with identity `create` or `read`", flatten)]
    pub local_identity: IdentityFileArgs,

    /// Number of identities to be generated
    #[clap(
        help = "Number of identities to be generated, e.g. 1",
        long,
        short,
        value_parser = RangedU64ValueParser::<u32>::new().range(1..),
        default_value_t = 1
    )]
    pub number: u32,
}

impl IdentityArgs {
    /// Execute the command with given parameters
    fn execute_identity_creation_loop(self) -> Result<(), HelperErrors> {
        let IdentityArgs {
            action,
            local_identity,
            number,
        } = self;

        // check if password is provided
        let pwd = match local_identity.clone().password.read() {
            Ok(read_pwd) => read_pwd,
            Err(e) => return Err(e),
        };

        let mut node_identities: HashMap<String, HoprKeys> = HashMap::new();

        match action {
            IdentityActionType::Create => {
                if local_identity.identity_from_directory.is_none() {
                    error!("Does not support file. Must provide an identity-directory");
                    return Err(HelperErrors::MissingIdentityDirectory);
                }
                let local_id = local_identity.identity_from_directory.unwrap();
                let id_dir = local_id.identity_directory.unwrap();
                for index in 0..=number - 1 {
                    // build file name
                    let file_prefix = local_id
                        .identity_prefix
                        .as_ref()
                        .map(|provided_name| provided_name.to_owned() + &index.to_string());

                    match create_identity(&id_dir, &pwd, &file_prefix) {
                        Ok((id_filename, identity)) => _ = node_identities.insert(id_filename, identity),
                        Err(_) => return Err(HelperErrors::UnableToCreateIdentity),
                    }
                }
            }
            IdentityActionType::Read => {
                // read ids
                let files = local_identity.get_files();
                debug!("Identities read {:?}", files.len());
                match read_identities(files, &pwd) {
                    Ok(identities) => node_identities = identities,
                    Err(_) => return Err(HelperErrors::UnableToReadIdentity),
                }
            }
        }

        info!("Identities: {:?}", node_identities);
        Ok(())
    }
}

impl Cmd for IdentityArgs {
    /// Run the execute_identity_creation_loop function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_identity_creation_loop()
    }

    async fn async_run(self) -> Result<(), HelperErrors> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity;
    use tempfile::tempdir;

    const DUMMY_PRIVATE_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    #[test]
    fn private_key_args_can_read_env_or_cli_args_in_different_scenarios() {
        // possible private key args
        let pk_args_none = PrivateKeyArgs { private_key: None };
        let pk_args_some = PrivateKeyArgs {
            private_key: Some(DUMMY_PRIVATE_KEY.into()),
        };

        // when env is set but no cli arg, it returns the env value
        env::set_var("PRIVATE_KEY", DUMMY_PRIVATE_KEY);
        if let Ok(kp_1) = pk_args_none.clone().read() {
            assert_eq!(
                DUMMY_PRIVATE_KEY,
                hex::encode(kp_1.secret().as_ref()),
                "read a wrong private key from env"
            );
        } else {
            panic!("cannot read private key from env when no cli arg is provied");
        }

        // when both env and cli args are set, it still uses cli
        env::set_var("PRIVATE_KEY", "0123");
        if let Ok(kp_2) = pk_args_some.clone().read() {
            assert_eq!(
                DUMMY_PRIVATE_KEY,
                hex::encode(kp_2.secret().as_ref()),
                "read a wrong private key from cli"
            );
        } else {
            panic!("cannot read private key from cli when both are provied");
        }

        // when no env and no cli arg, it returns error
        env::remove_var("PRIVATE_KEY");
        assert!(pk_args_none.read().is_err());

        // when no env is supplied, but private key is supplied
        if let Ok(kp_3) = pk_args_some.read() {
            assert_eq!(
                DUMMY_PRIVATE_KEY,
                hex::encode(kp_3.secret().as_ref()),
                "read a wrong private key from env"
            );
        } else {
            panic!("cannot read private key from env when no cli arg is provied");
        }
    }

    #[test]
    fn password_args_can_read_env_or_cli_args_in_different_scenarios() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().to_str().unwrap();
        create_file(path, None, 1);

        // possible password args
        let pwd_args_some = PasswordArgs {
            password_path: Some(PathBuf::from(path).join("fileid1")),
        };
        let pwd_args_none = PasswordArgs { password_path: None };

        env::set_var("IDENTITY_PASSWORD", "Hello");
        // fail to take cli password path when both cli arg and env are supplied
        if let Ok(kp_1) = pwd_args_some.clone().read() {
            assert_eq!(kp_1, "Hello".to_string(), "read a wrong password from env");
        } else {
            panic!("cannot read password from env when cli arg is also provied");
        }
        // ok when no password path is supplied but env is supplied
        if let Ok(kp_2) = pwd_args_none.clone().read() {
            assert_eq!(kp_2, "Hello".to_string(), "read a wrong password from env");
        } else {
            panic!("cannot read password from env when no cli arg is provied");
        }

        // revert when no password path or identity password env is supplied
        env::remove_var("IDENTITY_PASSWORD");
        assert!(pwd_args_none.read().is_err());

        // ok when no env is supplied but password path is supplied
        if let Ok(kp_3) = pwd_args_some.clone().read() {
            assert_eq!(kp_3, "Hello".to_string(), "read a wrong password from path");
        } else {
            panic!("cannot read password from path when no env is provied");
        }
    }

    #[test]
    fn revert_get_dir_from_non_existing_dir() {
        let path = "./tmp_non_exist";

        let dir_args = IdentityFromDirectoryArgs {
            identity_directory: Some(path.to_string()),
            identity_prefix: None,
        };

        assert!(dir_args.get_files_from_directory().is_err());
    }

    #[test]
    fn pass_get_empty_dir_from_existing_dir() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().to_str().unwrap();
        create_file(path, None, 0);

        let dir_args = IdentityFromDirectoryArgs {
            identity_directory: Some(path.to_string()),
            identity_prefix: None,
        };

        if let Ok(vp) = dir_args.get_files_from_directory() {
            assert!(vp.is_empty())
        } else {
            panic!("failed to revert when the path contains no file")
        }
    }

    #[test]
    fn pass_get_dir_from_existing_dir() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().to_str().unwrap();
        create_file(path, None, 4);

        let dir_args = IdentityFromDirectoryArgs {
            identity_directory: Some(path.to_string()),
            identity_prefix: None,
        };

        if let Ok(vp) = dir_args.get_files_from_directory() {
            assert_eq!(4, vp.len())
        } else {
            panic!("failed to get files")
        }
    }

    #[test]
    fn pass_get_path_from_existing_path() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().to_str().unwrap();
        create_file(path, None, 4);

        let id_path = PathBuf::from(format!("{path}/fileid1"));
        let path_args: IdentityFileArgs = IdentityFileArgs {
            identity_from_directory: None,
            identity_from_path: Some(id_path),
            password: identity::PasswordArgs { password_path: None },
        };

        let vp = path_args.get_files();
        assert_eq!(1, vp.len());
    }

    #[test]
    fn pass_get_files_from_directory_and_path() {
        // an path to file
        let tmp_file = tempdir().unwrap();
        let path_file = tmp_file.path().to_str().unwrap();
        create_file(path_file, None, 4);
        let id_path = PathBuf::from(format!("{path_file}/fileid1"));

        // a dir for files
        let tmp = tempdir().unwrap();
        let path = tmp.path().to_str().unwrap();
        create_file(path, None, 4);

        let dir_args = IdentityFromDirectoryArgs {
            identity_directory: Some(path.to_string()),
            identity_prefix: None,
        };

        let path_args: IdentityFileArgs = IdentityFileArgs {
            identity_from_directory: Some(dir_args),
            identity_from_path: Some(id_path),
            password: identity::PasswordArgs { password_path: None },
        };

        let vp = path_args.get_files();
        assert_eq!(5, vp.len());
    }

    fn create_file(dir_name: &str, prefix: Option<String>, num: u32) {
        // create dir if not exist
        fs::create_dir_all(dir_name).unwrap();

        if num > 0 {
            for _n in 1..=num {
                let file_name = match prefix {
                    Some(ref file_prefix) => format!("{file_prefix}{_n}"),
                    None => format!("fileid{_n}"),
                };

                let file_path = PathBuf::from(dir_name).join(file_name);
                fs::write(&file_path, "Hello").unwrap();
            }
        }
    }
}
