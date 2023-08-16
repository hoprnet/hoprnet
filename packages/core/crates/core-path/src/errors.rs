use thiserror::Error;
use utils_db::errors::DbError;
use utils_types::errors::GeneralError;

#[derive(Error, Debug)]
pub enum PathError {
    #[error("path is not valid")]
    PathNotValid,

    #[error("path contains an invalid peer id: {0}")]
    InvalidPeer(String),

    #[error("missing channel between {0} and {1}")]
    MissingChannel(String, String),

    #[error("channel between {0} and {1} is not opened")]
    ChannelNotOpened(String, String),

    #[error("path contains loop on {0}")]
    LoopsNotAllowed(String),

    #[error(transparent)]
    DatabaseError(#[from] DbError),

    #[error(transparent)]
    OtherError(#[from] GeneralError),
}

pub type Result<T> = std::result::Result<T, PathError>;

#[cfg(feature = "wasm")]
impl From<PathError> for wasm_bindgen::JsValue {
    fn from(value: PathError) -> Self {
        value.to_string().into()
    }
}
