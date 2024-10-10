use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Represents an alias
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alias {
    pub peer_id: String,
    pub alias: String,
}

impl Alias {
    pub fn new(peer_id: String, alias: String) -> Self {
        Self { peer_id, alias }
    }
}

impl Display for Alias {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.peer_id, self.alias)
    }
}

impl From<crate::codegen::sqlite::aliases::Model> for Alias {
    fn from(value: crate::codegen::sqlite::aliases::Model) -> Self {
        Alias::new(value.peer_id, value.alias)
    }
}
