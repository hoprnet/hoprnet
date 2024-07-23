use serde::{Deserialize, Serialize};

/// Represents an alias
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AliasEntry {
    pub peer_id: String,
    pub alias: String,
}

impl AliasEntry {
    pub fn new(peer_id: String, alias: String) -> Self {
        Self { peer_id, alias }
    }
}
