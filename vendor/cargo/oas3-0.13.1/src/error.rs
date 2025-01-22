//! Error types

use std::io;

use derive_more::derive::{Display, Error, From};

use crate::spec::Error as SpecError;

/// Top-level errors.
#[derive(Debug, Display, Error, From)]
pub enum Error {
    /// I/O error.
    #[display("I/O error")]
    Io(io::Error),

    /// YAML error.
    #[display("YAML error")]
    Yaml(serde_yml::Error),

    /// JSON error.
    #[display("JSON error")]
    Serialize(serde_json::Error),

    /// Spec error.
    #[display("Spec error")]
    Spec(SpecError),
}
