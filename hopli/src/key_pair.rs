//! This module contains struct definition,  utility functions around private keys, password, and keystores.
//!
//! Keystore file is often referred as HOPR node identity file, which is an encrypted private key for an Ethereum
//! wallet. This identity file uses password (received from [PasswordArgs]) for encryption.
//!
//! Location of identity files can be provided with [IdentityFileArgs].
//!
//! This module also contains definition of argument for private key, defined in [PrivateKeyArgs].

use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use clap::{Parser, ValueHint};
use hopr_crypto_keypair::key_pair::{HoprKeys, IdentityRetrievalModes};
use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
use hopr_primitive_types::primitives::Address;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::utils::HelperErrors;

pub fn read_identity(file: &Path, password: &str) -> Result<(String, HoprKeys), HelperErrors> {
    let file_str = file
        .to_str()
        .ok_or(HelperErrors::IncorrectFilename(file.to_string_lossy().to_string()))?;

    match HoprKeys::read_eth_keystore(file_str, password) {
        Ok((keys, needs_migration)) => {
            if needs_migration {
                keys.write_eth_keystore(file_str, password)
                    .map_err(HelperErrors::KeyStoreError)?;
            }
            let file_key = file
                .file_name()
                .ok_or(HelperErrors::ParseError("Failed to extract file name".into()))?;
            Ok((
                String::from(
                    file_key
                        .to_str()
                        .ok_or(HelperErrors::ParseError("Failed to extract file key".into()))?,
                ),
                keys,
            ))
        }
        Err(e) => {
            error!("Could not decrypt keystore file at {}. {}", file_str, e.to_string());
            Err(HelperErrors::UnableToReadIdentity)
        }
    }
}

/// Decrypts identity files and returns an vec of PeerIds and Ethereum Addresses
///
/// # Arguments
///
/// * `files` - Paths to identity files
/// * `password` - Password to unlock all the identity files
pub fn read_identities(files: Vec<PathBuf>, password: &str) -> Result<HashMap<String, HoprKeys>, HelperErrors> {
    let mut results: HashMap<String, HoprKeys> = HashMap::with_capacity(files.len());

    for file in files.iter() {
        let id_path = file
            .to_str()
            .ok_or(HelperErrors::IncorrectFilename(file.to_string_lossy().to_string()))?;

        match HoprKeys::try_from(IdentityRetrievalModes::FromFile { password, id_path }) {
            Ok(keys) => {
                results.insert(id_path.into(), keys);
            }
            Err(e) => {
                warn!("Could not read keystore file at {} due to {}", id_path, e.to_string())
            }
        }
    }

    Ok(results)
}

/// encrypt HoprKeys with a new password to an identity file
pub fn update_identity_password(
    keys: HoprKeys,
    path: &Path,
    password: &str,
) -> Result<(String, HoprKeys), HelperErrors> {
    let file_path = path
        .to_str()
        .ok_or(HelperErrors::IncorrectFilename(path.to_string_lossy().to_string()))?;

    if path.exists() && path.is_file() && file_path.ends_with(".id") {
        // insert remove actual file with name `file_path`
        fs::remove_file(file_path).map_err(|_err| HelperErrors::UnableToUpdateIdentityPassword)?;
        keys.write_eth_keystore(file_path, password)?;
        Ok((String::from(file_path), keys))
    } else {
        warn!(
            "Could not update keystore file at {}. {}",
            file_path, "File name does not end with `.id`"
        );
        Err(HelperErrors::UnableToUpdateIdentityPassword)
    }
}

/// Create one identity file and return the ethereum address
///
/// # Arguments
///
/// * `dir_name` - Directory to the storage of an identity file
/// * `password` - Password to encrypt the identity file
/// * `name` - Prefix of identity files.
pub fn create_identity(
    dir_name: &str,
    password: &str,
    maybe_name: &Option<String>,
) -> Result<(String, HoprKeys), HelperErrors> {
    // create dir if not exist
    fs::create_dir_all(dir_name)?;

    let id = Uuid::new_v4();

    // check if `name` is end with `.id`, if not, append it
    let file_name = match maybe_name {
        Some(name) => {
            // check if ending with `.id`
            if name.ends_with(".id") {
                name.to_owned()
            } else {
                format!("{name}.id")
            }
        }
        // if none is specified, use UUID
        None => format!("{}.id", { id.to_string() }),
    };

    let mut file_path = PathBuf::from(dir_name);

    // add filename, depending on platform
    file_path.push(file_name);

    let file_path_str = file_path
        .to_str()
        .ok_or(HelperErrors::IncorrectFilename(file_path.to_string_lossy().to_string()))?;

    Ok((
        file_path_str.into(),
        IdentityRetrievalModes::FromIdIntoFile {
            id,
            password,
            id_path: file_path_str,
        }
        .try_into()?,
    ))
}

pub trait ArgEnvReader<T, K> {
    /// return the wrapped key
    fn get_key(&self) -> Option<K>;

    /// Try to read the value from the cli param (given by the key), or read from the env variable
    fn read(&self, default_env_name: &str) -> Result<T, HelperErrors>;

    /// Read the private key with a default env value and return an address string
    fn read_default(&self) -> Result<T, HelperErrors>;
}

/// Arguments for private key.
#[derive(Debug, Clone, Parser, Default)]
pub struct PrivateKeyArgs {
    /// Either provide a private key as argument or as a specific environment variable, e.g. `PRIVATE_KEY`,
    /// `MANAGER_PRIVATE_KEY`
    #[clap(
        long,
        short = 'k',
        help = "Private key to unlock the account that broadcasts the transaction",
        name = "private_key",
        value_name = "PRIVATE_KEY"
    )]
    pub private_key: Option<String>,
}

impl ArgEnvReader<ChainKeypair, String> for PrivateKeyArgs {
    /// Return the wrapped key. cli arg: --private-key
    fn get_key(&self) -> Option<String> {
        self.private_key.to_owned()
    }

    /// Read the value from either the cli arg or env
    fn read(&self, default_env_name: &str) -> Result<ChainKeypair, HelperErrors> {
        let pri_key = if let Some(pk) = self.get_key() {
            info!("Reading private key from CLI");
            pk
        } else if let Ok(env_pk) = env::var(default_env_name) {
            info!("Reading private key from environment variable {:?}", default_env_name);
            env_pk
        } else if let Ok(prompt_pk) = rpassword::prompt_password("Enter private key:") {
            info!("Reading private key from prompt");
            prompt_pk
        } else {
            error!(
                "Unable to read private key from environment variable: {:?}",
                default_env_name
            );
            return Err(HelperErrors::UnableToReadPrivateKey(default_env_name.into()));
        };

        // trim the 0x prefix if needed
        let priv_key_without_prefix = pri_key.strip_prefix("0x").unwrap_or(&pri_key).to_string();

        let decoded_key = hex::decode(priv_key_without_prefix)
            .map_err(|e| HelperErrors::UnableToReadPrivateKey(format!("Failed to decode private key: {e:?}")))?;
        ChainKeypair::from_secret(&decoded_key)
            .map_err(|e| HelperErrors::UnableToReadPrivateKey(format!("Failed to create keypair: {e:?}")))
    }

    /// Read the default private key and return an address string
    fn read_default(&self) -> Result<ChainKeypair, HelperErrors> {
        self.read("PRIVATE_KEY")
    }
}

/// Arguments for private key.
#[derive(Debug, Clone, Parser, Default)]
pub struct ManagerPrivateKeyArgs {
    /// Either provide a private key as argument or as a specific environment variable, e.g. `PRIVATE_KEY`,
    /// `MANAGER_PRIVATE_KEY`
    #[clap(
        long,
        short = 'q',
        help = "Private key to unlock the account with privilege that broadcasts the transaction",
        name = "manager_private_key",
        value_name = "MANAGER_PRIVATE_KEY"
    )]
    pub manager_private_key: Option<String>,
}

impl ArgEnvReader<ChainKeypair, String> for ManagerPrivateKeyArgs {
    /// Return the wrapped key. cli arg: --manager-private-key
    fn get_key(&self) -> Option<String> {
        self.manager_private_key.to_owned()
    }

    /// Read the value from either the cli arg or env
    fn read(&self, default_env_name: &str) -> Result<ChainKeypair, HelperErrors> {
        let pri_key = if let Some(pk) = self.get_key() {
            info!("Reading manager private key from CLI");
            pk
        } else if let Ok(env_pk) = env::var(default_env_name) {
            info!(
                "Reading manager private key from environment variable {:?}",
                default_env_name
            );
            env_pk
        } else if let Ok(prompt_pk) = rpassword::prompt_password("Enter manager private key:") {
            info!("Reading manager private key from prompt");
            prompt_pk
        } else {
            error!(
                "Unable to read private key from environment variable: {:?}",
                default_env_name
            );
            return Err(HelperErrors::UnableToReadPrivateKey(default_env_name.into()));
        };

        // trim the 0x prefix if needed
        let priv_key_without_prefix = pri_key.strip_prefix("0x").unwrap_or(&pri_key).to_string();
        let decoded_key = hex::decode(priv_key_without_prefix)
            .map_err(|e| HelperErrors::UnableToReadPrivateKey(format!("Failed to decode private key: {e:?}")))?;
        ChainKeypair::from_secret(&decoded_key)
            .map_err(|e| HelperErrors::UnableToReadPrivateKey(format!("Failed to create keypair: {e:?}")))
    }

    /// Read the default private key and return an address string
    fn read_default(&self) -> Result<ChainKeypair, HelperErrors> {
        self.read("MANAGER_PRIVATE_KEY")
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
        help = "The path to read the password. If not specified, use the IDENTITY_PASSWORD environment variable.",
        value_hint = ValueHint::FilePath,
        name = "password_path",
        value_name = "PASSWORD_PATH"
    )]
    pub password_path: Option<PathBuf>,
}

impl ArgEnvReader<String, PathBuf> for PasswordArgs {
    /// Return the wrapped key. cli arg: --password-path
    fn get_key(&self) -> Option<PathBuf> {
        self.password_path.clone()
    }

    /// Read the value from either the cli arg or env
    fn read(&self, default_env_name: &str) -> Result<String, HelperErrors> {
        let pwd = if let Some(pwd_path) = self.get_key() {
            info!("reading password from cli");
            fs::read_to_string(pwd_path).map_err(HelperErrors::UnableToReadFromPath)?
        } else {
            info!("reading password from env {:?}", default_env_name);
            env::var(default_env_name).map_err(|_| HelperErrors::UnableToReadPassword)?
        };

        Ok(pwd)
    }

    /// Read the default private key and return an address string
    fn read_default(&self) -> Result<String, HelperErrors> {
        self.read("IDENTITY_PASSWORD")
    }
}

/// Arguments for new password.
///
/// Password is used for encrypting an identity file
/// Password can be passed as an environment variable `NEW_IDENTITY_PASSWORD`, or
/// in a file of which the path is supplied in `--new_password_path`
#[derive(Debug, Clone, Parser, Default)]
pub struct NewPasswordArgs {
    /// The path to a file containing the password that encrypts the identity file
    #[clap(
        short,
        long,
        help = "The path to read the new password. If not specified, use the NEW_IDENTITY_PASSWORD environment variable.",
        value_hint = ValueHint::FilePath,
        name = "new_password_path",
        value_name = "NEW_IDENTITY_PASSWORD"
    )]
    pub new_password_path: Option<PathBuf>,
}

impl ArgEnvReader<String, PathBuf> for NewPasswordArgs {
    /// Return the wrapped key. cli arg: --new-password-path
    fn get_key(&self) -> Option<PathBuf> {
        self.new_password_path.clone()
    }

    /// Read the value from either the cli arg or env
    fn read(&self, default_env_name: &str) -> Result<String, HelperErrors> {
        let pwd = if let Some(pwd_path) = self.get_key() {
            info!("reading password from cli");
            fs::read_to_string(pwd_path).map_err(HelperErrors::UnableToReadFromPath)?
        } else {
            info!("reading password from env {:?}", default_env_name);
            env::var(default_env_name).map_err(|_| HelperErrors::UnableToReadPassword)?
        };

        Ok(pwd)
    }

    /// Read the default private key and return an address string
    fn read_default(&self) -> Result<String, HelperErrors> {
        self.read("NEW_IDENTITY_PASSWORD")
    }
}

/// CLI arguments to specify the directory of one or multiple identity files
#[derive(Debug, Clone, Parser, Default)]
pub struct IdentityFromDirectoryArgs {
    /// Directory to all the identity files
    #[clap(
        help = "Path to the directory that stores identity files",
        long,
        short = 'd',
        value_hint = ValueHint::DirPath,
        required = false
    )]
    pub identity_directory: Option<String>,

    /// Prefix of identity files. Only identity files with the provided are decrypted with the password
    #[clap(
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
        let id_dir = identity_directory.ok_or(HelperErrors::MissingIdentityDirectory)?;
        debug!(target: "identity_reader_from_directory", "Reading dir {}", &id_dir);

        // early return if failed in reading identity directory
        let directory = fs::read_dir(Path::new(&id_dir))?;
        // read all the files from the directory that contains
        // 1) "id" in its name
        // 2) the provided idetity_prefix
        let files: Vec<PathBuf> = directory
            .into_iter() // read all the files from the directory
            .filter_map(|r| r.ok())
            .map(|r| r.path()) // Read all the files from the given directory
            .filter(|r| r.is_file() && r.to_str().unwrap().contains("id")) // Filter out folders
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
    #[clap(
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
    pub fn get_files(self) -> Result<Vec<PathBuf>, HelperErrors> {
        let IdentityFileArgs {
            identity_from_directory,
            identity_from_path,
            ..
        } = self;
        debug!(target: "identity_location_reader", "Read from dir {}, path {}", &identity_from_directory.is_some(), &identity_from_path.is_some());

        let mut files: Vec<PathBuf> = Vec::new();
        if let Some(id_dir_args) = identity_from_directory {
            files = id_dir_args.get_files_from_directory()?;
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
        Ok(files)
    }

    /// read identity files and return their Ethereum addresses
    pub fn to_addresses(self) -> Result<Vec<Address>, HelperErrors> {
        let files = self.clone().get_files()?;

        // get Ethereum addresses from identity files
        if !files.is_empty() {
            // check if password is provided
            let pwd = self.password.read_default()?;

            // read all the identities from the directory
            Ok(read_identities(files, &pwd)?
                .values()
                .map(|ni| ni.chain_key.public().to_address())
                .collect())
        } else {
            Ok(Vec::<Address>::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use tempfile::tempdir;

    use super::*;

    const DUMMY_PRIVATE_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const SPECIAL_ENV_KEY: &str = "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

    #[test]
    fn read_pk_with_0x() -> anyhow::Result<()> {
        let private_key_args = PrivateKeyArgs {
            private_key: Some("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()),
        };
        let key = private_key_args.read_default()?;

        let ref_decoded_value = hex::decode(DUMMY_PRIVATE_KEY)?;
        println!("ref_decoded_value {ref_decoded_value:?}");

        assert_eq!(
            key.public().to_address().to_checksum(),
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            "cannot read private key with 0x prefix"
        );
        Ok(())
    }

    #[test]
    fn create_identities_from_directory_with_id_files() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();
        let tmp = tempdir()?;

        let path = tmp.path().to_str().context("should produce a valid tmp path string")?;
        let pwd = "password_create";
        assert!(create_identity(path, pwd, &Some(String::from("node1"))).is_ok());
        Ok(())
    }

    #[test]
    fn read_identity_from_path() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();
        let tmp = tempdir()?;

        let path = tmp.path().to_str().context("should produce a valid tmp path string")?;
        let pwd = "password";
        let (_, created_id) = create_identity(path, pwd, &None)?;

        // created and the read id is identical
        let files = get_files(path, &None);
        assert_eq!(files.len(), 1, "must have one identity file");

        let read_id = read_identity(files[0].as_path(), pwd)?;
        assert_eq!(
            read_id.1.chain_key.public().to_address(),
            created_id.chain_key.public().to_address()
        );
        Ok(())
    }

    #[test]
    fn update_identity_password_at_path() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();
        let tmp = tempdir()?;

        let path = tmp.path().to_str().context("should produce a valid tmp path string")?;
        let pwd = "password";
        let (_, created_id) = create_identity(path, pwd, &None)?;

        // created and the read id is identical
        let files = get_files(path, &None);
        assert_eq!(files.len(), 1, "must have one identity file");
        let address = created_id.chain_key.public().to_address();

        let new_pwd = "supersecured";
        let (_, returned_key) = update_identity_password(created_id, files[0].as_path(), new_pwd)?;

        // check the returned value
        assert_eq!(
            returned_key.chain_key.public().to_address(),
            address,
            "returned keys are identical"
        );

        // check the read value
        let (_, read_id) = read_identity(files[0].as_path(), new_pwd)?;
        assert_eq!(
            read_id.chain_key.public().to_address(),
            address,
            "cannot use the new password to read files"
        );
        Ok(())
    }

    #[test]
    fn read_identities_from_directory_with_id_files() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();
        let tmp = tempdir()?;

        let path = tmp.path().to_str().context("should produce a valid tmp path string")?;
        let pwd = "password";
        let (_, created_id) = create_identity(path, pwd, &None)?;

        // created and the read id is identical
        let files = get_files(path, &None);
        let read_id = read_identities(files, pwd)?;
        assert_eq!(read_id.len(), 1);
        assert_eq!(
            read_id.values().next().unwrap().chain_key.public().to_address(),
            created_id.chain_key.public().to_address()
        );

        // print the read id
        debug!("Debug {:#?}", read_id);
        debug!("Display {}", read_id.values().next().unwrap());
        Ok(())
    }

    #[test]
    fn read_identities_from_directory_with_id_files_but_wrong_password() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();
        let tmp = tempdir()?;

        let path = tmp.path().to_str().context("should produce a valid tmp path string")?;
        let pwd = "password";
        let wrong_pwd = "wrong_password";
        create_identity(path, pwd, &None)?;
        let files = get_files(path, &None);
        assert_eq!(read_identities(files, wrong_pwd)?.len(), 0);

        Ok(())
    }

    #[test]
    fn read_identities_from_directory_without_id_files() -> anyhow::Result<()> {
        let tmp = tempdir()?;

        let path = tmp.path().to_str().context("should produce a valid tmp path string")?;
        let files = get_files(path, &None);
        assert_eq!(read_identities(files, "")?.len(), 0);

        Ok(())
    }

    #[test]
    fn read_identities_from_tmp_folder() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();
        let tmp = tempdir()?;

        let path = tmp.path().to_str().context("should produce a valid tmp path string")?;
        let pwd = "local";
        create_identity(path, pwd, &Some("local-alice".into()))?;
        let files = get_files(path, &None);
        assert_eq!(read_identities(files, pwd)?.len(), 1);

        Ok(())
    }

    #[test]
    fn read_identities_from_tmp_folder_with_prefix() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();
        let tmp = tempdir()?;

        let path = tmp.path().to_str().context("should produce a valid tmp path string")?;
        let pwd = "local";
        create_identity(path, pwd, &Some("local-alice".into()))?;
        let files = get_files(path, &Some("local".to_string()));
        assert_eq!(read_identities(files, pwd)?.len(), 1);
        Ok(())
    }

    #[test]
    fn read_identities_from_tmp_folder_no_match() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();
        let tmp = tempdir()?;

        let path = tmp.path().to_str().context("should produce a valid tmp path string")?;
        let pwd = "local";
        create_identity(path, pwd, &Some("local-alice".into()))?;
        let files = get_files(path, &Some("npm-".to_string()));
        assert_eq!(read_identities(files, pwd)?.len(), 0);

        Ok(())
    }

    #[test]
    fn read_identities_from_tmp_folder_with_wrong_prefix() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();
        let tmp = tempdir()?;

        let path = tmp.path().to_str().context("should produce a valid tmp path string")?;
        let pwd = "local";
        create_identity(path, pwd, &Some("local-alice".into()))?;

        let files = get_files(path, &Some("alice".to_string()));
        assert_eq!(read_identities(files, pwd)?.len(), 0);

        Ok(())
    }

    #[test]
    fn read_complete_identities_from_tmp_folder() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();
        let tmp = tempdir()?;

        let path = tmp.path().to_str().context("should produce a valid tmp path string")?;
        let name = "alice.id";
        let pwd = "e2e-test";

        let weak_crypto_alice_keystore = r#"{"crypto":{"cipher":"aes-128-ctr","cipherparams":{"iv":"6084fab56497402930d0833fbc17e7ea"},"ciphertext":"50c0cf2537d7bc0ab6dbb7909d21d3da6445e5bd2cb1236de7efbab33302ddf1dd6a0393c986f8c111fe73a22f36af88858d79d23882a5f991713cb798172069d060f28c680afc28743e8842e8e849ebc21209825e23465afcee52a49f9c4f6734061f91a45b4cc8fbd6b4c95cc4c1b487f0007ed88a1b46b5ebdda616013b3f7ba465f97352b9412e69e6690cee0330c0b25bcf5fc3cdf12e4167336997920df9d6b7d816943ab3817481b9","kdf":"scrypt","kdfparams":{"dklen":32,"n":2,"p":1,"r":8,"salt":"46e30c2d74ba04b881e99fb276ae6a970974499f6abe286a00a69ba774ace095"},"mac":"70dccb366e8ddde13ebeef9a6f35bbc1333176cff3d33a72c925ce23753b34f4"},"id":"b5babdf4-da20-4cc1-9484-58ea24f1b3ae","version":3}"#;
        // let alice_peer_id = "16Uiu2HAmUYnGY3USo8iy13SBFW7m5BMQvC4NETu1fGTdoB86piw7";
        let alice_address = "0x838d3c1d2ff5c576d7b270aaaaaa67e619217aac";

        // create dir if not exist.
        fs::create_dir_all(path)?;
        // save the keystore as file
        fs::write(PathBuf::from(path).join(name), weak_crypto_alice_keystore.as_bytes())?;

        let files = get_files(path, &None);
        let val = read_identities(files, pwd)?;
        assert_eq!(val.len(), 1);
        assert_eq!(
            val.values().next().unwrap().chain_key.public().to_address().to_string(),
            alice_address
        );
        Ok(())
    }

    fn get_files(identity_directory: &str, identity_prefix: &Option<String>) -> Vec<PathBuf> {
        // early return if failed in reading identity directory
        let directory = fs::read_dir(Path::new(identity_directory))
            .unwrap_or_else(|_| panic!("cannot read directory {identity_directory}"));

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
                Some(identity_prefix) => r
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with(identity_prefix.as_str()),
                _ => true,
            })
            .collect();
        files
    }

    #[test]
    fn private_key_args_can_read_env_or_cli_args_in_different_scenarios() {
        // possible private key args
        let pk_args_none = PrivateKeyArgs { private_key: None };
        let pk_args_some = PrivateKeyArgs {
            private_key: Some(DUMMY_PRIVATE_KEY.into()),
        };

        // when a special env is set but no cli arg, it returns the special env value
        env::set_var("MANAGER_PK", SPECIAL_ENV_KEY);
        if let Ok(kp_0) = pk_args_none.clone().read("MANAGER_PK") {
            assert_eq!(
                SPECIAL_ENV_KEY,
                hex::encode(kp_0.secret().as_ref()),
                "read a wrong private key from env with a special name"
            );
        } else {
            panic!("cannot read private key from env when no cli arg is provied");
        }

        // when env is set but no cli arg, it returns the env value
        env::set_var("PRIVATE_KEY", DUMMY_PRIVATE_KEY);
        if let Ok(kp_1) = pk_args_none.clone().read_default() {
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
        if let Ok(kp_2) = pk_args_some.clone().read_default() {
            assert_eq!(
                DUMMY_PRIVATE_KEY,
                hex::encode(kp_2.secret().as_ref()),
                "read a wrong private key from cli"
            );
        } else {
            panic!("cannot read private key from cli when both are provied");
        }

        // when no env and no cli arg, it spawns an interactive CLI
        env::remove_var("PRIVATE_KEY");

        // when no env is supplied, but private key is supplied
        if let Ok(kp_3) = pk_args_some.read_default() {
            assert_eq!(
                DUMMY_PRIVATE_KEY,
                hex::encode(kp_3.secret().as_ref()),
                "read a wrong private key from env"
            );
        } else {
            panic!("cannot read private key from env when no cli arg is provied nor env is set");
        }

        // when no env and no cli arg, it spawns an interactive CLI and inputs
        // "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        // the test is commented out as there's no way to simulate the write in the interactive CLI,
        // unless directly mocking the buffer read.
        // if let Ok(kp_3) = pk_args_none.read_default() {
        //     assert_eq!(
        //         DUMMY_PRIVATE_KEY,
        //         hex::encode(kp_3.secret().as_ref()),
        //         "read a wrong private key from env"
        //     );
        // } else {
        //     panic!("cannot read private key from env when no cli arg is provied nor env is set");
        // }
    }

    #[test]
    fn password_args_can_read_env_or_cli_args_in_different_scenarios() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let path = tmp.path().to_str().context("should produce a valid tmp path string")?;
        create_file(path, None, 2)?;

        // possible password args
        let pwd_args_some = PasswordArgs {
            password_path: Some(PathBuf::from(path).join("fileid1")),
        };
        let pwd_args_none = PasswordArgs { password_path: None };
        let new_pwd_args_some = NewPasswordArgs {
            new_password_path: Some(PathBuf::from(path).join("fileid2")),
        };
        let new_pwd_args_none = NewPasswordArgs {
            new_password_path: None,
        };
        // let file_path = PathBuf::from(path).join("fileid2");
        fs::write(PathBuf::from(path).join("fileid2"), "supersound")?;

        // test new_password_path
        env::set_var("NEW_IDENTITY_PASSWORD", "ultraviolet");
        if let Ok(pwd_0) = new_pwd_args_some.read_default() {
            assert_eq!(
                pwd_0,
                "supersound".to_string(),
                "read a wrong password from path in new_password_path"
            );
        } else {
            panic!("cannot read new password from path");
        }
        if let Ok(pwd_1) = new_pwd_args_none.read_default() {
            assert_eq!(
                pwd_1,
                "ultraviolet".to_string(),
                "read a wrong password from cli in new_password_path"
            );
        } else {
            panic!("cannot read new password from path");
        }

        env::set_var("IDENTITY_PASSWORD", "Hello");
        // fail to take cli password path when both cli arg and env are supplied
        if let Ok(kp_1) = pwd_args_some.clone().read_default() {
            assert_eq!(kp_1, "Hello".to_string(), "read a wrong password from env");
        } else {
            panic!("cannot read password from env when cli arg is also provied");
        }
        // ok when no password path is supplied but env is supplied
        if let Ok(kp_2) = pwd_args_none.clone().read_default() {
            assert_eq!(kp_2, "Hello".to_string(), "read a wrong password from env");
        } else {
            panic!("cannot read password from env when no cli arg is provied");
        }

        // revert when no password path or identity password env is supplied
        env::remove_var("IDENTITY_PASSWORD");
        assert!(pwd_args_none.read_default().is_err());

        // ok when no env is supplied but password path is supplied
        if let Ok(kp_3) = pwd_args_some.clone().read_default() {
            assert_eq!(kp_3, "Hello".to_string(), "read a wrong password from path");
        } else {
            panic!("cannot read password from path when no env is provied");
        }
        Ok(())
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
    fn pass_get_empty_dir_from_existing_dir() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let path = tmp.path().to_str().context("should produce a valid tmp path string")?;
        create_file(path, None, 0)?;

        let dir_args = IdentityFromDirectoryArgs {
            identity_directory: Some(path.to_string()),
            identity_prefix: None,
        };

        if let Ok(vp) = dir_args.get_files_from_directory() {
            assert!(vp.is_empty())
        } else {
            panic!("failed to revert when the path contains no file")
        }
        Ok(())
    }

    #[test]
    fn pass_get_dir_from_existing_dir() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let path = tmp.path().to_str().context("should produce a valid tmp path string")?;
        create_file(path, None, 4)?;

        let dir_args = IdentityFromDirectoryArgs {
            identity_directory: Some(path.to_string()),
            identity_prefix: None,
        };

        if let Ok(vp) = dir_args.get_files_from_directory() {
            assert_eq!(4, vp.len())
        } else {
            panic!("failed to get files")
        }
        Ok(())
    }

    #[test]
    fn pass_get_path_from_existing_path() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let path = tmp.path().to_str().context("should produce a valid tmp path string")?;
        create_file(path, None, 4)?;

        let id_path = PathBuf::from(format!("{path}/fileid1"));
        let path_args: IdentityFileArgs = IdentityFileArgs {
            identity_from_directory: None,
            identity_from_path: Some(id_path),
            password: PasswordArgs { password_path: None },
        };

        let vp = path_args.get_files()?;
        assert_eq!(1, vp.len());
        Ok(())
    }

    #[test]
    fn pass_get_files_from_directory_and_path() -> anyhow::Result<()> {
        // an path to file
        let tmp_file = tempdir()?;
        let path_file = tmp_file.path().to_str().unwrap();
        create_file(path_file, None, 4)?;
        let id_path = PathBuf::from(format!("{path_file}/fileid1"));

        // a dir for files
        let tmp = tempdir()?;
        let path = tmp.path().to_str().context("should produce a valid tmp path string")?;
        create_file(path, None, 4)?;

        let dir_args = IdentityFromDirectoryArgs {
            identity_directory: Some(path.to_string()),
            identity_prefix: None,
        };

        let path_args: IdentityFileArgs = IdentityFileArgs {
            identity_from_directory: Some(dir_args),
            identity_from_path: Some(id_path),
            password: PasswordArgs { password_path: None },
        };

        let vp = path_args.get_files()?;
        assert_eq!(5, vp.len());

        Ok(())
    }

    fn create_file(dir_name: &str, prefix: Option<String>, num: u32) -> anyhow::Result<()> {
        // create dir if not exist
        fs::create_dir_all(dir_name)?;

        if num > 0 {
            for _n in 1..=num {
                let file_name = match prefix {
                    Some(ref file_prefix) => format!("{file_prefix}{_n}"),
                    None => format!("fileid{_n}"),
                };

                let file_path = PathBuf::from(dir_name).join(file_name);
                fs::write(&file_path, "Hello")?;
            }
        }
        Ok(())
    }
}
