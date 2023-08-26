use crate::errors::NetworkingError::MessagingError;
use core_crypto::derivation::derive_ping_pong;
use serde::{Deserialize, Serialize};

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
    /// Creates ping challenge message
    pub fn generate_ping_request() -> Self {
        let mut ping = PingMessage::default();
        ping.nonce.copy_from_slice(&derive_ping_pong(None));

        Self::Ping(ping)
    }

    /// Given the received ping challenge, generates an appropriate response to the challenge.
    pub fn generate_pong_response(request: &ControlMessage) -> Result<Self> {
        match request {
            ControlMessage::Ping(ping) => {
                let mut pong = PingMessage::default();
                pong.nonce.copy_from_slice(&derive_ping_pong(Some(ping.nonce())));
                Ok(Self::Pong(pong))
            }
            ControlMessage::Pong(_) => Err(MessagingError("invalid ping message".into())),
        }
    }

    /// Given the original ping challenge message, verifies that the received pong response is valid
    /// according to the challenge.
    pub fn validate_pong_response(request: &ControlMessage, response: &ControlMessage) -> Result<()> {
        if let Self::Pong(expected_pong) = Self::generate_pong_response(request).unwrap() {
            match response {
                ControlMessage::Pong(received_pong) => match expected_pong.nonce.eq(&received_pong.nonce) {
                    true => Ok(()),
                    false => Err(MessagingError("pong response does not match the challenge".into())),
                },
                ControlMessage::Ping(_) => Err(MessagingError("invalid pong response".into())),
            }
        } else {
            Err(MessagingError("request is not a valid ping message".into()))
        }
    }

    /// Convenience method to de-structure the ping message payload, if this
    /// instance is a Ping or Pong.
    pub fn get_ping_message(&self) -> Result<&PingMessage> {
        match self {
            ControlMessage::Ping(m) | ControlMessage::Pong(m) => Ok(m),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct PingMessage {
    nonce: [u8; core_crypto::parameters::PING_PONG_NONCE_SIZE],
}

impl PingMessage {
    /// Gets the challenge or response in this ping/pong message.
    pub fn nonce(&self) -> &[u8] {
        &self.nonce
    }
}

#[cfg(not(feature = "compat-ping"))]
impl utils_types::traits::AutoBinarySerializable for PingMessage {}

#[cfg(feature = "compat-ping")]
impl utils_types::traits::BinarySerializable for PingMessage {
    const SIZE: usize = core_crypto::parameters::PING_PONG_NONCE_SIZE;

    // This implementation is backwards compatible with older HOPR versions

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        if data.len() >= Self::SIZE {
            let mut ret = PingMessage::default();
            let mut buf = data.to_vec();
            ret.nonce.copy_from_slice(buf.drain(0..Self::SIZE).as_ref());
            Ok(ret)
        } else {
            Err(utils_types::errors::GeneralError::ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::<u8>::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.nonce);
        ret.into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {
    use crate::messaging::ControlMessage::{Ping, Pong};
    use crate::messaging::{ControlMessage, PingMessage};
    use utils_types::traits::BinarySerializable;

    #[test]
    fn test_ping_pong_roundtrip() {
        // ping initiator
        let sent_req_s: Box<[u8]>;
        let sent_req: ControlMessage;
        {
            sent_req = ControlMessage::generate_ping_request();
            sent_req_s = sent_req.get_ping_message().unwrap().to_bytes();
        }

        // pong responder
        let sent_resp_s: Box<[u8]>;
        {
            let recv_req = PingMessage::from_bytes(sent_req_s.as_ref()).unwrap();
            let send_resp = ControlMessage::generate_pong_response(&Ping(recv_req)).unwrap();
            sent_resp_s = send_resp.get_ping_message().unwrap().to_bytes();
        }

        // verify pong
        {
            let recv_resp = PingMessage::from_bytes(sent_resp_s.as_ref()).unwrap();
            assert!(ControlMessage::validate_pong_response(&sent_req, &Pong(recv_resp)).is_ok());
        }
    }
}
