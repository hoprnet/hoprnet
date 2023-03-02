use std::time::SystemTimeError;

pub trait Cmd: clap::Parser + Sized {
    fn run(self) -> Result<(), HelperErrors>;
}

#[derive(Debug)]
pub enum HelperErrors {
    UnableToReadIdentitiesFromPath(std::io::Error),
    UnableToParseAddress(String),
    SystemTime(SystemTimeError),
    UnableToCreateIdentity,
    UnableToReadIdentity,
    EnvironmentInfoMismatch,
    UnableToSetFoundryRoot,
    ErrorInRunningFoundry,
    UnableToReadPassword,
    UnableToReadPrivateKey,
}

impl From<SystemTimeError> for HelperErrors {
    fn from(err: SystemTimeError) -> HelperErrors {
        HelperErrors::SystemTime(err)
    }
}
