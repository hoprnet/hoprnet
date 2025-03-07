use crate::routing::RoutingInfo;
use crate::shared_keys::SharedSecret;

/// Pseudonym used to identify the creator of a [`SURB`].
/// This allows indexing [`SURB`] and [`LocalSURBEntry`] at both parties.
pub trait Pseudonym: AsRef<[u8]> {}

/// Single Use Reply Block
pub struct SURB<K: AsRef<[u8]>> {
    /// Encryption key to use to encrypt the data for the SURB's creator.
    pub sender_key: [u8; 16],
    /// Sphinx routing header.
    pub header: RoutingInfo,
    /// ID of the first relayer.
    pub first_relayer: K,
}

/// Entry stored locally by the [`SURB`] creator to allow decryption
/// of received responses.
pub struct LocalSURBEntry {
    /// Encryption key the other party should use to encrypt the data for us.
    pub sender_key: [u8; 16],
    /// Shared keys for nodes along the return path.
    pub shared_keys: Vec<SharedSecret>,
}
