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
        let (next_ticket_challenge, hint) = por_bytes.split_at(POR_SECRET_LENGTH / 2);
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

#[cfg(test)]
mod tests {
    use core_crypto::derivation::derive_ack_key_share;
    use core_crypto::random::random_bytes;
    use core_crypto::parameters::SECRET_KEY_LENGTH;
    use core_crypto::types::Response;
    use crate::por::{create_por_string, create_por_values_for_sender, decode_por_bytes, pre_verify, validate_por_half_keys, validate_por_hint, validate_por_response};

    #[test]
    fn test_por_preverify_validate() {
        const AMOUNT: usize = 4;

        let secrets = (0..AMOUNT).map(|_| random_bytes::<SECRET_KEY_LENGTH>()).collect::<Vec<_>>();

        // Generated challenge
        let first_challenge = create_por_values_for_sender(&secrets[0], Some(&secrets[1]));

        // For the first relayer
        let first_por_string = create_por_string(&secrets[1], Some(&secrets[2]));

        // For the second relayer
        let second_por_string = create_por_string(&secrets[2], Some(&secrets[3]));

        // Computation result of the first relayer before receiving an acknowledgement from the second relayer
        let first_challenge_eth = first_challenge.ticket_challenge.to_ethereum_challenge();
        let first_result = pre_verify(&secrets[0], &first_por_string, first_challenge_eth)
            .expect("First challenge must be plausible");

        let expected_hkc = derive_ack_key_share(&secrets[1]).to_challenge();
        assert_eq!(expected_hkc, first_result.ack_challenge);

        // Simulates the transformation done by the first relayer
        let (expected_next_ticket_challenge, _) = decode_por_bytes(&first_por_string).unwrap();
        assert_eq!(expected_next_ticket_challenge, first_result.next_ticket_challenge,
                   "Forward logic must extract correct challenge for the next downstream node");

        // Computes the cryptographic material that is part of the acknowledgement
        let first_ack = derive_ack_key_share(&secrets[1]);
        assert!(validate_por_half_keys(&first_challenge.ticket_challenge.to_ethereum_challenge(), &first_result.own_key, &first_ack),
                "Acknowledgement must solve the challenge");

        // Simulates the transformation as done by the second relayer
        let first_result_challenge_eth = first_result.next_ticket_challenge.to_ethereum_challenge();
        let second_result =  pre_verify(&secrets[1], &second_por_string, first_result_challenge_eth)
            .expect("Second challenge must be plausible");

        let second_ack = derive_ack_key_share(&secrets[2]);
        assert!(validate_por_half_keys(&first_result.next_ticket_challenge.to_ethereum_challenge(), &second_result.own_key, &second_ack),
                "Second acknowledgement must solve the challenge");

        assert!(validate_por_hint(&first_result.next_ticket_challenge.to_ethereum_challenge(), &second_result.own_share, &second_ack),
                "Second acknowledgement must solve the challenge");
    }

    #[test]
    fn test_challenge_and_response_solving() {
        const AMOUNT: usize = 2;
        let secrets = (0..AMOUNT).map(|_| random_bytes::<SECRET_KEY_LENGTH>()).collect::<Vec<_>>();

        let first_challenge = create_por_values_for_sender(&secrets[0], Some(&secrets[1]));
        let ack = derive_ack_key_share(&secrets[1]);

        assert!(validate_por_half_keys(&first_challenge.ticket_challenge.to_ethereum_challenge(),
                                       &first_challenge.own_key, &ack), "Challenge must be solved");

        assert!(validate_por_response(&first_challenge.ticket_challenge.to_ethereum_challenge(),
        &Response::from_half_keys(&first_challenge.own_key, &ack)),
            "Returned response must solve the challenge");
    }
}