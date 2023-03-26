use serde::{Deserialize, Serialize};
use utils_types::traits::AutoBinarySerializable;

/// Represents low-level control message
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ControlMessage {
    /// Ping challenge
    Ping(PingMessage),
    /// Pong response to a Ping
    Pong(PingMessage),
}

impl AutoBinarySerializable<'_> for ControlMessage { }

const VERSION_SIZE: usize = 3;
const PING_NONCE_SIZE: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct PingMessage {
    version: [u8; VERSION_SIZE],
    nonce: [u8; PING_NONCE_SIZE]
}

impl PingMessage {
    /// Retrieves the HOPRd version string formatted as semver
    pub fn format_version(&self) -> String {
        format!("{}.{}.{}", self.version[0], self.version[1], self.version[2])
    }
}

impl AutoBinarySerializable<'_> for PingMessage { const SIZE: usize = VERSION_SIZE + PING_NONCE_SIZE; }