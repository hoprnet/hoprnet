use core_crypto::derivation::{derive_ack_key_share, derive_own_key_share};
use core_crypto::parameters::SECRET_KEY_LENGTH;
use core_crypto::random::random_bytes;
use core_crypto::types::{Challenge, CurvePoint, HalfKey, HalfKeyChallenge, PublicKey, Response};
use utils_types::errors::GeneralError;
use utils_types::errors::GeneralError::ParseError;
use utils_types::primitives::EthereumChallenge;
use utils_types::traits::BinarySerializable;

/// Proof of Relay secret length is twice the size of secp256k1 public key
pub const POR_SECRET_LENGTH: usize = 2 * PublicKey::SIZE_COMPRESSED;

#[derive(Clone)]
pub struct ProofOfRelayValues {
    pub ack_challenge: HalfKeyChallenge,
    pub ticket_challenge: Challenge,
    pub own_key: HalfKey
}

pub fn create_por_values_for_sender(secret_b: &[u8], secret_c: Option<&[u8]>) -> ProofOfRelayValues {
    let s0 = derive_own_key_share(secret_b);
    let s1 = derive_ack_key_share(secret_c.unwrap_or(&random_bytes::<SECRET_KEY_LENGTH>()));

    ProofOfRelayValues {
        ack_challenge: derive_ack_key_share(secret_b).to_challenge(),
        ticket_challenge: Response::from_half_keys(&s0, &s1).to_challenge(),
        own_key: s0
    }
}

pub fn create_por_string(secret_c: &[u8], secret_d: Option<&[u8]>) -> Box<[u8]> {
    assert_eq!(SECRET_KEY_LENGTH, secret_c.len(), "invalid secret length");
    assert!(secret_d.is_none() || secret_d.unwrap().len() == SECRET_KEY_LENGTH);

    let s0 = derive_ack_key_share(secret_c);
    let s1 = derive_own_key_share(secret_c);
    let s2 = derive_ack_key_share(secret_d.unwrap_or(&random_bytes::<SECRET_KEY_LENGTH>()));

    let mut ret = Vec::<u8>::with_capacity(2 * Challenge::SIZE);
    ret.extend_from_slice(&Response::from_half_keys(&s1, &s2).to_challenge().serialize());
    ret.extend_from_slice(&s0.to_challenge().serialize());
    ret.into_boxed_slice()
}

#[derive(Clone)]
pub struct ProofOfRelayOutput {
    pub own_key: HalfKey,
    pub own_share: HalfKeyChallenge,
    pub next_ticket_challenge: Challenge,
    pub ack_challenge: HalfKeyChallenge
}

pub fn decode_por_bytes(por_bytes: &[u8]) -> Result<(Challenge, HalfKeyChallenge), GeneralError> {
    if por_bytes.len() == POR_SECRET_LENGTH {
        let (next_ticket_challenge, hint) = por_bytes.split_at(PublicKey::SIZE_COMPRESSED);
        Ok((CurvePoint::deserialize(next_ticket_challenge)?.into(), HalfKeyChallenge::new(hint)))
    } else {
        Err(ParseError)
    }
}

pub fn pre_verify(secret: &[u8], por_bytes: &[u8], challenge: EthereumChallenge) -> Result<ProofOfRelayOutput, ()> {
    assert_eq!(SECRET_KEY_LENGTH, secret.len(), "invalid secret length");
    assert_eq!(POR_SECRET_LENGTH, por_bytes.len(), "invalid por bytes length");

    let (next_ticket_challenge, ack_challenge) = decode_por_bytes(por_bytes).map_err(|_|())?;

    let own_key = derive_own_key_share(secret);
    let own_share = own_key.to_challenge();

    if Challenge::from_hint_and_share(&own_share, &ack_challenge).map_err(|_|())?.to_ethereum_challenge() == challenge {
        Ok(ProofOfRelayOutput { own_key, own_share, next_ticket_challenge, ack_challenge })
    } else {
        Err(())
    }
}

pub fn validate_por_half_keys(ethereum_challenge: &EthereumChallenge, own_key: &HalfKey, ack: &HalfKey) -> bool {
    Response::from_half_keys(own_key, ack).to_challenge().to_ethereum_challenge().eq(ethereum_challenge)
}

pub fn validate_por_response(ethereum_challenge: &EthereumChallenge, response: &Response) -> bool {
    response.to_challenge().to_ethereum_challenge().eq(ethereum_challenge)
}

pub fn validate_por_hint(ethereum_challenge: &EthereumChallenge, own_share: &HalfKeyChallenge, ack: &HalfKey) -> bool {
    Challenge::from_own_share_and_half_key(own_share, ack)
        .map(|c| c.to_ethereum_challenge().eq(ethereum_challenge))
        .unwrap_or(false)
}