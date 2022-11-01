use aes::cipher::{KeyIvInit, StreamCipher};

type Aes128Ctr32BE = ctr::Ctr32BE<aes::Aes128>;
const AES_BLOCK_SIZE: usize = 16;
const AES_KEY_SIZE: usize = 16;

const PRG_KEY_LENGTH: usize = AES_KEY_SIZE;
const PRG_COUNTER_LENGTH: usize = 4;
const PRG_IV_LENGTH: usize = AES_BLOCK_SIZE - PRG_COUNTER_LENGTH;

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
        let end = count_blocks - if last_block_end > 0 { AES_BLOCK_SIZE - last_block_end } else { 0 };

        // Allocate required memory
        let mut key_stream = vec![0u8; count_blocks * AES_BLOCK_SIZE];

        // Set correct counter value to the IV
        let mut new_iv = [0u8; AES_BLOCK_SIZE];
        let (prefix, counter) = new_iv.split_at_mut(PRG_IV_LENGTH);
        prefix.copy_from_slice(&self.iv);
        counter.copy_from_slice(&first_block.to_be_bytes());

        // Create key stream
        let mut cipher = Aes128Ctr32BE::new(&self.key.into(), &new_iv.into());
        cipher.apply_keystream(&mut key_stream);

        // Take the result as slice
        let result = &key_stream.as_slice()[start..end];
        Ok(Box::from(result))
    }

}