use crate::identity_input::LocalIdentityFromDirectoryArgs;
use crate::key_pair::{create_identity, read_identities};
use crate::password::PasswordArgs;
use crate::utils::{Cmd, HelperErrors};
use clap::{builder::RangedU64ValueParser, Parser};
use log::{log, Level};
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

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
    local_identity: LocalIdentityFromDirectoryArgs,

    // #[clap(
    //     help = "Path to the directory that stores identity files",
    //     long,
    //     short,
    //     default_value = "/tmp/hopli"
    // )]
    // directory: String,

    // #[clap(help = "Prefix of the identity file to create/read", long)]
    // name: Option<String>,
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
            // directory,
            // name,
            number,
        } = self;

        // check if password is provided
        let pwd = match password.read_password() {
            Ok(read_pwd) => read_pwd,
            Err(e) => return Err(e),
        };

        let mut node_identities = Vec::new();

        match action {
            IdentityActionType::Create => {
                for _n in 1..=number {
                    // build file name
                    match local_identity.identity_prefix {
                        Some(ref provided_name) => Some(
                            provided_name.to_owned()
                                + &SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs().to_string(),
                        ),
                        None => None,
                    };

                    match create_identity(
                        &local_identity.identity_directory,
                        &pwd,
                        &local_identity.identity_prefix,
                    ) {
                        Ok(identity) => node_identities.push(identity),
                        Err(_) => return Err(HelperErrors::UnableToCreateIdentity),
                    }
                }
            }
            IdentityActionType::Read => {
                // read ids
                let files = local_identity.get_files().unwrap();
                match read_identities(files, &pwd) {
                    Ok(identities) => node_identities.extend(identities),
                    Err(_) => return Err(HelperErrors::UnableToReadIdentity),
                }
            }
        }
        log!(target: "identity", Level::Info, "Identities: {:?}", node_identities);
        Ok(())
    }
}

impl Cmd for IdentityArgs {
    /// Run the execute_identity_creation_loop function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_identity_creation_loop()
    }
}
