#[derive(Debug)]
pub enum HelperErrors {
    UnableToReadIdentitiesFromPath(std::io::Error),
    UnableToParseAddress(String),
    UnableToSetFoundryRoot,
    ErrorInRunningFoundry
}