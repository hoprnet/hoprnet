
// General constants
pub const AES_BLOCK_SIZE: usize = 16;
pub const AES_KEY_SIZE: usize = 16;

/// Commitment specific
pub const HASH_KEY_COMMITMENT_SEED: &str = "HASH_KEY_COMMITMENT_SEED";
pub const HASH_KEY_HMAC: &str = "HASH_KEY_HMAC";

pub const SECRET_KEY_LENGTH: usize = 32;

/// PRP specific
pub const PRP_INTERMEDIATE_KEY_LENGTH: usize = 32;
pub const PRP_INTERMEDIATE_IV_LENGTH: usize = 16;

// The minimum input length must be at least size of the key, which is XORed with plaintext/ciphertext
pub const PRP_MIN_LENGTH: usize = PRP_INTERMEDIATE_KEY_LENGTH;

/// PRG specific
pub const PRG_KEY_LENGTH: usize = AES_KEY_SIZE;
pub const PRG_COUNTER_LENGTH: usize = 4;
pub const PRG_IV_LENGTH: usize = AES_BLOCK_SIZE - PRG_COUNTER_LENGTH;
