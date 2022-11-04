use aes::cipher::{KeyIvInit, StreamCipher};
use crate::parameters::{AES_BLOCK_SIZE, AES_KEY_SIZE, PRG_IV_LENGTH, PRG_KEY_LENGTH};

type Aes128Ctr32BE = ctr::Ctr32BE<aes::Aes128>;

#[derive(Default)]
pub struct PRG {
    key: [u8; AES_KEY_SIZE],
    iv: [u8; PRG_IV_LENGTH]
}

impl PRG {
    pub fn new(key: &[u8], iv: &[u8]) -> Result<Self, String> {
        if key.len() == PRG_KEY_LENGTH && iv.len() == PRG_IV_LENGTH {
            let mut ret: Self = Default::default();
            ret.key.copy_from_slice(key);
            ret.iv.copy_from_slice(iv);

            Ok(ret)
        }
        else {
            Err("invalid parameter size".into())
        }
    }

    pub fn digest(&self, from: usize, to: usize) -> Result<Box<[u8]>, String> {
        if from >= to {
            return Err("invalid parameter size".into())
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
        prefix.copy_from_slice(&self.iv);
        counter.copy_from_slice(&(first_block as u32).to_be_bytes());

        // Create key stream
        let mut cipher = Aes128Ctr32BE::new(&self.key.into(), &new_iv.into());
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
    use wasm_bindgen::JsValue;
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::utils::as_jsvalue;

    #[wasm_bindgen]
    pub struct PRG {
        w: super::PRG
    }

    #[wasm_bindgen]
    impl PRG {
        #[wasm_bindgen(constructor)]
        pub fn new(key: &[u8], iv: &[u8]) -> Result<PRG, JsValue> {
            Ok(Self {
                w: super::PRG::new(key,iv).map_err(as_jsvalue)?
            })
        }

        pub fn digest(&self, from: usize, to: usize) -> Result<Box<[u8]>, JsValue> {
            self.w.digest(from, to).map_err(as_jsvalue)
        }
    }

}