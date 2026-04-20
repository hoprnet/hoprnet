#[derive(Debug, thiserror::Error)]
pub enum PixError {
    #[error("secret sharing error: {0}")]
    VsssError(vsss_rs::Error),
}

impl From<vsss_rs::Error> for PixError {
    fn from(err: vsss_rs::Error) -> Self {
        PixError::VsssError(err)
    }
}

pub type Result<T> = std::result::Result<T, PixError>;
