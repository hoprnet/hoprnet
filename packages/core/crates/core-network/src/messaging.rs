use blake2::{Digest, Blake2s256};
use rand::Rng;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use utils_types::traits::AutoBinarySerializable;
use crate::errors::NetworkingError::MessagingError;

use crate::errors::Result;

/// Implements the Control Message sub-protocol, which currently consists of Ping/Pong
/// messaging for the HOPR protocol.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ControlMessage {
    /// Ping challenge
    Ping(PingMessage),
    /// Pong response to a Ping
    Pong(PingMessage),
}

impl ControlMessage {
    pub fn generate_ping_request() -> Self {
        let mut ping = PingMessage{
            version: [0u8; VERSION_SIZE],
            nonce: [0u8; PING_NONCE_SIZE]
        };
        // TODO: use core-crypto random_fill once rebased
        OsRng.fill(&mut ping.nonce);
        Self::Ping(ping)
    }

    pub fn generate_pong_response(request: &ControlMessage) -> Result<Self> {
        match request {
            ControlMessage::Ping(ping) => {
                let mut pong = PingMessage {
                    version: [0u8; VERSION_SIZE],
                    nonce: [0u8; PING_NONCE_SIZE]
                };

                // TODO: move this to cory-crypto once rebased
                let mut hasher = Blake2s256::new();
                hasher.update(&ping.nonce);
                let hash= hasher.finalize().to_vec();
                pong.nonce.copy_from_slice(&hash[0..PING_NONCE_SIZE]);

                Ok(Self::Pong(pong))
            }
            ControlMessage::Pong(_) => Err(MessagingError("invalid ping message".into()))
        }
    }

    pub fn validate_pong_response(request: &ControlMessage, response: &ControlMessage) -> Result<()> {
        if let Self::Pong(expected_pong) = Self::generate_pong_response(request).unwrap() {
            match response {
                ControlMessage::Pong(received_pong) => {
                    match expected_pong.nonce.eq(&received_pong.nonce) {
                        true => Ok(()),
                        false => Err(MessagingError("pong response does not match the challenge".into()))
                    }
                }
                ControlMessage::Ping(_) => Err(MessagingError("invalid pong response".into()))
            }
        } else {
            Err(MessagingError("request is not a valid ping message".into()))
        }
    }
}

impl AutoBinarySerializable<'_> for ControlMessage { }

const VERSION_SIZE: usize = 3;
const PING_NONCE_SIZE: usize = 16;

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

#[cfg(test)]
mod tests {
    use crate::messaging::ControlMessage;

    #[test]
    fn test_ping_pong_rountrip() {
        let sent_req = ControlMessage::generate_ping_request();
        let recv_req = ControlMessage::generate_pong_response(&sent_req).unwrap();
        assert!(ControlMessage::validate_pong_response(&sent_req, &recv_req).is_ok());
    }
}