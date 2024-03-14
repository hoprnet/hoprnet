//! This module contains subcommands for `hopli identity`,
//! This command contains subcommands to read, create or update identity files, providing correct [crate::key_pair::PasswordArgs]
//!
//! For all three actions around identity files, at least two arguments are needed:
//! - Path to files
//! - Password to encrypt or decrypt the file
//! To update identity files, a new password must be provided.
//!
//! Some sample commands:
//!
//! - To read identities
//! ```text
//! hopli identity create \
//!     --identity-directory "./test" \
//!     --identity-prefix nodes_ \
//!     --number 2 \
//!     --password-path "./test/pwd"
//! ```
//!
//! - To create identities
//! ```text
//! hopli identity read \
//!     --identity-directory "./test" \
//!     --identity-prefix node_ \
//!     --password-path "./test/pwd"
//! ```
//!
//! - To update password of identities
//! ```text
//!     hopli identity update \
//!     --identity-directory "./test" \
//!     --identity-prefix node_ \
//!     --password-path "./test/pwd" \
//!     --new-password-path "./test/newpwd"
//! ```
use clap::{builder::RangedU64ValueParser, Parser};
use hopr_crypto_types::keypairs::Keypair;
use hopr_primitive_types::primitives::Address;
use hoprd_keypair::key_pair::HoprKeys;
use tracing::{debug, info};

use crate::key_pair::{
    create_identity, read_identities, read_identity, update_identity_password, ArgEnvReader, IdentityFileArgs,
    NewPasswordArgs,
};
use crate::utils::{Cmd, HelperErrors};
use std::collections::HashMap;

/// CLI arguments for `hopli identity`
#[derive(Clone, Debug, Parser)]
pub enum IdentitySubcommands {
    /// Create safe and module proxy if nothing exists
    #[command(visible_alias = "cr")]
    Create {
        /// Arguments to locate identity file(s) of HOPR node(s)
        #[command(flatten)]
        local_identity: IdentityFileArgs,

        /// Number of identities to be generated
        #[clap(
            help = "Number of identities to be generated, e.g. 1",
            long,
            short,
            value_parser = RangedU64ValueParser::<u32>::new().range(1..),
            default_value_t = 1
        )]
        number: u32,
    },

    /// Migrate safe and module to a new network
    #[command(visible_alias = "rd")]
    Read {
        /// Arguments to locate identity file(s) of HOPR node(s)
        #[command(flatten)]
        local_identity: IdentityFileArgs,
    },

    /// Update the password of identity files
    #[command(visible_alias = "up")]
    Update {
        /// Arguments to locate identity files of HOPR node(s)
        #[command(flatten)]
        local_identity: IdentityFileArgs,

        /// New password
        #[command(flatten)]
        new_password: NewPasswordArgs,
    },
}

impl IdentitySubcommands {
    /// Execute the command to create identities
    fn execute_identity_creation_loop(local_identity: IdentityFileArgs, number: u32) -> Result<(), HelperErrors> {
        // check if password is provided
        let pwd = local_identity.clone().password.read_default()?;

        let mut node_identities: HashMap<String, HoprKeys> = HashMap::new();

        let local_id = local_identity
            .identity_from_directory
            .ok_or(HelperErrors::MissingIdentityDirectory)
            .unwrap();

        let id_dir = local_id.identity_directory.unwrap();
        for index in 0..=number - 1 {
            // build file name
            let file_prefix = local_id
                .identity_prefix
                .as_ref()
                .map(|provided_name| provided_name.to_owned() + &index.to_string());

            let (id_filename, identity) =
                create_identity(&id_dir, &pwd, &file_prefix).map_err(|_| HelperErrors::UnableToCreateIdentity)?;
            node_identities.insert(id_filename, identity);
        }

        info!("Identities: {:?}", node_identities);
        Ok(())
    }

    /// Execute the command to read identities
    fn execute_identity_read_loop(local_identity: IdentityFileArgs) -> Result<(), HelperErrors> {
        // check if password is provided
        let pwd = local_identity.clone().password.read_default()?;

        // read ids
        let files = local_identity.get_files();
        debug!("Identities read {:?}", files.len());

        let node_identities: HashMap<String, HoprKeys> =
            read_identities(files, &pwd).map_err(|_| HelperErrors::UnableToReadIdentity)?;

        let node_addresses: Vec<Address> = node_identities
            .values()
            .map(|n| n.chain_key.public().to_address())
            .collect();

        info!("Identities: {:?}", node_identities);
        info!("Identity addresses: {:?}", node_addresses);
        Ok(())
    }

    /// update the password of an identity file
    fn execute_identity_update(
        local_identity: IdentityFileArgs,
        new_password: NewPasswordArgs,
    ) -> Result<(), HelperErrors> {
        // check if old password is provided
        let pwd = local_identity.clone().password.read_default()?;
        // check if new password is provided
        let new_pwd = new_password.read_default()?;

        // read ids
        let files = local_identity.get_files();
        debug!("Identities read {:?}", files.len());

        let _ = files
            .iter()
            .map(|file| {
                read_identity(file, &pwd)
                    .map_err(|_| HelperErrors::UnableToUpdateIdentityPassword)
                    .and_then(|(_, keys)| update_identity_password(keys, file, &new_pwd))
            })
            .collect::<Result<Vec<_>, _>>()?;

        info!("Updated password for {:?} identity files", files.len());
        Ok(())
    }
}

impl Cmd for IdentitySubcommands {
    /// Run the execute_identity_creation_loop function
    fn run(self) -> Result<(), HelperErrors> {
        match self {
            IdentitySubcommands::Create { local_identity, number } => {
                IdentitySubcommands::execute_identity_creation_loop(local_identity, number)
            }
            IdentitySubcommands::Read { local_identity } => {
                IdentitySubcommands::execute_identity_read_loop(local_identity)
            }
            IdentitySubcommands::Update {
                local_identity,
                new_password,
            } => IdentitySubcommands::execute_identity_update(local_identity, new_password),
        }
    }

    async fn async_run(self) -> Result<(), HelperErrors> {
        Ok(())
    }
}
