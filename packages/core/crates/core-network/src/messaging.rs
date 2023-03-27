use serde::{Deserialize, Serialize};
use utils_types::traits::AutoBinarySerializable;
use crate::errors::NetworkingError::MessagingError;

use crate::errors::Result;

/// Represents low-level control message sub-protocol
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ControlMessage {
    /// Empty message
    Good,
    /// Ping challenge
    Ping(PingMessage),
    /// Pong response to a Ping
    Pong(PingMessage),
}

impl AutoBinarySerializable<'_> for ControlMessage { }

impl ControlMessage {
    pub fn generate_ping_request() -> Self {
        Self::Ping(PingMessage{
            version: [0u8; VERSION_SIZE],
            nonce: [0u8; PING_NONCE_SIZE]
        })
    }

    pub fn try_response(msg: &ControlMessage) -> Result<Self> {
        match msg {
            ControlMessage::Ping(_) => {
                Ok(Self::Pong(PingMessage {
                    version: [0u8; VERSION_SIZE],
                    nonce: [0u8; PING_NONCE_SIZE]
                }))
            }
            _ => Err(MessagingError(format!("cannot respond to control-message {:?}", msg)))
        }
    }
}

const VERSION_SIZE: usize = 3;
const PING_NONCE_SIZE: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct PingMessage {
    version: [u8; VERSION_SIZE],
    nonce: [u8; PING_NONCE_SIZE]
}

impl PingMessage {
    pub fn nonce(&self) -> &[u8] {
        &self.nonce
    }

    /// Retrieves the HOPRd version string formatted as semver
    pub fn format_version(&self) -> String {
        format!("{}.{}.{}", self.version[0], self.version[1], self.version[2])
    }
}

impl AutoBinarySerializable<'_> for PingMessage { const SIZE: usize = VERSION_SIZE + PING_NONCE_SIZE; }