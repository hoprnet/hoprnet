use derive_more::derive::{Display, Error, From};
use semver::{Error as SemVerError, Version};

use crate::spec::{r#ref::RefError, schema::Error as SchemaError};

/// Spec Errors
#[derive(Debug, Display, Error, From)]
pub enum Error {
    #[display("Reference error")]
    Ref(RefError),

    #[display("Schema error")]
    Schema(SchemaError),

    #[display("Semver error")]
    SemVerError(SemVerError),

    #[display("Unsupported spec file version ({})", _0)]
    UnsupportedSpecFileVersion(#[error(not(source))] Version),
}
