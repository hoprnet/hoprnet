//! Transport packet primitives for the HOPR protocol transport layer.

use std::fmt::Debug;

use libp2p_identity::PeerId;

use hopr_crypto_packet::prelude::HoprPacket;
use hopr_internal_types::protocol::Acknowledgement;

#[derive(Debug, Clone, PartialEq)]
pub struct DataBytes(Box<[u8]>);

impl AsRef<[u8]> for DataBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryFrom<&[u8]> for DataBytes {
    type Error = std::io::Error;

    fn try_from(data: &[u8]) -> std::result::Result<Self, Self::Error> {
        if data.len() == HoprPacket::PAYLOAD_SIZE {
            Ok(DataBytes(Box::from(data)))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Payload too small",
            ))
        }
    }
}

impl TryFrom<Box<[u8]>> for DataBytes {
    type Error = std::io::Error;

    fn try_from(data: Box<[u8]>) -> std::result::Result<Self, Self::Error> {
        if data.len() == HoprPacket::PAYLOAD_SIZE {
            Ok(DataBytes(data))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Payload too small",
            ))
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Payload {
    Msg(DataBytes),
    Ack(Acknowledgement),
}

impl Debug for Payload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Payload::Msg(_data) => write!(f, "Msg"),
            Payload::Ack(_ack) => write!(f, "Ack"),
        }
    }
}

impl AsRef<[u8]> for Payload {
    fn as_ref(&self) -> &[u8] {
        match self {
            Payload::Msg(data) => data.0.as_ref(),
            Payload::Ack(ack) => ack.as_ref(),
        }
    }
}

impl TryFrom<Box<[u8]>> for Payload {
    type Error = std::io::Error;

    fn try_from(data: Box<[u8]>) -> std::result::Result<Self, Self::Error> {
        Ok(Payload::Msg(DataBytes::try_from(data)?))
    }
}

impl From<DataBytes> for Payload {
    fn from(data: DataBytes) -> Self {
        Payload::Msg(data)
    }
}

impl From<Acknowledgement> for Payload {
    fn from(ack: Acknowledgement) -> Self {
        Payload::Ack(ack)
    }
}

impl Payload {
    /// TODO: Possibyl remove
    pub const SIZE: usize = HoprPacket::PAYLOAD_SIZE + 1;
}

pub struct Header {
    peer: PeerId,
}

pub struct Packet {
    header: Header,
    payload: Payload,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_bytes_cannot_be_shorter_than_expected_bytes_count() {
        let too_short = [1u8, 2u8, 3u8, 4u8];

        assert!(DataBytes::try_from(Box::from(too_short.as_slice())).is_err());
    }
    #[test]
    fn data_bytes_cannot_be_longer_than_expected_bytes_count() {
        let too_long = [1u8; HoprPacket::PAYLOAD_SIZE + 10];

        assert!(DataBytes::try_from(Box::from(too_long.as_slice())).is_err());
    }

    #[test]
    fn data_bytes_of_expected_length_can_be_built() {
        let just_right = [1u8; HoprPacket::PAYLOAD_SIZE];

        assert!(DataBytes::try_from(Box::from(just_right.as_slice())).is_ok());
    }
}
