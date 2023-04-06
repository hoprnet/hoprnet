use core_crypto::parameters::SECRET_KEY_LENGTH;
use core_crypto::types::HalfKey;

pub fn derive_own_key_share(secret: &[u8]) -> HalfKey {
    assert_eq!(SECRET_KEY_LENGTH, secret.len());


}