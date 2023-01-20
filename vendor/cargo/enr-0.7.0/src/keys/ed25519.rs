use super::{
    ed25519_dalek::{self as ed25519, Signer as _, Verifier as _},
    EnrKey, EnrKeyUnambiguous, EnrPublicKey, SigningError,
};
use crate::Key;
use bytes::Bytes;
use rlp::DecoderError;
use std::{collections::BTreeMap, convert::TryFrom};

/// The ENR key that stores the public key in the ENR record.
pub const ENR_KEY: &str = "ed25519";

impl EnrKey for ed25519::Keypair {
    type PublicKey = ed25519::PublicKey;

    /// Performs ENR-specific signing.
    ///
    /// Using `ed25519` keys do not currently follow the `v4` identity scheme, which dictates
    /// `secp256k1` keys should be used.
    fn sign_v4(&self, msg: &[u8]) -> Result<Vec<u8>, SigningError> {
        Ok(self.sign(msg).to_bytes().to_vec())
    }

    /// Returns the public key associated with the private key.
    fn public(&self) -> Self::PublicKey {
        self.public
    }

    /// Decodes the raw bytes of an ENR's content into a public key if possible.
    fn enr_to_public(content: &BTreeMap<Key, Bytes>) -> Result<Self::PublicKey, DecoderError> {
        let pubkey_bytes = content
            .get(ENR_KEY.as_bytes())
            .ok_or(DecoderError::Custom("Unknown signature"))?;

        // Decode the RLP
        let pubkey_bytes = rlp::Rlp::new(pubkey_bytes).data()?;

        Self::decode_public(pubkey_bytes)
    }
}

impl EnrKeyUnambiguous for ed25519::Keypair {
    fn decode_public(bytes: &[u8]) -> Result<Self::PublicKey, DecoderError> {
        ed25519::PublicKey::from_bytes(bytes)
            .map_err(|_| DecoderError::Custom("Invalid ed25519 Signature"))
    }
}

impl EnrPublicKey for ed25519::PublicKey {
    type Raw = [u8; ed25519::PUBLIC_KEY_LENGTH];
    type RawUncompressed = [u8; ed25519::PUBLIC_KEY_LENGTH];

    /// Verify a raw message, given a public key for the v4 identity scheme.
    fn verify_v4(&self, msg: &[u8], sig: &[u8]) -> bool {
        ed25519::Signature::try_from(sig)
            .and_then(|s| self.verify(msg, &s))
            .is_ok()
    }

    /// Encodes the public key into compressed form, if possible.
    fn encode(&self) -> Self::Raw {
        self.to_bytes()
    }

    /// Encodes the public key in uncompressed form. This is the same as the compressed form for
    /// ed25519 keys
    fn encode_uncompressed(&self) -> Self::RawUncompressed {
        self.encode()
    }

    /// Generates the ENR public key string associated with the ed25519 key type.
    fn enr_key(&self) -> Key {
        ENR_KEY.into()
    }
}
