use crate::key_pair::create_identity;
use clap::Parser;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::utils::{Cmd, HelperErrors};

#[derive(Parser, Default, Debug)]
pub struct IdentityArgs {
    #[clap(
        help = "Password to encrypt identity files",
        long,
        short,
        default_value = ""
    )]
    password: String,

    #[clap(
        help = "Path to the directory that stores identity files",
        long,
        short,
        default_value = "/tmp"
    )]
    directory: String,

    #[clap(help = "Name of the identity file", long, default_value = "node_")]
    name: Option<String>,

    #[clap(
        help = "Number of identities to be generated, e.g. 1",
        long,
        short,
        default_value_t = 1
    )]
    number: u32,
}

impl IdentityArgs {
    fn execute_identity_creation_loop(self) -> Result<(), HelperErrors> {
        let IdentityArgs {
            password,
            directory,
            name,
            number,
        } = self;

        let mut addresses = Vec::new();

        for _n in 1..=number {
            // build file name
            let id_name = match name {
                Some(ref provided_name) => Some(
                    provided_name.to_owned()
                        + &SystemTime::now()
                            .duration_since(UNIX_EPOCH)?
                            .as_secs()
                            .to_string(),
                ),
                None => None,
            };

            match create_identity(&directory, &password, &id_name) {
                Ok(addr) => addresses.push(addr),
                Err(_) => return Err(HelperErrors::UnableToCreateIdentity),
            }
        }

        println!("Addresses created with identities: {:?}", addresses);

        Ok(())
    }
}

impl Cmd for IdentityArgs {
    fn run(self) -> Result<(), HelperErrors> {
        self.execute_identity_creation_loop()
    }
}
