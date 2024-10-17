use libp2p_identity::PeerId;
use hopr_primitive_types::prelude::GeneralError;
use crate::keypairs::OffchainKeypair;
use crate::prelude::CryptoError;
use crate::types::OffchainPublicKey;

pub fn seal_data<T: serde::Serialize>(data: T, peer_id: PeerId) -> crate::errors::Result<Box<[u8]>> {
    let opk = OffchainPublicKey::try_from(&peer_id)?;

    let recipient_pub_key = crypto_box::PublicKey::from_bytes(
        opk.as_ref().try_into().map_err(|_| CryptoError::InvalidPublicKey)?
    );

    let plain_text = bincode::serialize(&data)
        .map_err(|e| GeneralError::NonSpecificError(e.to_string()))?;

    recipient_pub_key.seal(&hopr_crypto_random::rng)
}

pub fn unseal_data<'a, T: serde::Deserialize<'a>>(data: &[u8], key: &OffchainKeypair) -> crate::errors::Result<T> {
    todo!()
}

#[cfg(test)]
mod tests {

}