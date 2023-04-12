use crate::key_pair::{create_identity, read_identities};
use crate::password::PasswordArgs;
use clap::{builder::RangedU64ValueParser, Parser};
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::utils::{Cmd, HelperErrors};

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

    #[clap(
        help = "Path to the directory that stores identity files",
        long,
        short,
        default_value = "/tmp/hopli"
    )]
    directory: String,

    #[clap(help = "Prefix of the identity file to create/read", long, default_value = "node_")]
    name: Option<String>,

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
            directory,
            name,
            number,
        } = self;

        // check if password is provided
        let pwd = match password.read_password() {
            Ok(read_pwd) => read_pwd,
            Err(e) => return Err(e),
        };

        match action {
            IdentityActionType::Create => {
                let mut addresses = Vec::new();

                for _n in 1..=number {
                    // build file name
                    let id_name = match name {
                        Some(ref provided_name) => Some(
                            provided_name.to_owned()
                                + &SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs().to_string(),
                        ),
                        None => None,
                    };

                    match create_identity(&directory, &pwd, &id_name) {
                        Ok(addr) => addresses.push(addr),
                        Err(_) => return Err(HelperErrors::UnableToCreateIdentity),
                    }
                }
                println!("Addresses from identities: {:?}", addresses);
                Ok(())
            }
            IdentityActionType::Read => {
                let mut node_identities = Vec::new();

                // read ids
                match read_identities(&directory, &pwd, &name) {
                    Ok(identities) => node_identities.extend(identities),
                    Err(_) => return Err(HelperErrors::UnableToReadIdentity),
                }
                println!("Identities: {:?}", node_identities);
                Ok(())
            }
        }
    }
}

impl Cmd for IdentityArgs {
    /// Run the execute_identity_creation_loop function
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_identity_creation_loop()
    }
}
