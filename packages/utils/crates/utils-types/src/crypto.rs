// TODO: All types specified in this module will be moved over to the core-crypto crate once merged.

use k256::ecdsa::{SigningKey, Signature as ECDSASignature, signature::Signer, VerifyingKey};
use k256::{elliptic_curve, NonZeroScalar, Secp256k1};
use crate::errors::{Result, GeneralError::ParseError};

#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct PublicKey {
    key: elliptic_curve::PublicKey<Secp256k1>
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl PublicKey {
    pub fn eq(&self, other: &PublicKey) -> bool {
        self.key.eq(&other.key)
    }

    // TODO: Add more methods
}

impl PublicKey {
    pub fn deserialize(bytes: &[u8]) -> Result<PublicKey> {
        Ok(PublicKey{
            key: elliptic_curve::PublicKey::<Secp256k1>::from_sec1_bytes(bytes)
                .map_err(|_| ParseError)?
        })
    }
}

// TODO: Move all Signature related stuff to core-crypto once merged
pub const SIGNATURE_LENGTH: usize = 64;

#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Signature {
    signature: [u8; SIGNATURE_LENGTH],
    recovery: u8,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Signature {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(raw_bytes: &[u8], recovery: u8) -> Signature {
        assert_eq!(raw_bytes.len(), SIGNATURE_LENGTH, "invalid length");
        assert!(recovery <= 1, "invalid recovery bit");
        let mut ret = Self {
            signature: [0u8; SIGNATURE_LENGTH],
            recovery
        };
        ret.signature.copy_from_slice(raw_bytes);
        ret
    }

    pub fn sign_message(message: &[u8], private_key: &[u8]) -> Signature {
        let key = SigningKey::from_bytes(private_key)
            .expect("invalid signing key");
        let signature: ECDSASignature = key.sign(message);

        // TODO: Find if compression is really needed here
        Self::deserialize(signature.to_bytes().as_slice()).expect("signing failed")
    }

    pub fn verify(&self, message: &[u8], public_key: &[u8]) -> bool {
        //let pub_key = VerifyingKey::from_sec1_bytes(public_key)
        //    .expect("invalid public key");
        todo!()
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.signature)
    }

    pub fn raw_signature(&self) -> Box<[u8]> {
        self.signature.into()
    }

    pub fn serialize(&self) -> Box<[u8]> {
        let mut compressed = Vec::from(self.signature);
        compressed[SIGNATURE_LENGTH/2] &= 0x7f;
        compressed[SIGNATURE_LENGTH/2] |= self.recovery << 7;
        compressed.into_boxed_slice()
    }
}

impl Signature {
    pub fn deserialize(signature: &[u8]) -> Result<Signature> {
        if signature.len() == SIGNATURE_LENGTH {
            let mut ret = Signature {
                signature: [0u8; SIGNATURE_LENGTH],
                recovery: if signature[SIGNATURE_LENGTH/2]&0x80 != 0 { 1 } else { 0 }
            };
            ret.signature.copy_from_slice(signature);
            ret.signature[SIGNATURE_LENGTH/2] &= 0x7f;

            Ok(ret)
        } else {
            Err(ParseError)
        }
    }
}

#[cfg(test)]
pub mod tests {
    #[test]
    fn signature_tests() {

    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::*;
    use crate::crypto::Signature;

    #[wasm_bindgen]
    impl Signature {
        pub fn deserialize_signature(signature: &[u8]) -> JsResult<Signature> {
            ok_or_jserr!(Signature::deserialize(signature))
        }
    }
}