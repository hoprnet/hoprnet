use crate::errors::GeneralError::ParseError;
use crate::errors::Result;
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// A generic type that can be converted to a hexadecimal string.
pub trait ToHex {
    /// Hexadecimal representation of this type.
    fn to_hex(&self) -> String;

    /// Tries to parse the type from the hexadecimal representation.
    fn from_hex(str: &str) -> Result<Self>
    where
        Self: Sized;
}

/// A type that can be serialized and deserialized to a binary form.
/// Implementing this trait automatically implements ToHex trait
/// which then uses the serialize method.
pub trait BinarySerializable: Sized {
    /// Minimum size of this type in bytes.
    const SIZE: usize;

    /// Deserializes the type from a binary blob.
    fn from_bytes(data: &[u8]) -> Result<Self>;

    /// Serializes the type into a fixed size binary blob.
    fn to_bytes(&self) -> Box<[u8]>;
}

/// Type implementing this trait has automatic binary serialization/deserialization capability
/// using the default binary format, which is currently `bincode`.
pub trait AutoBinarySerializable: Serialize + for<'a> Deserialize<'a> {
    /// Minimum size of an automatically serialized type in bytes is 1.
    const SIZE: usize = 1;
}

impl<T> BinarySerializable for T
where
    T: AutoBinarySerializable,
{
    const SIZE: usize = Self::SIZE;

    /// Deserializes the type from a binary blob.
    fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data).map_err(|_| ParseError)
    }

    /// Serializes the type into a fixed size binary blob.
    fn to_bytes(&self) -> Box<[u8]> {
        bincode::serialize(&self).unwrap().into_boxed_slice()
    }
}

impl<T> ToHex for T
where
    T: BinarySerializable,
{
    fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(&self.to_bytes()))
    }

    fn from_hex(str: &str) -> Result<Self> {
        let data = if &str[..2] == "0x" || &str[..2] == "0X" {
            &str[2..]
        } else {
            str
        };

        hex::decode(data)
            .map_err(|_| ParseError)
            .and_then(|bytes| T::from_bytes(&bytes))
    }
}

/// A generic type which can be equivalently and completely represented by a PeerID
pub trait PeerIdLike: Sized {
    /// Creates type from a PeerID representation
    fn from_peerid(peer_id: &PeerId) -> Result<Self>;

    /// Converts type to a PeerID representation
    fn to_peerid(&self) -> PeerId;

    /// Creates instance from base-58 PeerId representation
    fn from_peerid_str(peer_id: &str) -> Result<Self> {
        Self::from_peerid(&PeerId::from_str(peer_id).map_err(|_| ParseError)?)
    }

    /// Outputs base-58 PeerId representation
    fn to_peerid_str(&self) -> String {
        self.to_peerid().to_base58()
    }
}
