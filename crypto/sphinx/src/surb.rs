use hopr_crypto_types::prelude::SecretKey16;
use hopr_primitive_types::prelude::{BytesRepresentable, KeyIdent};
use crate::routing::{RoutingInfo, SphinxHeaderSpec};
use crate::shared_keys::SharedSecret;

/// Pseudonym used to identify the creator of a [`SURB`].
/// This allows indexing [`SURB`] and [`LocalSURBEntry`] at both parties.
pub trait Pseudonym: AsRef<[u8]> {}

/// Single Use Reply Block
pub struct SURB<H: SphinxHeaderSpec> {
    /// Encryption key to use to encrypt the data for the SURB's creator.
    pub sender_key: SecretKey16,
    /// Sphinx routing header.
    pub header: RoutingInfo<H>,
    /// ID of the first relayer.
    pub first_relayer: KeyIdent,
}

impl<H: SphinxHeaderSpec> SURB<H> {
    pub const SIZE: usize = SecretKey16::LENGTH + RoutingInfo::<H>::SIZE + KeyIdent::SIZE;
}

/// Entry stored locally by the [`SURB`] creator to allow decryption
/// of received responses.
pub struct LocalSURBEntry {
    /// Encryption key the other party should use to encrypt the data for us.
    pub sender_key: SecretKey16,
    /// Shared keys for nodes along the return path.
    pub shared_keys: Vec<SharedSecret>,
}
