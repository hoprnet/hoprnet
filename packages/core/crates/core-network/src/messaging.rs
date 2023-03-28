use blake2::{Digest, Blake2s256};
use rand::Rng;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use utils_types::traits::{AutoBinarySerializable, BinarySerializable};
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
        let mut ping = PingMessage::default();

        // TODO: use core-crypto random_fill once rebased
        OsRng.fill(&mut ping.nonce);
        Self::Ping(ping)
    }

    pub fn generate_pong_response(request: &ControlMessage) -> Result<Self> {
        match request {
            ControlMessage::Ping(ping) => {
                let mut pong = PingMessage::default();

                // TODO: move this to core-crypto once rebased
                let mut hasher = Blake2s256::new();
                hasher.update(&ping.nonce);
                let hash= hasher.finalize().to_vec();
                pong.nonce.copy_from_slice(&hash[0..PingMessage::PING_NONCE_SIZE]);

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

    pub fn get_ping_message(&self) -> Result<&PingMessage> {
        match self {
            ControlMessage::Ping(m) | ControlMessage::Pong(m) => Ok(m),
        }
    }
}

impl AutoBinarySerializable<'_> for ControlMessage { }

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct PingMessage {
    nonce: [u8; Self::PING_NONCE_SIZE],
}

impl PingMessage {
    const PING_NONCE_SIZE: usize = 16;

    /// Retrieves the challenge or response in this ping/pong message.
    pub fn nonce(&self) -> &[u8] {
        &self.nonce
    }
}

#[cfg(not(feature = "compat-ping"))]
impl AutoBinarySerializable<'_> for PingMessage { const SIZE: usize = PING_NONCE_SIZE; }

#[cfg(feature = "compat-ping")]
impl BinarySerializable<'_> for PingMessage {
    const SIZE: usize = Self::PING_NONCE_SIZE;

    // This implementation is backwards compatible with older HOPR versions

    fn deserialize(data: &[u8]) -> utils_types::errors::Result<Self> {
        let mut ret = PingMessage::default();
        let mut buf: Vec<u8> = data.into();
        ret.nonce.copy_from_slice(buf.drain(..).as_ref());
        Ok(ret)
    }

    fn serialize(&self) -> Box<[u8]> {
        let mut ret = Vec::<u8>::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.nonce);
        ret.into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {
    use utils_types::traits::BinarySerializable;
    use crate::messaging::ControlMessage;

    #[test]
    fn test_ping_pong_roundtrip() {
        let sent_req_s: Box<[u8]>;
        let sent_req: ControlMessage;
        {
            sent_req = ControlMessage::generate_ping_request();
            sent_req_s = sent_req.serialize();
        }

        let sent_resp_s: Box<[u8]>;
        {
            let recv_req = ControlMessage::deserialize(sent_req_s.as_ref()).unwrap();
            let send_resp = ControlMessage::generate_pong_response(&recv_req).unwrap();
            match &send_resp {
                ControlMessage::Ping(_) => panic!("invalid response version"),
                ControlMessage::Pong(m) => assert_eq!("4.5.6", m.format_version())
            }

            sent_resp_s = send_resp.serialize();
        }

        {
            let recv_resp = ControlMessage::deserialize(sent_resp_s.as_ref()).unwrap();
            assert!(ControlMessage::validate_pong_response(&sent_req, &recv_resp).is_ok());
            match recv_resp {
                ControlMessage::Ping(_) => panic!("must be pong"),
                ControlMessage::Pong(m) => assert_eq!("4.5.6", m.format_version())
            }
        }
    }
}