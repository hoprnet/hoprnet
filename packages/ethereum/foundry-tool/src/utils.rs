pub trait Cmd: clap::Parser + Sized {
    fn run(self) -> Result<(), HelperErrors>;
}

#[derive(Debug)]
pub enum HelperErrors {
    UnableToReadIdentitiesFromPath(std::io::Error),
    UnableToParseAddress(String),
    EnvironmentInfoMismatch,
    UnableToSetFoundryRoot,
    ErrorInRunningFoundry,
}
