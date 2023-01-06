use crate::errors::Result;
use crate::errors::CryptoError::{InvalidInputSize, InvalidParameterSize};

use crate::parameters::{generate_key_iv, HASH_KEY_PRP, PRP_INTERMEDIATE_IV_LENGTH, PRP_INTERMEDIATE_KEY_LENGTH, PRP_IV_LENGTH, PRP_KEY_LENGTH, PRP_MIN_LENGTH};
use crate::primitives::{calculate_mac, SimpleStreamCipher};

pub struct PRPParameters {
    key: [u8; PRP_KEY_LENGTH],
    iv: [u8; PRP_IV_LENGTH]
}

impl Default for PRPParameters {
    fn default() -> Self {
        Self {
            key: [0u8; PRP_KEY_LENGTH],
            iv: [0u8; PRP_IV_LENGTH]
        }
    }
}

impl PRPParameters {
    pub fn new(secret: &[u8]) -> Result<Self> {
        let mut ret = PRPParameters::default();
        generate_key_iv(secret, HASH_KEY_PRP.as_bytes(), &mut ret.key, &mut ret.iv)?;
        Ok(ret)
    }
}

/// Implementation of Pseudo-Random Permutation (PRP).
/// Currently based on Lioness wide-block cipher
pub struct PRP {
    keys: [Vec<u8>; 4],
    ivs: [Vec<u8>; 4]
}

impl PRP {

    /// Creates new instance of the PRP
    pub fn new(key: &[u8], iv: &[u8]) -> Result<Self> {
        if key.len() != PRP_KEY_LENGTH {
            return Err(InvalidParameterSize{name: "key".into(), expected: PRP_KEY_LENGTH})
        }

        if iv.len() != PRP_IV_LENGTH {
            return Err(InvalidParameterSize{name: "iv".into(), expected: PRP_IV_LENGTH})
        }

        Ok(Self {
            keys: [
                key[0* PRP_INTERMEDIATE_KEY_LENGTH..1* PRP_INTERMEDIATE_KEY_LENGTH].to_vec(),
                key[1* PRP_INTERMEDIATE_KEY_LENGTH..2* PRP_INTERMEDIATE_KEY_LENGTH].to_vec(),
                key[2* PRP_INTERMEDIATE_KEY_LENGTH..3* PRP_INTERMEDIATE_KEY_LENGTH].to_vec(),
                key[3* PRP_INTERMEDIATE_KEY_LENGTH..4* PRP_INTERMEDIATE_KEY_LENGTH].to_vec()
            ],
            ivs: [ // NOTE: ChaCha20 takes only 12 byte IV
                iv[0* PRP_INTERMEDIATE_IV_LENGTH..1* PRP_INTERMEDIATE_IV_LENGTH].to_vec(),
                iv[1* PRP_INTERMEDIATE_IV_LENGTH..2* PRP_INTERMEDIATE_IV_LENGTH].to_vec(),
                iv[2* PRP_INTERMEDIATE_IV_LENGTH..3* PRP_INTERMEDIATE_IV_LENGTH].to_vec(),
                iv[3* PRP_INTERMEDIATE_IV_LENGTH..4* PRP_INTERMEDIATE_IV_LENGTH].to_vec()
            ]
        })
    }

    pub fn from_parameters(params: PRPParameters) -> Self {
        Self::new(&params.key, &params.iv).unwrap() // Parameter size checking taken care of by PRPParameters
    }

    /// Applies forward permutation on the given plaintext and returns a new buffer
    /// containing the result.
    pub fn forward(&self, plaintext: &[u8]) -> Result<Box<[u8]>> {
        if plaintext.len() < PRP_MIN_LENGTH {
            return Err(InvalidInputSize);
        }

        let mut out = Vec::from(plaintext);
        let data = out.as_mut_slice();

        Self::xor_keystream(data, &self.keys[0], &self.ivs[0])?;
        Self::xor_hash(data, &self.keys[1], &self.ivs[1])?;
        Self::xor_keystream(data, &self.keys[2], &self.ivs[2])?;
        Self::xor_hash(data, &self.keys[3], &self.ivs[3])?;

        Ok(out.into_boxed_slice())
    }

    /// Applies inverse permutation on the given plaintext and returns a new buffer
    /// containing the result.
    pub fn inverse(&self, ciphertext: &[u8]) -> Result<Box<[u8]>> {
        if ciphertext.len() < PRP_MIN_LENGTH {
            return Err(InvalidInputSize);
        }

        let mut out = Vec::from(ciphertext);
        let data = out.as_mut_slice();

        Self::xor_hash(data, &self.keys[3], &self.ivs[3])?;
        Self::xor_keystream(data, &self.keys[2], &self.ivs[2])?;
        Self::xor_hash(data, &self.keys[1], &self.ivs[1])?;
        Self::xor_keystream(data, &self.keys[0], &self.ivs[0])?;

        Ok(out.into_boxed_slice())
    }

    // Internal helper functions

    fn xor_hash(data: &mut [u8], key: &[u8], iv: &[u8]) -> Result<()> {
        let res = calculate_mac([key, iv].concat().as_slice(), &data[PRP_MIN_LENGTH..])?;
        Self::xor_inplace(data, res.as_ref());
        Ok(())
    }

    fn xor_inplace(a: &mut [u8], b: &[u8]) {
        let bound = if a.len() > b.len() { b.len() } else { a.len() };
        for i in 0..bound {
            a[i] = a[i] ^ b[i];
        }
    }

    fn xor_keystream(data: &mut [u8], key: &[u8], iv: &[u8]) -> Result<()> {
        let mut key_cpy = Vec::from(key);
        Self::xor_inplace(key_cpy.as_mut_slice(), &data[0..PRP_MIN_LENGTH]);
        let mut cipher = SimpleStreamCipher::new(key_cpy.as_slice(), iv)?;
        cipher.apply(&mut data[PRP_MIN_LENGTH..]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use getrandom::getrandom;
    use hex_literal::hex;
    use crate::prp::PRP;

    #[test]
    fn test_prp_fixed() {
        let key = [0u8; 4*32];
        let iv = [0u8; 4*16];

        let prp = PRP::new(&key, &iv).unwrap();

        let data = [1u8; 278];

        let ct = prp.forward(&data).unwrap();
        let pt = prp.inverse(&ct).unwrap();

        assert_eq!(&data, pt.as_ref());
    }

    #[test]
    fn test_prp_forward_only() {
        let key = [0u8; 4*32];
        let iv = [0u8; 4*16];

        let prp = PRP::new(&key, &iv).unwrap();

        let pt = [0u8; 100];
        let ct = prp.forward(&pt).unwrap();

        let expected_ct = hex!("e31d924dd07dbe87b54854a05cc09453b873d4b520f6cd787fbaa43e543ac9bf480457c20b39a93f4f05a7aa2566b944cedfcc1bec7fa0f456d361150835edca0c1e0c475350d39e2c658acced7d7cd00ded9dd44bbcd2b1ae367b3a7b2d3b45937ca118");
        assert_eq!([0u8;100], pt); // input is not overwritten
        assert_eq!(&expected_ct, ct.as_ref());
    }

    #[test]
    fn test_prp_inverse_only() {
        let key = [0u8; 4*32];
        let iv = [0u8; 4*16];

        let prp = PRP::new(&key, &iv).unwrap();

        let ct = hex!("e31d924dd07dbe87b54854a05cc09453b873d4b520f6cd787fbaa43e543ac9bf480457c20b39a93f4f05a7aa2566b944cedfcc1bec7fa0f456d361150835edca0c1e0c475350d39e2c658acced7d7cd00ded9dd44bbcd2b1ae367b3a7b2d3b45937ca118");
        let ct_c = hex!("e31d924dd07dbe87b54854a05cc09453b873d4b520f6cd787fbaa43e543ac9bf480457c20b39a93f4f05a7aa2566b944cedfcc1bec7fa0f456d361150835edca0c1e0c475350d39e2c658acced7d7cd00ded9dd44bbcd2b1ae367b3a7b2d3b45937ca118");
        let pt = prp.inverse(&ct).unwrap();

        let expected_pt = [0u8; 100];
        assert_eq!(ct_c, ct); // input is not overwritten
        assert_eq!(&expected_pt, pt.as_ref())
    }

    #[test]
    fn test_prp_random() {
        let mut key = [0u8; 4*32];
        getrandom(&mut key).unwrap();

        let mut iv = [0u8; 4*16];
        getrandom(&mut iv).unwrap();

        let prp = PRP::new(&key, &iv).unwrap();

        let mut data = [1u8; 278];
        getrandom(&mut data).unwrap();

        let ct = prp.forward(&data).unwrap();
        let pt = prp.inverse(&ct).unwrap();

        assert_eq!(&data, pt.as_ref());
    }
}

pub mod wasm {
    use wasm_bindgen::JsValue;
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::utils::{as_jsvalue, JsResult};

    #[wasm_bindgen]
    pub struct PRPParameters {
        w: super::PRPParameters
    }

    #[wasm_bindgen]
    impl PRPParameters {

        pub fn create(secret: &[u8]) -> Result<PRPParameters, JsValue> {
            Ok(Self {
                w: super::PRPParameters::new(secret).map_err(as_jsvalue)?
            })
        }
    }

    #[wasm_bindgen]
    pub struct PRP {
        w: super::PRP
    }

    #[wasm_bindgen]
    impl PRP {

        #[wasm_bindgen(constructor)]
        pub fn new(params: PRPParameters) -> PRP {
            Self {
                w: super::PRP::from_parameters(params.w)
            }
        }

        pub fn forward(&self, plaintext: &[u8]) -> JsResult<Box<[u8]>> {
            self.w.forward(plaintext)
                .map_err(as_jsvalue)
        }

        pub fn inverse(&self, ciphertext: &[u8]) -> JsResult<Box<[u8]>> {
            self.w.inverse(ciphertext)
                .map_err(as_jsvalue)
        }
    }
}