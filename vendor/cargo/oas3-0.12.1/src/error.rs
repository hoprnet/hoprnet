//! Error types

use std::io;

use derive_more::derive::{Display, Error, From};

use crate::spec::Error as SpecError;

/// Top-level errors.
#[derive(Debug, Display, Error, From)]
pub enum Error {
    #[display("I/O error")]
    Io(io::Error),

    #[display("YAML error")]
    Yaml(serde_yml::Error),

    #[display("JSON error")]
    Serialize(serde_json::Error),

    #[display("Spec error")]
    Spec(SpecError),
}
