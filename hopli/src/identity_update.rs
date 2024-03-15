use crate::identity_input::LocalIdentityArgs;
use crate::key_pair::{read_identity, update_identity_password};
use crate::password::PasswordsArgs;
use crate::utils::{Cmd, HelperErrors};
use clap::Parser;
use tracing::{debug, error};

#[derive(Parser, Clone, Debug)]
pub struct IdentityUpdateArgs {
    #[clap(flatten)]
    passwords: PasswordsArgs,

    #[clap(flatten)]
    local_identity: LocalIdentityArgs,
}

impl IdentityUpdateArgs {
    /// Execute the command with given parameters
    fn execute_identity_password_update(self) -> Result<(), HelperErrors> {
        let IdentityUpdateArgs {
            passwords,
            local_identity,
        } = self;

        let pwd = match passwords.read_old_password() {
            Ok(read_pwd) => read_pwd,
            Err(e) => return Err(e),
        };
        let new_pwd = match passwords.read_new_password() {
            Ok(read_pwd) => read_pwd,
            Err(e) => return Err(e),
        };

        if local_identity.identity_from_directory.is_none() {
            error!("Does not support file. Must provide an identity-directory");
            return Err(HelperErrors::MissingIdentityDirectory);
        }

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

        Ok(())
    }
}

impl Cmd for IdentityUpdateArgs {
    /// Run the execute_identity_password_update function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_identity_password_update()
    }
}
