use thiserror::Error;

#[derive(Error, Debug)]
pub enum HoprError {
    #[error("General error")]
    General(#[from] utils_types::errors::GeneralError),

    #[error("Other error")]
    Other(#[from] utils_db::errors::DbError),
}

pub type Result<T> = core::result::Result<T, HoprError>;
