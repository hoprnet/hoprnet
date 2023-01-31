// TODO: All types specified in this module will be moved over to the core-crypto crate once merged.

use crate::errors::GeneralError;
use crate::errors::GeneralError::ParseError;
use crate::errors::Result;

#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct PublicKey {

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
    use crate::crypto::Signature;

    #[wasm_bindgen]
    impl Signature {
        pub fn deserialize_signature(signature: &[u8]) -> JsResult<Signature> {
            ok_or_jserr!(Signature::deserialize(signature))
        }
    }
}