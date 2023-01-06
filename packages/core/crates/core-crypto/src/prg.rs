use aes::cipher::{KeyIvInit, StreamCipher};
use crate::derivation::generate_key_iv;
use crate::errors::Result;
use crate::errors::CryptoError::{InvalidInputSize, InvalidParameterSize};
use crate::parameters::{AES_BLOCK_SIZE, HASH_KEY_PRG, PRG_IV_LENGTH, PRG_KEY_LENGTH};

type Aes128Ctr32BE = ctr::Ctr32BE<aes::Aes128>;

pub struct PRGParameters {
    key: [u8; PRG_KEY_LENGTH],
    iv: [u8; PRG_IV_LENGTH]
}

impl Default for PRGParameters {
    fn default() -> Self {
        Self {
            key: [0u8; PRG_KEY_LENGTH],
            iv: [0u8; PRG_IV_LENGTH]
        }
    }
}

impl PRGParameters {
    pub fn new(secret: &[u8]) -> Result<Self> {
        let mut ret = PRGParameters::default();
        generate_key_iv(secret, HASH_KEY_PRG.as_bytes(), &mut ret.key, &mut ret.iv)?;
        Ok(ret)
    }
}

pub struct PRG {
    params: PRGParameters
}

impl PRG {
    pub fn new(key: &[u8], iv: &[u8]) -> Result<Self> {
        if key.len() != PRG_KEY_LENGTH {
            return Err(InvalidParameterSize { name: "key".into(), expected: PRG_KEY_LENGTH})
        }

        if iv.len() != PRG_IV_LENGTH {
            return Err(InvalidParameterSize { name: "iv".into(), expected: PRG_IV_LENGTH})
        }

        let mut ret = Self {
            params: PRGParameters::default()
        };

        ret.params.key.copy_from_slice(key);
        ret.params.iv.copy_from_slice(iv);

        Ok(ret)
    }

    pub fn from_parameters(params: PRGParameters) -> Self {
        Self::new(&params.key, &params.iv).unwrap() // Correct sizing taken care of by PRGParameters
    }

    pub fn digest(&self, from: usize, to: usize) -> Result<Box<[u8]>> {
        if from >= to {
            return Err(InvalidInputSize)
        }

        let first_block = from / AES_BLOCK_SIZE;
        let start = from % AES_BLOCK_SIZE;

        let last_block_end = to % AES_BLOCK_SIZE;
        let last_block = to / AES_BLOCK_SIZE + if last_block_end != 0 { 1 } else { 0 };
        let count_blocks = last_block - first_block;
        let end = AES_BLOCK_SIZE * count_blocks - if last_block_end > 0 { AES_BLOCK_SIZE - last_block_end } else { 0 };

        // Allocate required memory
        let mut key_stream = vec![0u8; count_blocks * AES_BLOCK_SIZE];

        // Set correct counter value to the IV
        // NOTE: We are using Big Endian ordering for the counter
        let mut new_iv = [0u8; AES_BLOCK_SIZE];
        let (prefix, counter) = new_iv.split_at_mut(PRG_IV_LENGTH);
        prefix.copy_from_slice(&self.params.iv);
        counter.copy_from_slice(&(first_block as u32).to_be_bytes());

        // Create key stream
        let mut cipher = Aes128Ctr32BE::new(&self.params.key.into(), &new_iv.into());
        cipher.apply_keystream(&mut key_stream);

        // Slice the result accordingly
        let result = &key_stream.as_slice()[start..end];
        Ok(Box::from(result))
    }
}

#[cfg(test)]
mod tests {
    use crate::parameters::{AES_BLOCK_SIZE, AES_KEY_SIZE};
    use crate::prg::PRG;

    #[test]
    fn test_prg_single_block() {
        let key = [0u8; 16];
        let iv = [0u8; 12];

        let out = PRG::new(&key, &iv)
            .unwrap()
            .digest(5,10)
            .unwrap();

        assert_eq!(5, out.len());
    }

    #[test]
    fn test_prg_more_blocks() {
        let key = [0u8; 16];
        let iv = [0u8; 12];

        let out = PRG::new(&key, &iv)
            .unwrap()
            .digest(0,AES_BLOCK_SIZE * 2)
            .unwrap();

        assert_eq!(32, out.len());
    }

    #[test]
    fn test_prg_across_blocks() {
        let key = [0u8; 16];
        let iv = [0u8; 12];

        let out = PRG::new(&key, &iv)
            .unwrap()
            .digest(5,AES_KEY_SIZE * 2 + 10)
            .unwrap();

        assert_eq!(AES_BLOCK_SIZE * 2 + 5, out.len());
    }
}

pub mod wasm {
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::utils::{as_jsvalue, JsResult};

    #[wasm_bindgen]
    pub struct PRGParameters {
        w: super::PRGParameters
    }

    #[wasm_bindgen]
    impl PRGParameters {

        pub fn create(secret: &[u8]) -> JsResult<PRGParameters> {
            Ok(Self {
                w: super::PRGParameters::new(secret).map_err(as_jsvalue)?
            })
        }
    }

    #[wasm_bindgen]
    pub struct PRG {
        w: super::PRG
    }

    #[wasm_bindgen]
    impl PRG {
        #[wasm_bindgen(constructor)]
        pub fn new(params: PRGParameters) -> PRG {
            Self {
                w: super::PRG::from_parameters(params.w)
            }
        }

        pub fn digest(&self, from: usize, to: usize) -> JsResult<Box<[u8]>> {
            self.w.digest(from, to).map_err(as_jsvalue)
        }
    }

}