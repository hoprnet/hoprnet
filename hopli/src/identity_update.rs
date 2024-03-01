use crate::identity_input::LocalIdentityArgs;
use crate::key_pair::{read_identity, update_identity_password};
use crate::password::PasswordArgs;
use crate::utils::{Cmd, HelperErrors};
use clap::Parser;
use tracing::{debug, error};

#[derive(Parser, Clone, Debug)]
pub struct IdentityUpdateArgs {
    #[clap(flatten)]
    old_password: PasswordArgs,

    #[clap(flatten)]
    new_password: PasswordArgs,

    #[clap(flatten)]
    local_identity: LocalIdentityArgs,
}

impl IdentityUpdateArgs {
    /// Execute the command with given parameters
    fn execute_identity_password_update(self) -> Result<(), HelperErrors> {
        let IdentityUpdateArgs {
            old_password,
            new_password,
            local_identity,
        } = self;

        let pwd = match old_password.read_password() {
            Ok(read_pwd) => read_pwd,
            Err(e) => return Err(e),
        };
        let new_pwd = match new_password.read_password() {
            Ok(read_pwd) => read_pwd,
            Err(e) => return Err(e),
        };

        if local_identity.identity_from_directory.is_none() {
            error!("Does not support file. Must provide an identity-directory");
            return Err(HelperErrors::MissingIdentityDirectory);
        }

        let files = local_identity.get_files();
        debug!("Identities read {:?}", files.len());

        for file in files {

            let keys = match read_identity(&file, &pwd) {
                Ok((_, keys)) => keys,
                Err(_) => return Err(HelperErrors::UnableToUpdateIdentityPassword),
            };

            match update_identity_password(keys, &file, &new_pwd) {
                Ok(_) => (),
                Err(_) => return Err(HelperErrors::UnableToUpdateIdentityPassword),
            }
        }

        Ok(())
    }
}

impl Cmd for IdentityUpdateArgs {
    /// Run the execute_identity_password_update function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_identity_password_update()
    }
}
