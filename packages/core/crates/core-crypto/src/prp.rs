use crate::primitives::{calculate_mac, SimpleStreamCipher};

const INTERMEDIATE_KEY_LENGTH: usize = 32;
const INTERMEDIATE_IV_LENGTH: usize = 16;
const HASH_LENGTH: usize = 32;
const PRP_MIN_LENGTH: usize = 32;

pub struct PRP {
    keys: [Vec<u8>; 4],
    ivs: [Vec<u8>; 4]
}

impl PRP {
    pub fn new(key: &[u8], iv: &[u8]) -> Self {
        Self {
            keys: [
                key[0*INTERMEDIATE_KEY_LENGTH..1*INTERMEDIATE_KEY_LENGTH].to_vec(),
                key[1*INTERMEDIATE_KEY_LENGTH..2*INTERMEDIATE_KEY_LENGTH].to_vec(),
                key[2*INTERMEDIATE_KEY_LENGTH..3*INTERMEDIATE_KEY_LENGTH].to_vec(),
                key[3*INTERMEDIATE_KEY_LENGTH..4*INTERMEDIATE_KEY_LENGTH].to_vec()
            ],
            ivs: [
                iv[0*INTERMEDIATE_IV_LENGTH..1*INTERMEDIATE_IV_LENGTH].to_vec(),
                iv[1*INTERMEDIATE_IV_LENGTH..2*INTERMEDIATE_IV_LENGTH].to_vec(),
                iv[2*INTERMEDIATE_IV_LENGTH..3*INTERMEDIATE_IV_LENGTH].to_vec(),
                iv[3*INTERMEDIATE_IV_LENGTH..4*INTERMEDIATE_IV_LENGTH].to_vec()
            ]
        }
    }

    pub fn permute(&mut self, plaintext: &[u8]) -> Result<Box<[u8]>, String> {
        if plaintext.len() < PRP_MIN_LENGTH {
            return Err(format!("Expected plaintext with a length of a least {} bytes. Got {}.", PRP_MIN_LENGTH, plaintext.len()));
        }

        let mut out = Vec::from(plaintext);
        let mut data = out.as_mut_slice();

        Self::xor_keystream(data, &self.keys[0], &self.ivs[0])?;
        Self::xor_hash(data, &self.keys[1], &self.ivs[1])?;
        Self::xor_keystream(data, &self.keys[2], &self.ivs[2])?;
        Self::xor_hash(data, &self.keys[3], &self.ivs[3])?;

        Ok(out.into_boxed_slice())
    }

    pub fn inverse(&mut self, plaintext: &[u8]) -> Result<Box<[u8]>, String> {
        if plaintext.len() < PRP_MIN_LENGTH {
            return Err(format!("Expected plaintext with a length of a least {} bytes. Got {}.", PRP_MIN_LENGTH, plaintext.len()));
        }

        let mut out = Vec::from(plaintext);
        let mut data = out.as_mut_slice();

        Self::xor_hash(data, &self.keys[3], &self.ivs[3])?;
        Self::xor_keystream(data, &self.keys[2], &self.ivs[2])?;
        Self::xor_hash(data, &self.keys[1], &self.ivs[1])?;
        Self::xor_keystream(data, &self.keys[0], &self.ivs[0])?;

        Ok(out.into_boxed_slice())
    }

    fn xor_hash(data: &mut [u8], key: &[u8], iv: &[u8]) -> Result<(), String> {
        let res = calculate_mac([key, iv].concat().as_slice(), data)?;
        for i in 0..data.len() {
            data[i] = data[i] ^ res[i];
        }
        Ok(())
    }

    fn xor_keystream(data: &mut [u8], key: &[u8], iv: &[u8]) -> Result<(), String> {
        let mut cipher = SimpleStreamCipher::new(key, iv)?;
        cipher.apply(data);
        Ok(())
    }
}