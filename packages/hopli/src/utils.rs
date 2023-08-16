use hoprd_keypair::errors::KeyPairError;
use std::time::SystemTimeError;
use thiserror::Error;

pub trait Cmd: clap::Parser + Sized {
    fn run(self) -> Result<(), HelperErrors>;
}

#[derive(Error, Debug)]
pub enum HelperErrors {
    /// Error propagated by IO operations
    #[error(transparent)]
    UnableToReadIdentitiesFromPath(#[from] std::io::Error),
    // UnableToReadIdentitiesFromPath(std::io::Error),
    #[error("error parsig address: {0:?}")]
    UnableToParseAddress(String),
    /// System time rrror
    #[error(transparent)]
    SystemTime(#[from] SystemTimeError),
    // SystemTime(SystemTimeError),
    #[error("unable to create identity")]
    UnableToCreateIdentity,

    #[error("incorrect filename: {0}")]
    IncorrectFilename(String),

    #[error("identity file exists: {0}")]
    IdentityFileExists(String),

    #[error("unable to read identity")]
    UnableToReadIdentity,

    #[error("unable to read identity directory")]
    MissingIdentityDirectory,

    #[error("unable to delete identity")]
    UnableToDeleteIdentity,

    #[error("environment info mismatch")]
    EnvironmentInfoMismatch,

    #[error("unable to set foundry root")]
    UnableToSetFoundryRoot,

    #[error("unable to run foundry")]
    ErrorInRunningFoundry,

    #[error("unable read password")]
    UnableToReadPassword,

    #[error("unable read private key")]
    UnableToReadPrivateKey,

    #[error(transparent)]
    KeyStoreError(#[from] KeyPairError),
}
