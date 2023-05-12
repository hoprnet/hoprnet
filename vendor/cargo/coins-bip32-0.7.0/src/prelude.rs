pub use crate::derived::{DerivedKey, DerivedPubkey, DerivedXPriv, DerivedXPub};
pub use crate::enc::{MainnetEncoder, TestnetEncoder, XKeyEncoder};
pub use crate::path::KeyDerivation;
pub use crate::primitives::*;
pub use crate::xkeys::{Parent, XPriv, XPub};
pub use crate::Bip32Error;

#[cfg(any(feature = "mainnet", feature = "testnet"))]
pub use crate::defaults::*;

/// Re-exported signer traits
pub use k256::ecdsa::{
    recoverable::Signature as RecoverableSignature,
    signature::{DigestSigner, DigestVerifier, Signature as SigTrait},
    Signature, SigningKey, VerifyingKey,
};

/// shortcut for easy usage
pub fn fingerprint_of(k: &k256::ecdsa::VerifyingKey) -> KeyFingerprint {
    use coins_core::hashes::Digest;
    let digest = coins_core::hashes::Hash160::digest(&k.to_bytes());
    let mut fingerprint = [0u8; 4];
    fingerprint.copy_from_slice(&digest[..4]);
    fingerprint.into()
}
