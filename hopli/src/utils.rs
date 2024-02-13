//! This module contains errors produced in this crate
use hoprd_keypair::errors::KeyPairError;
use thiserror::Error;

pub trait Cmd: clap::Parser + Sized {
    fn run(self) -> Result<(), HelperErrors>;
}

/// Enumerates different errors produced by this crate.
#[derive(Error, Debug)]
pub enum HelperErrors {
    /// Error propagated by IO operations
    #[error(transparent)]
    UnableToReadFromPath(#[from] std::io::Error),

    /// Error in parsing provided comma-separated addresses
    #[error("error parsig address: {0:?}")]
    UnableToParseAddress(String),

    /// System time rrror
    #[error(transparent)]
    SystemTime(#[from] std::time::SystemTimeError),

    /// Error when identity cannot be created
    #[error("unable to create identity")]
    UnableToCreateIdentity,

    /// Error due to supplying a non-existing file name
    #[error("incorrect filename: {0}")]
    IncorrectFilename(String),

    /// Error when identity existed
    #[error("identity file exists: {0}")]
    IdentityFileExists(String),

    /// Fail to read identity
    #[error("unable to read identity")]
    UnableToReadIdentity,

    /// Fail to find the identity directory
    #[error("unable to read identity directory")]
    MissingIdentityDirectory,

    /// Fail to delete an identity
    #[error("unable to delete identity")]
    UnableToDeleteIdentity,

    /// Provided environement does not match with that in the `ethereum/contracts/contracts-addresses.json`
    #[error("environment info mismatch")]
    EnvironmentInfoMismatch,

    /// Wrong foundry contract root is provided
    #[error("unable to set foundry root")]
    UnableToSetFoundryRoot,

    /// Fail to run foundry
    #[error("unable to run foundry")]
    ErrorInRunningFoundry,

    /// Fail to read password
    #[error("unable read password")]
    UnableToReadPassword,

    /// Fail to read private key
    #[error("cannot read private key error: {0}")]
    UnableToReadPrivateKey(#[from] std::env::VarError),

    /// Paramters are missing
    #[error("missing parameter: {0}")]
    MissingParameter(String),

    /// Error with the keystore file
    #[error(transparent)]
    KeyStoreError(#[from] KeyPairError),
}
