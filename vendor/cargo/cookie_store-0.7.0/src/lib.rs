use failure::Fail;
use idna;

mod cookie;
pub use crate::cookie::Error as CookieError;
pub use crate::cookie::{Cookie, CookieResult};
mod cookie_domain;
mod cookie_expiration;
mod cookie_path;
mod cookie_store;
pub use crate::cookie_store::CookieStore;
mod utils;

#[derive(Debug, Fail)]
#[fail(display = "IDNA errors: {:#?}", _0)]
pub struct IdnaErrors(idna::uts46::Errors);

impl From<idna::uts46::Errors> for IdnaErrors {
    fn from(e: idna::uts46::Errors) -> Self {
        IdnaErrors(e)
    }
}

pub type Result<T> = std::result::Result<T, failure::Error>;
