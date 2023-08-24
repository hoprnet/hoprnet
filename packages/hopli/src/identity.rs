use crate::identity_input::LocalIdentityArgs;
use crate::key_pair::{create_identity, read_identities};
use crate::password::PasswordArgs;
use crate::utils::{Cmd, HelperErrors};
use clap::{builder::RangedU64ValueParser, Parser};
use hoprd_keypair::key_pair::HoprKeys;
use log::{debug, error, info};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(clap::ValueEnum, Debug, Clone, PartialEq, Eq)]
pub enum IdentityActionType {
    Create,
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

/// CLI arguments for `hopli identity`
#[derive(Parser, Clone, Debug)]
pub struct IdentityArgs {
    #[clap(
        value_enum,
        long,
        short,
        help_heading = "Identity action",
        help = "Action with identity `create` or `read`"
    )]
    pub action: IdentityActionType,

    #[clap(flatten)]
    password: PasswordArgs,

    #[clap(flatten)]
    local_identity: LocalIdentityArgs,

    #[clap(
        help = "Number of identities to be generated, e.g. 1",
        long,
        short,
        value_parser = RangedU64ValueParser::<u32>::new().range(1..),
        default_value_t = 1
    )]
    number: u32,
}

impl IdentityArgs {
    /// Execute the command with given parameters
    fn execute_identity_creation_loop(self) -> Result<(), HelperErrors> {
        let IdentityArgs {
            action,
            password,
            local_identity,
            number,
        } = self;

        // check if password is provided
        let pwd = match password.read_password() {
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
                    let file_prefix = match &local_id.identity_prefix {
                        Some(ref provided_name) => Some(provided_name.to_owned() + &index.to_string()),
                        None => None,
                    };

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
        println!("{}", serde_json::to_string(&node_identities).unwrap());
        Ok(())
    }
}

impl Cmd for IdentityArgs {
    /// Run the execute_identity_creation_loop function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_identity_creation_loop()
    }
}
