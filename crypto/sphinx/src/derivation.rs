use hopr_crypto_types::prelude::*;

// Module-specific constants
const HASH_KEY_PACKET_TAG: &str = "HASH_KEY_PACKET_TAG";

pub(crate) fn create_kdf_instance<S: AsRef<[u8]>>(
    secret: &S,
    context: &str,
    salt: Option<&[u8]>,
) -> hopr_crypto_types::errors::Result<Blake3Output> {
    let key_material = secret.as_ref();
    if key_material.len() < 16 {
        return Err(CryptoError::InvalidInputValue("secret must have at least 128-bits"));
    }

    if let Some(salt) = salt {
        Ok(Blake3::new_derive_key(context)
            .update_reader(salt)
            .map_err(|_| CryptoError::InvalidInputValue("salt"))?
            .update_reader(key_material)
            .map_err(|_| CryptoError::InvalidInputValue("key"))?
            .finalize_xof())
    } else {
        Ok(Blake3::new_derive_key(context)
            .update_reader(key_material)
            .map_err(|_| CryptoError::InvalidInputValue("key"))?
            .finalize_xof())
    }
}

/// Derives the packet tag used during packet construction by expanding the given secret.
pub fn derive_packet_tag(secret: &SecretKey) -> hopr_crypto_types::errors::Result<PacketTag> {
    let mut packet_tag: PacketTag = [0u8; PACKET_TAG_LENGTH];

    let mut output = create_kdf_instance(secret, HASH_KEY_PACKET_TAG, None)?;
    output.fill(&mut packet_tag);
    Ok(packet_tag)
}

/// Internal convenience function to generate key and IV from the given secret,
/// that is cryptographically strong.
///
/// The `secret` must be at least 16 bytes long.
/// The function internally uses Blake2s256 based HKDF (see RFC 5869).
///
/// For `extract_with_salt` is given, the HKDF uses `Extract` with the given salt first
/// and then calls `Expand` to derive the key and IV.
///
/// Otherwise, only `Expand` is used to derive key and IV using the given `info`, but
/// the secret size must be exactly 32 bytes.
pub(crate) fn generate_key<T: crypto_traits::KeyInit, S: AsRef<[u8]>>(
    secret: &S,
    context: &str,
    with_salt: Option<&[u8]>,
) -> hopr_crypto_types::errors::Result<T> {
    let mut out = crypto_traits::Key::<T>::default();

    let mut output = create_kdf_instance(secret, context, with_salt)?;
    output.fill(&mut out);

    Ok(T::new(&out))
}

/// Internal convenience function to generate key and IV from the given secret,
/// that is cryptographically strong.
///
/// See [`generate_key`] for details.
pub(crate) fn generate_key_iv<T: crypto_traits::KeyIvInit, S: AsRef<[u8]>>(
    secret: &S,
    context: &str,
    with_salt: Option<&[u8]>,
) -> hopr_crypto_types::errors::Result<T> {
    let mut output = create_kdf_instance(secret, context, with_salt)?;

    let mut key = crypto_traits::Key::<T>::default();
    let mut iv = crypto_traits::Iv::<T>::default();

    let mut out = vec![0u8; key.len() + iv.len()];
    output.fill(&mut out);

    let (v_iv, v_key) = out.split_at(iv.len());
    iv.copy_from_slice(v_iv);
    key.copy_from_slice(v_key);

    Ok(T::new(&key, &iv))
}
