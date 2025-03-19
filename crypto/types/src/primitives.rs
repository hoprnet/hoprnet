use crate::utils::SecretValue;

/// AES with 128-bit key in counter-mode (with big-endian counter).
pub type Aes128Ctr = ctr::Ctr64BE<aes::Aes128>;

// Re-exports of used cryptographic primitives
pub use blake2::Blake2s256;
pub use chacha20::ChaCha20;
pub use poly1305::Poly1305;
pub use sha3::Keccak256;
pub use sha3::Sha3_256;

/// Represents a 256-bit secret key of fixed length.
/// The value is auto-zeroized on drop.
pub type SecretKey = SecretValue<typenum::U32>;

/// Represents a 128-bit secret key of fixed length.
/// The value is auto-zeroized on drop.
pub type SecretKey16 = SecretValue<typenum::U16>;
