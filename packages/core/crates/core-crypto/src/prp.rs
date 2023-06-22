use crate::derivation::generate_key_iv;
use crate::errors::CryptoError::InvalidParameterSize;
use crate::errors::Result;

use crate::primitives::{calculate_mac, SecretKey, SimpleStreamCipher};
use crate::utils;

// Module-specific constants
const PRP_INTERMEDIATE_KEY_LENGTH: usize = 32;
const PRP_INTERMEDIATE_IV_LENGTH: usize = 16;
const PRP_KEY_LENGTH: usize = 4 * PRP_INTERMEDIATE_KEY_LENGTH;
const PRP_IV_LENGTH: usize = 4 * PRP_INTERMEDIATE_IV_LENGTH;
const HASH_KEY_PRP: &str = "HASH_KEY_PRP";

// The minimum input length must be at least size of the key, which is XORed with plaintext/ciphertext
pub const PRP_MIN_LENGTH: usize = PRP_INTERMEDIATE_KEY_LENGTH;

/// Parameters for the Pseudo-Random Permutation (PRP) function
/// This consists of IV and the raw secret key for use by the underlying cryptographic transformation.
pub struct PRPParameters {
    key: [u8; PRP_KEY_LENGTH],
    iv: [u8; PRP_IV_LENGTH],
}

impl Default for PRPParameters {
    fn default() -> Self {
        Self {
            key: [0u8; PRP_KEY_LENGTH],
            iv: [0u8; PRP_IV_LENGTH],
        }
    }
}

impl PRPParameters {
    /// Creates new parameters for the PRP by expanding the given
    /// keying material into the secret key and IV for the underlying cryptographic transformation.
    pub fn new(secret: &SecretKey) -> Self {
        let mut ret = PRPParameters::default();
        generate_key_iv(secret, HASH_KEY_PRP.as_bytes(), &mut ret.key, &mut ret.iv, false);
        ret
    }
}

/// Implementation of Pseudo-Random Permutation (PRP).
/// Currently based on the Lioness wide-block cipher.
pub struct PRP {
    keys: [Box<[u8]>; 4],
    ivs: [Box<[u8]>; 4],
}

impl PRP {
    /// Creates new instance of the PRP using the raw key and IV.
    pub fn new(key: [u8; PRP_KEY_LENGTH], iv: [u8; PRP_IV_LENGTH]) -> Self {
        Self {
            keys: [
                key[0 * PRP_INTERMEDIATE_KEY_LENGTH..1 * PRP_INTERMEDIATE_KEY_LENGTH].into(),
                key[1 * PRP_INTERMEDIATE_KEY_LENGTH..2 * PRP_INTERMEDIATE_KEY_LENGTH].into(),
                key[2 * PRP_INTERMEDIATE_KEY_LENGTH..3 * PRP_INTERMEDIATE_KEY_LENGTH].into(),
                key[3 * PRP_INTERMEDIATE_KEY_LENGTH..4 * PRP_INTERMEDIATE_KEY_LENGTH].into(),
            ],
            ivs: [
                // NOTE: ChaCha20 takes only 12 byte IV
                iv[0 * PRP_INTERMEDIATE_IV_LENGTH..1 * PRP_INTERMEDIATE_IV_LENGTH].into(),
                iv[1 * PRP_INTERMEDIATE_IV_LENGTH..2 * PRP_INTERMEDIATE_IV_LENGTH].into(),
                iv[2 * PRP_INTERMEDIATE_IV_LENGTH..3 * PRP_INTERMEDIATE_IV_LENGTH].into(),
                iv[3 * PRP_INTERMEDIATE_IV_LENGTH..4 * PRP_INTERMEDIATE_IV_LENGTH].into(),
            ],
        }
    }

    /// Creates a new PRP instance using the given parameters
    pub fn from_parameters(params: PRPParameters) -> Self {
        Self::new(params.key, params.iv) // Parameter size checking taken care of by PRPParameters
    }
}

impl PRP {
    /// Applies forward permutation on the given plaintext and returns a new buffer
    /// containing the result.
    pub fn forward(&self, plaintext: &[u8]) -> Result<Box<[u8]>> {
        let mut out: Vec<u8> = plaintext.into();
        self.forward_inplace(&mut out)?;
        Ok(out.into_boxed_slice())
    }

    /// Applies forward permutation on the given plaintext and modifies the given buffer in-place.
    pub fn forward_inplace(&self, plaintext: &mut [u8]) -> Result<()> {
        if plaintext.len() >= PRP_MIN_LENGTH {
            Self::xor_keystream(plaintext, &self.keys[0], &self.ivs[0]);
            Self::xor_hash(plaintext, &self.keys[1], &self.ivs[1]);
            Self::xor_keystream(plaintext, &self.keys[2], &self.ivs[2]);
            Self::xor_hash(plaintext, &self.keys[3], &self.ivs[3]);
            Ok(())
        } else {
            Err(InvalidParameterSize {
                name: "plaintext".into(),
                expected: PRP_MIN_LENGTH
            })
        }
    }

    /// Applies inverse permutation on the given plaintext and returns a new buffer
    /// containing the result.
    pub fn inverse(&self, ciphertext: &[u8]) -> Result<Box<[u8]>> {
        let mut out: Vec<u8> = ciphertext.into();
        self.inverse_inplace(&mut out)?;
        Ok(out.into_boxed_slice())
    }

    /// Applies inverse permutation on the given ciphertext and modifies the given buffer in-place.
    pub fn inverse_inplace(&self, ciphertext: &mut [u8]) -> Result<()> {
        if ciphertext.len() >= PRP_MIN_LENGTH {
            Self::xor_hash(ciphertext, &self.keys[3], &self.ivs[3]);
            Self::xor_keystream(ciphertext, &self.keys[2], &self.ivs[2]);
            Self::xor_hash(ciphertext, &self.keys[1], &self.ivs[1]);
            Self::xor_keystream(ciphertext, &self.keys[0], &self.ivs[0]);
            Ok(())
        } else {
            Err(InvalidParameterSize {
                name: "ciphertext".into(),
                expected: PRP_MIN_LENGTH
            })
        }
    }

    // Internal helper functions

    fn xor_hash(data: &mut [u8], key: &[u8], iv: &[u8]) {
        let res = calculate_mac([key, iv].concat().as_slice(), &data[PRP_MIN_LENGTH..]);
        utils::xor_inplace(data, res.as_ref());
    }

    fn xor_keystream(data: &mut [u8], key: &[u8], iv: &[u8]) {
        let mut key_cpy = Vec::from(key);
        utils::xor_inplace(key_cpy.as_mut_slice(), &data[0..PRP_MIN_LENGTH]);

        let iv_cpy = &iv[4..iv.len()];

        let mut cipher = SimpleStreamCipher::new(
            key_cpy.try_into().expect("invalid keystream key size"),
            iv_cpy.try_into().expect("invalid keystream iv size")
        );

        let block_counter = u32::from_le_bytes(iv[0..4].try_into().unwrap());
        cipher.set_block_counter(block_counter);

        cipher.apply(&mut data[PRP_MIN_LENGTH..]);
    }
}

#[cfg(test)]
mod tests {
    use crate::prp::{PRPParameters, PRP};
    use hex_literal::hex;
    use crate::primitives::SecretKey;
    use crate::random::random_bytes;

    #[test]
    fn test_prp_fixed() {
        let prp = PRP::new([0u8; 4 * 32], [0u8; 4 * 16]);

        let data = [1u8; 278];

        let ct = prp.forward(&data).unwrap();
        let pt = prp.inverse(&ct).unwrap();

        assert_eq!(&data, pt.as_ref());
    }

    #[test]
    fn test_prp_random() {
        let prp = PRP::new(random_bytes(), random_bytes());
        let data: [u8; 278] = random_bytes();

        let ct = prp.forward(&data).unwrap();
        let pt = prp.inverse(&ct).unwrap();

        assert_ne!(&data, ct.as_ref(), "ciphertext must be different than plaintext");
        assert_eq!(&data, pt.as_ref(), "plaintexts must be the same");
    }

    #[test]
    fn test_prp_random_inplace() {
        let prp = PRP::new(random_bytes(), random_bytes());
        let mut data: [u8; 278] = random_bytes();
        let data_old = data.clone();

        prp.forward_inplace(&mut data).unwrap();
        assert_ne!(&data_old, data.as_ref(), "buffer must be encrypted in-place");

        prp.inverse_inplace(&mut data).unwrap();
        assert_eq!(&data_old, data.as_ref(), "buffer must be decrypted in-place");
    }

    #[test]
    fn test_prp_parameters() {
        let expected_key = hex!("a9c6632c9f76e5e4dd03203196932350a47562f816cebb810c64287ff68586f35cb715a26e268fc3ce68680e16767581de4e2cb3944c563d1f1a0cc077f3e788a12f31ae07111d77a876a66de5bdd6176bdaa2e07d1cb2e36e428afafdebb2109f70ce8422c8821233053bdd5871523ffb108f1e0f86809999a99d407590df25");
        let expected_iv = hex!("a59991716be504b26471dea53d688c4bab8e910328e54ebb6ebf07b49e6d12eacfc56e0935ba2300559b43ede25aa09eee7e8a2deea5f0bdaee2e859834edd38");

        let params = PRPParameters::new(&SecretKey::default());

        assert_eq!(expected_key, params.key);
        assert_eq!(expected_iv, params.iv)
    }

    #[test]
    fn test_prp_ciphertext_from_params() {
        let params = PRPParameters::new(&SecretKey::default());

        let expected_key = hex!("a9c6632c9f76e5e4dd03203196932350a47562f816cebb810c64287ff68586f35cb715a26e268fc3ce68680e16767581de4e2cb3944c563d1f1a0cc077f3e788a12f31ae07111d77a876a66de5bdd6176bdaa2e07d1cb2e36e428afafdebb2109f70ce8422c8821233053bdd5871523ffb108f1e0f86809999a99d407590df25");
        let expected_iv = hex!("a59991716be504b26471dea53d688c4bab8e910328e54ebb6ebf07b49e6d12eacfc56e0935ba2300559b43ede25aa09eee7e8a2deea5f0bdaee2e859834edd38");
        assert_eq!(expected_key, params.key);
        assert_eq!(expected_iv, params.iv);

        let prp = PRP::from_parameters(params);

        let pt = [0u8; 100];
        let ct = prp.forward(&pt).unwrap();

        assert_eq!([0u8; 100], pt, "plain text must not change for in-place operation");
        assert_ne!(&pt, ct.as_ref(), "plain text must be different from ciphertext");

        //let expected_ct = hex!("f80036d72b5e61e20f3f5840a013d12b5dd496f2da55b930f961905fbbbc8158dc17b58510bf280d0359e0b233a099bde840e07d54ca308e55ee0196b8f013b5def9b6a3ec9a727071c5dbdbeabdedcecfbdc3ecdd69fdcd957ff60ac573cc0dbab45b04");
        //assert_eq!(&expected_ct, ct.as_ref());
    }
}
