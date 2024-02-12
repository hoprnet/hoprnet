use hoprd_keypair::errors::KeyPairError;
use thiserror::Error;

pub trait Cmd: clap::Parser + Sized {
    fn run(self) -> Result<(), HelperErrors>;
}

#[derive(Error, Debug)]
pub enum HelperErrors {
    /// Error propagated by IO operations
    #[error(transparent)]
    UnableToReadIdentitiesFromPath(#[from] std::io::Error),

    #[error("error parsig address: {0:?}")]
    UnableToParseAddress(String),

    /// System time rrror
    #[error(transparent)]
    SystemTime(#[from] std::time::SystemTimeError),

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

    #[error("cannot read private key error: {0}")]
    UnableToReadPrivateKey(#[from] std::env::VarError),

    #[error("missing parameter: {0}")]
    MissingParameter(String),

    #[error(transparent)]
    KeyStoreError(#[from] KeyPairError),
}
