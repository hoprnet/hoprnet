use derive_more::derive::{Display, Error, From};
use semver::{Error as SemverError, Version};

use crate::spec::{r#ref::RefError, schema::Error as SchemaError};

/// Spec errors.
#[derive(Debug, Display, Error, From)]
pub enum Error {
    /// Reference error.
    #[display("Reference error")]
    Ref(RefError),

    /// Schema error.
    #[display("Schema error")]
    Schema(SchemaError),

    /// Semver error.
    #[display("Semver error")]
    Semver(SemverError),

    /// Unsupported spec file version.
    #[display("Unsupported spec file version ({})", _0)]
    UnsupportedSpecFileVersion(#[error(not(source))] Version),
}
