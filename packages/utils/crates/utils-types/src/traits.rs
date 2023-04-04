use crate::errors::{GeneralError::ParseError, Result};
use libp2p_identity::PeerId;
use std::str::FromStr;

// NOTE on wasm_bindgen: since #[wasm_bindgen] attributes cannot be used
// on trait impl blocks, the trait inherited methods need to be re-implemented
// in a #[wasm_bindgen] annotated blocks if they need to be exposed to TypeScript.
// Therefore, use the traits reasonably for now to avoid too much code duplication.

/// A generic type that can be converted to a hexadecimal string.
pub trait ToHex {
    /// Hexadecimal representation of this type.
    fn to_hex(&self) -> String;
}

/// A type that can be serialized and deserialized to a binary
/// form with a fixed size.
/// Implementing this trait automatically implements ToHex trait
/// which then uses the serialize method.
pub trait BinarySerializable: Sized {
    /// Fixed serialized size of this type in bytes.
    const SIZE: usize;

    /// Deserializes the type from a binary blob.
    fn deserialize(data: &[u8]) -> Result<Self>;

    /// Serializes the type into a fixed size binary blob.
    fn serialize(&self) -> Box<[u8]>;
}

impl<T> ToHex for T
where
    T: BinarySerializable,
{
    fn to_hex(&self) -> String {
        hex::encode(&self.serialize())
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

    /// Convenience method that creates a new instance of this type from a random PeerId.
    fn generate_from_random_peerid() -> Self {
        Self::from_peerid(&PeerId::random()).unwrap()
    }
}
