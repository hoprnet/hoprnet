use std::marker::PhantomData;

use hopr_primitive_types::prelude::{GeneralError::ParseError, *};
use sha2::Sha512;

use crate::prelude::*;

const ECDSA_SIGNATURE_SIZE: usize = 64;

type RawSignature = ([u8; ECDSA_SIGNATURE_SIZE], u8);

/// Trait for ECDSA signature engines.
pub trait EcdsaEngine {
    /// Sign the given `hash` with the private key from the `chain_keypair`.
    fn sign_hash(hash: &Hash, chain_keypair: &ChainKeypair) -> Result<RawSignature, CryptoError>;
    /// Verify the given `signature` against the `hash` and the `public_key`.
    fn verify_hash(signature: &RawSignature, hash: &Hash, public_key: &PublicKey) -> Result<bool, CryptoError>;
    /// Recover the signer public key from the `signature` and the `hash`.
    fn recover_from_hash(signature: &RawSignature, hash: &Hash) -> Result<PublicKey, CryptoError>;
}

/// ECDSA signing engine based on the pure Rust [`k256`](https://docs.rs/k256/latest/k256/) crate.
///
/// This is usually slower than [`NativeEcdsaSigningEngine`], but pure Rust-based.
#[cfg(feature = "rust-ecdsa")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct K256EcdsaSigningEngine;
#[cfg(feature = "rust-ecdsa")]
impl EcdsaEngine for K256EcdsaSigningEngine {
    #[inline]
    fn sign_hash(hash: &Hash, chain_keypair: &ChainKeypair) -> Result<RawSignature, CryptoError> {
        let key = k256::ecdsa::SigningKey::from_bytes(chain_keypair.secret().as_ref().into())
            .map_err(|_| CryptoError::InvalidInputValue("chain_keypair"))?;
        let (sig, rec) = key
            .sign_prehash_recoverable(hash.as_ref())
            .map_err(|_| CryptoError::CalculationError)?;

        Ok((sig.to_bytes().into(), rec.to_byte()))
    }

    #[inline]
    fn verify_hash(signature: &RawSignature, hash: &Hash, public_key: &PublicKey) -> Result<bool, CryptoError> {
        use k256::ecdsa::signature::hazmat::PrehashVerifier;

        let pub_key = k256::ecdsa::VerifyingKey::from_sec1_bytes(&public_key.to_uncompressed_bytes())
            .map_err(|_| CryptoError::InvalidInputValue("public key"))?;

        if let Ok(signature) = k256::ecdsa::Signature::try_from(signature.0.as_ref()) {
            Ok(pub_key.verify_prehash(hash.as_ref(), &signature).is_ok())
        } else {
            Err(CryptoError::InvalidInputValue("signature"))
        }
    }

    #[inline]
    fn recover_from_hash(signature: &RawSignature, hash: &Hash) -> Result<PublicKey, CryptoError> {
        let sig = k256::ecdsa::Signature::from_bytes(&signature.0.into())
            .map_err(|_| CryptoError::InvalidInputValue("signature.sig"))?;

        let recid = k256::ecdsa::RecoveryId::try_from(signature.1)
            .map_err(|_| CryptoError::InvalidInputValue("signature.recid"))?;

        let recovered_key = k256::ecdsa::VerifyingKey::recover_from_prehash(hash.as_ref(), &sig, recid)
            .map_err(|_| CryptoError::SignatureVerification)?;

        (*recovered_key.as_affine()).try_into()
    }
}

/// ECDSA signing engine based on the fast [`secp256k1`](https://docs.rs/secp256k1/latest/secp256k1/) crate.
///
/// The crate uses bindings to the Bitcoin secp256k1 C library.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeEcdsaSigningEngine;

impl EcdsaEngine for NativeEcdsaSigningEngine {
    fn sign_hash(hash: &Hash, chain_keypair: &ChainKeypair) -> Result<RawSignature, CryptoError> {
        let sk_arr: &generic_array::GenericArray<u8, typenum::U32> = chain_keypair.secret().into();
        let sk = secp256k1::SecretKey::from_byte_array(sk_arr.into_array())
            .map_err(|_| CryptoError::InvalidInputValue("chain_keypair"))?;

        let sig =
            secp256k1::global::SECP256K1.sign_ecdsa_recoverable(secp256k1::Message::from_digest(hash.into()), &sk);
        let (recid, sig) = sig.serialize_compact();
        Ok((sig, i32::from(recid) as u8))
    }

    fn verify_hash(signature: &RawSignature, hash: &Hash, public_key: &PublicKey) -> Result<bool, CryptoError> {
        let pk = secp256k1::PublicKey::from_slice(&public_key.to_uncompressed_bytes())
            .map_err(|_| CryptoError::InvalidInputValue("public key"))?;

        let sig = secp256k1::ecdsa::Signature::from_compact(&signature.0)
            .map_err(|_| CryptoError::InvalidInputValue("signature"))?;

        Ok(secp256k1::global::SECP256K1
            .verify_ecdsa(secp256k1::Message::from_digest(hash.into()), &sig, &pk)
            .is_ok())
    }

    fn recover_from_hash(signature: &RawSignature, hash: &Hash) -> Result<PublicKey, CryptoError> {
        let sig = secp256k1::ecdsa::RecoverableSignature::from_compact(
            &signature.0,
            secp256k1::ecdsa::RecoveryId::from_u8_masked(signature.1),
        )
        .map_err(|_| CryptoError::InvalidInputValue("signature"))?;

        let pk = secp256k1::global::SECP256K1
            .recover_ecdsa(secp256k1::Message::from_digest(hash.into()), &sig)
            .map_err(|_| CryptoError::SignatureVerification)?;

        PublicKey::try_from(pk.serialize_uncompressed().as_ref()).map_err(|_| CryptoError::CalculationError)
    }
}

/// Represents an ECDSA signature based on the secp256k1 curve with a recoverable public key.
/// The signature uses [Keccak256](Hash) as the hash function.
///
/// This signature encodes the 2-bit recovery information into the
/// uppermost bits from MSB of the `S` value, which are never used by this ECDSA
/// instantiation over secp256k1.
///
/// The instance holds the byte array consisting of `R` and `S` values with the recovery bit
/// already embedded in `S`.
///
/// See [EIP-2098](https://eips.ethereum.org/EIPS/eip-2098) for details.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChainSignature<E>(
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; ECDSA_SIGNATURE_SIZE],
    PhantomData<E>,
);

impl<E: EcdsaEngine> ChainSignature<E> {
    fn new(raw: RawSignature) -> Self {
        let mut ret = Self(raw.0, PhantomData);

        // Embed the parity recovery bit into the S value
        let parity = raw.1 & 0x01;
        ret.0[Self::SIZE / 2] &= 0x7f;
        ret.0[Self::SIZE / 2] |= parity << 7;

        ret
    }

    /// Returns the raw signature, without the encoded public key recovery bit and
    /// the recovery parity bit as a separate value.
    pub fn raw_signature(&self) -> RawSignature {
        let mut raw_sig = self.0;
        let parity: u8 = (raw_sig[Self::SIZE / 2] & 0x80 != 0).into();
        raw_sig[Self::SIZE / 2] &= 0x7f;
        (raw_sig, parity)
    }

    /// Signs the given message using the chain private key.
    #[inline]
    pub fn sign_message(message: &[u8], chain_keypair: &ChainKeypair) -> Self {
        Self::sign_hash(&Hash::create(&[message]), chain_keypair)
    }

    /// Signs the given hash using the raw private key.
    #[inline]
    pub fn sign_hash(hash: &Hash, chain_keypair: &ChainKeypair) -> Self {
        Self::new(
            E::sign_hash(hash, chain_keypair).expect("signing cannot fail: keypair always contains a valid secret key"),
        )
    }

    /// Verifies this signature against the given message and a public key object
    #[inline]
    pub fn verify_message(&self, message: &[u8], public_key: &PublicKey) -> crate::errors::Result<bool> {
        self.verify_hash(&Hash::create(&[message]), public_key)
    }

    /// Verifies this signature against the given hash and a public key object
    #[inline]
    pub fn verify_hash(&self, hash: &Hash, public_key: &PublicKey) -> crate::errors::Result<bool> {
        E::verify_hash(&self.raw_signature(), hash, public_key)
    }

    /// Recovers signer public key if this signature is a valid signature of the given `msg`.
    #[inline]
    pub fn recover_from_msg(&self, msg: &[u8]) -> crate::errors::Result<PublicKey> {
        self.recover_from_hash(&Hash::create(&[msg]))
    }

    /// Recovers signer public key if this signature is a valid signature of the given `hash`.
    #[inline]
    pub fn recover_from_hash(&self, hash: &Hash) -> crate::errors::Result<PublicKey> {
        // We saved only the parity bit in the S value, so test both x-coordinate signs.
        let (sig, parity) = self.raw_signature();
        for alt in [0u8, 2u8] {
            if let Ok(pk) = E::recover_from_hash(&(sig, parity | alt), hash) {
                return Ok(pk);
            }
        }
        Err(CryptoError::CalculationError)
    }
}

impl<E> AsRef<[u8]> for ChainSignature<E> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<E> TryFrom<&[u8]> for ChainSignature<E> {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        Ok(Self(
            value.try_into().map_err(|_| ParseError("Signature".into()))?,
            PhantomData,
        ))
    }
}

impl<E> BytesRepresentable for ChainSignature<E> {
    const SIZE: usize = ECDSA_SIGNATURE_SIZE;
}

#[cfg(not(feature = "rust-ecdsa"))]
pub type Signature = ChainSignature<NativeEcdsaSigningEngine>;

#[cfg(feature = "rust-ecdsa")]
pub type Signature = ChainSignature<K256EcdsaSigningEngine>;

/// Represents an EdDSA signature using the Ed25519 Edwards curve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OffchainSignature(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; Self::SIZE]);

impl OffchainSignature {
    /// Sign the given message using the [`OffchainKeypair`].
    pub fn sign_message(msg: &[u8], signing_keypair: &OffchainKeypair) -> Self {
        // Expand the SK from the given keypair
        let expanded_sk = ed25519_dalek::hazmat::ExpandedSecretKey::from(
            &ed25519_dalek::SecretKey::try_from(signing_keypair.secret().as_ref())
                .expect("cannot fail: OffchainKeypair always contains a valid secret key"),
        );

        // Get the verifying key from the SAME keypair, avoiding Double Public Key Signing Function Oracle Attack on
        // Ed25519 See https://github.com/MystenLabs/ed25519-unsafe-libs for details
        let verifying = ed25519_dalek::VerifyingKey::from(signing_keypair.public().edwards);

        ed25519_dalek::hazmat::raw_sign::<Sha512>(&expanded_sk, msg, &verifying).into()
    }

    /// Verify this signature of the given message and [OffchainPublicKey].
    pub fn verify_message(&self, msg: &[u8], public_key: &OffchainPublicKey) -> bool {
        let sgn = ed25519_dalek::Signature::from_slice(&self.0)
            .expect("cannot fail: OffchainSignature always contains a valid signature");
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
        let hash = Hash::create(&[&msg]);

        let kp = ChainKeypair::from_secret(&PRIVATE_KEY)?;

        let signature1 = Signature::sign_message(&msg, &kp);
        let signature2 = Signature::sign_hash(&hash, &kp);

        let pub_key1 = PublicKey::from_privkey(&PRIVATE_KEY)?;
        let pub_key2 = signature1.recover_from_msg(&msg)?;
        let pub_key3 = signature2.recover_from_hash(&hash)?;

        assert_eq!(pub_key1, *kp.public());
        assert_eq!(pub_key1, pub_key2, "recovered public key does not match");
        assert_eq!(pub_key1, pub_key3, "recovered public key does not match");

        assert!(
            signature1.verify_message(&msg, &pub_key1)?,
            "signature 1 verification failed with pub key 1"
        );
        assert!(
            signature1.verify_message(&msg, &pub_key2)?,
            "signature 1 verification failed with pub key 2"
        );
        assert!(
            signature1.verify_message(&msg, &pub_key3)?,
            "signature 1 verification failed with pub key 3"
        );

        assert!(
            signature2.verify_hash(&hash, &pub_key1)?,
            "signature 2 verification failed with pub key 1"
        );
        assert!(
            signature2.verify_hash(&hash, &pub_key2)?,
            "signature 2 verification failed with pub key 2"
        );
        assert!(
            signature2.verify_hash(&hash, &pub_key3)?,
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
