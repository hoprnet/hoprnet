use hopr_primitive_types::prelude::{GeneralError::ParseError, *};
use k256::ecdsa::{
    self, RecoveryId, Signature as ECDSASignature, SigningKey, VerifyingKey,
    signature::{Verifier, hazmat::PrehashVerifier},
};
use sha2::Sha512;
use tracing::warn;

use crate::prelude::*;

/// Represents an ECDSA signature based on the secp256k1 curve with a recoverable public key.
///
/// This signature encodes the 2-bit recovery information into the
/// uppermost bits from MSB of the `S` value, which are never used by this ECDSA
/// instantiation over secp256k1.
///
/// The instance holds the byte array consisting of `R` and `S` values with the recovery bit
/// already embedded in `S`.
///
/// See [EIP-2098](https://eips.ethereum.org/EIPS/eip-2098) for details.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Signature(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; Self::SIZE]);

impl Signature {
    pub fn new(raw_bytes: &[u8], recovery: u8) -> Signature {
        assert!(recovery <= 1, "invalid recovery bit");

        let mut ret = Self([0u8; Self::SIZE]);
        ret.0.copy_from_slice(raw_bytes);

        // Embed the recovery bit into the S value
        ret.0[Self::SIZE / 2] &= 0x7f;
        ret.0[Self::SIZE / 2] |= recovery << 7;

        ret
    }

    fn sign<S>(data: &[u8], private_key: &[u8], signing_method: S) -> Signature
    where
        S: FnOnce(&SigningKey, &[u8]) -> ecdsa::signature::Result<(ECDSASignature, RecoveryId)>,
    {
        let key = SigningKey::from_bytes(private_key.into()).expect("invalid signing key");
        let (sig, rec) = signing_method(&key, data).expect("signing failed");

        Self::new(&sig.to_vec(), rec.to_byte())
    }

    /// Signs the given message using the chain private key.
    pub fn sign_message(message: &[u8], chain_keypair: &ChainKeypair) -> Signature {
        Self::sign(
            message,
            chain_keypair.secret().as_ref(),
            |k: &SigningKey, data: &[u8]| k.sign_recoverable(data),
        )
    }

    /// Signs the given hash using the raw private key.
    pub fn sign_hash(hash: &[u8], chain_keypair: &ChainKeypair) -> Signature {
        Self::sign(hash, chain_keypair.secret().as_ref(), |k: &SigningKey, data: &[u8]| {
            k.sign_prehash_recoverable(data)
        })
    }

    fn verify<V>(&self, message: &[u8], public_key: &[u8], verifier: V) -> bool
    where
        V: FnOnce(&VerifyingKey, &[u8], &ECDSASignature) -> ecdsa::signature::Result<()>,
    {
        let pub_key = VerifyingKey::from_sec1_bytes(public_key).expect("invalid public key");

        if let Ok(signature) = ECDSASignature::try_from(self.raw_signature().0.as_ref()) {
            verifier(&pub_key, message, &signature).is_ok()
        } else {
            warn!("un-parseable signature encountered");
            false
        }
    }

    /// Verifies this signature against the given message and a public key object
    pub fn verify_message(&self, message: &[u8], public_key: &PublicKey) -> bool {
        self.verify(message, &public_key.to_uncompressed_bytes(), |k, msg, sgn| {
            k.verify(msg, sgn)
        })
    }

    /// Verifies this signature against the given hash and a public key object
    pub fn verify_hash(&self, hash: &[u8], public_key: &PublicKey) -> bool {
        self.verify(hash, &public_key.to_uncompressed_bytes(), |k, msg, sgn| {
            k.verify_prehash(msg, sgn)
        })
    }

    /// Returns the raw signature, without the encoded public key recovery bit and
    /// the recovery bit as a separate value.
    pub fn raw_signature(&self) -> ([u8; Self::SIZE], u8) {
        let mut raw_sig = self.0;
        let recovery: u8 = (raw_sig[Self::SIZE / 2] & 0x80 != 0).into();
        raw_sig[Self::SIZE / 2] &= 0x7f;
        (raw_sig, recovery)
    }

    fn recover<R>(&self, msg: &[u8], recovery_method: R) -> crate::errors::Result<PublicKey>
    where
        R: FnOnce(&[u8], &ECDSASignature, RecoveryId) -> ecdsa::signature::Result<VerifyingKey>,
    {
        let (sig, v) = self.raw_signature();

        let recid = RecoveryId::try_from(v).map_err(|_| ParseError("Signature".into()))?;

        let signature = ECDSASignature::from_bytes(&sig.into()).map_err(|_| ParseError("Signature".into()))?;

        let recovered_key = *recovery_method(msg, &signature, recid)
            .map_err(|_| CryptoError::CalculationError)?
            .as_affine();

        // Verify that it is a valid public key
        recovered_key.try_into()
    }

    #[inline]
    pub fn recover_from_msg(&self, msg: &[u8]) -> crate::errors::Result<PublicKey> {
        self.recover(msg, VerifyingKey::recover_from_msg)
    }

    #[inline]
    pub fn recover_from_hash(&self, hash: &[u8]) -> crate::errors::Result<PublicKey> {
        self.recover(hash, VerifyingKey::recover_from_prehash)
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for Signature {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        Ok(Self(value.try_into().map_err(|_| ParseError("Signature".into()))?))
    }
}

impl BytesRepresentable for Signature {
    const SIZE: usize = 64;
}

impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for Signature {}

/// Represents an EdDSA signature using the Ed25519 Edwards curve.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OffchainSignature(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; Self::SIZE]);

impl OffchainSignature {
    /// Sign the given message using the [OffchainKeypair].
    pub fn sign_message(msg: &[u8], signing_keypair: &OffchainKeypair) -> Self {
        // Expand the SK from the given keypair
        let expanded_sk = ed25519_dalek::hazmat::ExpandedSecretKey::from(
            &ed25519_dalek::SecretKey::try_from(signing_keypair.secret().as_ref()).expect("invalid private key"),
        );

        // Get the verifying key from the SAME keypair, avoiding Double Public Key Signing Function Oracle Attack on
        // Ed25519 See https://github.com/MystenLabs/ed25519-unsafe-libs for details
        let verifying = ed25519_dalek::VerifyingKey::from(signing_keypair.public().edwards);

        ed25519_dalek::hazmat::raw_sign::<Sha512>(&expanded_sk, msg, &verifying).into()
    }

    /// Verify this signature of the given message and [OffchainPublicKey].
    pub fn verify_message(&self, msg: &[u8], public_key: &OffchainPublicKey) -> bool {
        let sgn = ed25519_dalek::Signature::from_slice(&self.0).expect("corrupted OffchainSignature");
        let pk = ed25519_dalek::VerifyingKey::from(public_key.edwards);
        pk.verify_strict(msg, &sgn).is_ok()
    }
}

impl AsRef<[u8]> for OffchainSignature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for OffchainSignature {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(ed25519_dalek::Signature::from_slice(value)
            .map_err(|_| ParseError("OffchainSignature".into()))?
            .into())
    }
}

impl BytesRepresentable for OffchainSignature {
    /// Size of the EdDSA signature using Ed25519.
    const SIZE: usize = ed25519_dalek::Signature::BYTE_SIZE;
}

impl From<ed25519_dalek::Signature> for OffchainSignature {
    fn from(value: ed25519_dalek::Signature) -> Self {
        let mut ret = Self([0u8; Self::SIZE]);
        ret.0.copy_from_slice(value.to_bytes().as_ref());
        ret
    }
}

impl TryFrom<([u8; 32], [u8; 32])> for OffchainSignature {
    type Error = GeneralError;

    fn try_from(value: ([u8; 32], [u8; 32])) -> std::result::Result<Self, Self::Error> {
        Ok(ed25519_dalek::Signature::from_components(value.0, value.1).into())
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::Signer;
    use hex_literal::hex;

    use super::*;
    use crate::keypairs::Keypair;

    const PRIVATE_KEY: [u8; 32] = hex!("e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8");

    #[test]
    fn test_signature_serialize() -> anyhow::Result<()> {
        let msg = b"test000000";
        let kp = ChainKeypair::from_secret(&PRIVATE_KEY)?;
        let sgn = Signature::sign_message(msg, &kp);

        let deserialized = Signature::try_from(sgn.as_ref())?;
        assert_eq!(sgn, deserialized, "signatures don't match");

        Ok(())
    }

    #[test]
    fn test_sign_and_recover() -> anyhow::Result<()> {
        let msg = hex!("eff80b9f035b1d369c6a60f362ac7c8b8c3b61b76d151d1be535145ccaa3e83e");

        let kp = ChainKeypair::from_secret(&PRIVATE_KEY)?;

        let signature1 = Signature::sign_message(&msg, &kp);
        let signature2 = Signature::sign_hash(&msg, &kp);

        let pub_key1 = PublicKey::from_privkey(&PRIVATE_KEY)?;
        let pub_key2 = signature1.recover_from_msg(&msg)?;
        let pub_key3 = signature2.recover_from_hash(&msg)?;

        assert_eq!(pub_key1, *kp.public());
        assert_eq!(pub_key1, pub_key2, "recovered public key does not match");
        assert_eq!(pub_key1, pub_key3, "recovered public key does not match");

        assert!(
            signature1.verify_message(&msg, &pub_key1),
            "signature 1 verification failed with pub key 1"
        );
        assert!(
            signature1.verify_message(&msg, &pub_key2),
            "signature 1 verification failed with pub key 2"
        );
        assert!(
            signature1.verify_message(&msg, &pub_key3),
            "signature 1 verification failed with pub key 3"
        );

        assert!(
            signature2.verify_hash(&msg, &pub_key1),
            "signature 2 verification failed with pub key 1"
        );
        assert!(
            signature2.verify_hash(&msg, &pub_key2),
            "signature 2 verification failed with pub key 2"
        );
        assert!(
            signature2.verify_hash(&msg, &pub_key3),
            "signature 2 verification failed with pub key 3"
        );

        Ok(())
    }

    #[test]
    fn test_offchain_signature_signing() -> anyhow::Result<()> {
        let msg = b"test12345";
        let keypair = OffchainKeypair::from_secret(&PRIVATE_KEY)?;

        let key = ed25519_dalek::SecretKey::try_from(PRIVATE_KEY)?;
        let kp = ed25519_dalek::SigningKey::from_bytes(&key);
        let pk = ed25519_dalek::VerifyingKey::from(&kp);

        let sgn = kp.sign(msg);
        assert!(pk.verify_strict(msg, &sgn).is_ok(), "blomp");

        let sgn_1 = OffchainSignature::sign_message(msg, &keypair);
        let sgn_2 = OffchainSignature::try_from(sgn_1.as_ref())?;

        assert!(
            sgn_1.verify_message(msg, keypair.public()),
            "cannot verify message via sig 1"
        );
        assert!(
            sgn_2.verify_message(msg, keypair.public()),
            "cannot verify message via sig 2"
        );
        assert_eq!(sgn_1, sgn_2, "signatures must be equal");

        Ok(())
    }

    #[test]
    fn test_offchain_signature() -> anyhow::Result<()> {
        let msg = b"test12345";
        let keypair = OffchainKeypair::from_secret(&PRIVATE_KEY)?;

        let key = ed25519_dalek::SecretKey::try_from(PRIVATE_KEY)?;
        let kp = ed25519_dalek::SigningKey::from_bytes(&key);
        let pk = ed25519_dalek::VerifyingKey::from(&kp);

        let sgn = kp.sign(msg);
        assert!(pk.verify_strict(msg, &sgn).is_ok(), "blomp");

        let sgn_1 = OffchainSignature::sign_message(msg, &keypair);
        let sgn_2 = OffchainSignature::try_from(sgn_1.as_ref())?;

        assert!(
            sgn_1.verify_message(msg, keypair.public()),
            "cannot verify message via sig 1"
        );
        assert!(
            sgn_2.verify_message(msg, keypair.public()),
            "cannot verify message via sig 2"
        );
        assert_eq!(sgn_1, sgn_2, "signatures must be equal");

        let keypair = OffchainKeypair::from_secret(&PRIVATE_KEY)?;
        let sig = OffchainSignature::sign_message("my test msg".as_bytes(), &keypair);
        assert!(sig.verify_message("my test msg".as_bytes(), keypair.public()));

        Ok(())
    }
}
