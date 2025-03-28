use hopr_crypto_sphinx::{
    derivation::{derive_ack_key_share, derive_own_key_share},
    shared_keys::SharedSecret,
};
use hopr_crypto_types::types::{Challenge, CurvePoint, HalfKey, HalfKeyChallenge, PublicKey, Response};
use hopr_primitive_types::prelude::*;
use tracing::error;

use crate::errors::{PacketError, Result};

/// Proof of Relay secret length is twice the size of secp256k1 public key
pub const POR_SECRET_LENGTH: usize = 2 * PublicKey::SIZE_COMPRESSED;

/// Type that contains the challenge for the first ticket sent to the first relayer.
#[derive(Clone)]
pub struct ProofOfRelayValues {
    pub ack_challenge: HalfKeyChallenge,
    pub ticket_challenge: Challenge,
    pub own_key: HalfKey,
}

impl ProofOfRelayValues {
    /// Takes the secrets which the first and the second relayer are able to derive from the packet header
    /// and computes the challenge for the first ticket.
    pub fn new(secret_b: &SharedSecret, secret_c: Option<&SharedSecret>) -> Self {
        let s0 = derive_own_key_share(secret_b);
        let s1 = derive_ack_key_share(secret_c.unwrap_or(&SharedSecret::random()));

        Self {
            // ack_challenge = s0_ack * G
            ack_challenge: derive_ack_key_share(secret_b).to_challenge(),
            // ticket challenge (s0_own + s1_ack) * G
            ticket_challenge: Response::from_half_keys(&s0, &s1)
                .expect("failed to derive response")
                .to_challenge(),
            own_key: s0,
        }
    }
}

/// Contains the Proof of Relay challenge for the next downstream node as well as the hint that is used to
/// verify the challenge that is given to the relayer.
#[derive(Clone)]
pub struct ProofOfRelayString {
    pub next_ticket_challenge: Challenge,
    pub hint: HalfKeyChallenge,
}

impl ProofOfRelayString {
    /// Creates an instance from the shared secrets with node+2 and node+3
    pub fn new(secret_c: &SharedSecret, secret_d: Option<&SharedSecret>) -> Self {
        let s0 = derive_ack_key_share(secret_c); // s0_ack
        let s1 = derive_own_key_share(secret_c); // s1_own
        let s2 = derive_ack_key_share(secret_d.unwrap_or(&SharedSecret::random())); // s2_ack

        Self {
            next_ticket_challenge: Response::from_half_keys(&s1, &s2) // (s1_own + s2_ack) * G
                .expect("failed to derive response")
                .to_challenge(),
            hint: s0.to_challenge(), // s0_ack * G
        }
    }

    /// Generates Proof of Relay challenges from the shared secrets of the
    /// outgoing packet.
    pub fn from_shared_secrets(secrets: &[SharedSecret]) -> Vec<[u8; ProofOfRelayString::SIZE]> {
        (1..secrets.len())
            .map(|i| ProofOfRelayString::new(&secrets[i], secrets.get(i + 1)).into())
            .collect::<Vec<_>>()
    }
}

impl TryFrom<&[u8]> for ProofOfRelayString {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        if value.len() == POR_SECRET_LENGTH {
            let (next_ticket_challenge, hint) = value.split_at(POR_SECRET_LENGTH / 2);
            Ok(Self {
                next_ticket_challenge: CurvePoint::try_from(next_ticket_challenge)?.into(),
                hint: HalfKeyChallenge::new(hint),
            })
        } else {
            Err(GeneralError::ParseError("ProofOfRelayString".into()))
        }
    }
}

impl From<ProofOfRelayString> for [u8; PROOF_OF_RELAY_STRING_LEN] {
    fn from(value: ProofOfRelayString) -> Self {
        let mut ret = [0u8; PROOF_OF_RELAY_STRING_LEN];
        ret[0..Challenge::SIZE].copy_from_slice(value.next_ticket_challenge.as_ref());
        ret[Challenge::SIZE..].copy_from_slice(value.hint.as_ref());
        ret
    }
}

const PROOF_OF_RELAY_STRING_LEN: usize = Challenge::SIZE + HalfKeyChallenge::SIZE;
impl BytesEncodable<PROOF_OF_RELAY_STRING_LEN> for ProofOfRelayString {}

/// Derivable challenge which contains the key share of the relayer as well as the secret that was used
/// to create it and the challenge for the next relayer.
#[derive(Clone)]
pub struct ProofOfRelayOutput {
    pub own_key: HalfKey,
    pub own_share: HalfKeyChallenge,
    pub next_ticket_challenge: Challenge,
    pub ack_challenge: HalfKeyChallenge,
}

/// Verifies that an incoming packet contains all values that are necessary to reconstruct the response to redeem the
/// incentive for relaying the packet.
/// # Arguments
/// * `secret` shared secret with the creator of the packet
/// * `por_bytes` serialized `ProofOfRelayString` as included within the packet
/// * `challenge` ticket challenge of the incoming ticket
pub fn pre_verify(
    secret: &SharedSecret,
    por_bytes: &[u8],
    challenge: &EthereumChallenge,
) -> Result<ProofOfRelayOutput> {
    assert_eq!(POR_SECRET_LENGTH, por_bytes.len(), "invalid por bytes length");

    let pors = ProofOfRelayString::try_from(por_bytes)?;

    let own_key = derive_own_key_share(secret);
    let own_share = own_key.to_challenge();

    if Challenge::from_hint_and_share(&own_share, &pors.hint)?
        .to_ethereum_challenge()
        .eq(challenge)
    {
        Ok(ProofOfRelayOutput {
            next_ticket_challenge: pors.next_ticket_challenge,
            ack_challenge: pors.hint,
            own_key,
            own_share,
        })
    } else {
        Err(PacketError::PoRVerificationError)
    }
}

/// Checks if the given acknowledgement solves the given challenge.
pub fn validate_por_half_keys(ethereum_challenge: &EthereumChallenge, own_key: &HalfKey, ack: &HalfKey) -> bool {
    Response::from_half_keys(own_key, ack)
        .map(|response| validate_por_response(ethereum_challenge, &response))
        .unwrap_or_else(|e| {
            error!(error = %e, "failed to validate por half keys");
            false
        })
}

/// Checks if the given response solves the given challenge.
pub fn validate_por_response(ethereum_challenge: &EthereumChallenge, response: &Response) -> bool {
    response.to_challenge().to_ethereum_challenge().eq(ethereum_challenge)
}

/// Checks if the given acknowledgement solves the given challenge.
pub fn validate_por_hint(ethereum_challenge: &EthereumChallenge, own_share: &HalfKeyChallenge, ack: &HalfKey) -> bool {
    Challenge::from_own_share_and_half_key(own_share, ack)
        .map(|c| c.to_ethereum_challenge().eq(ethereum_challenge))
        .unwrap_or_else(|e| {
            error!(error = %e,"failed to validate por hint");
            false
        })
}

#[cfg(test)]
mod tests {
    use crate::por::{
        pre_verify, validate_por_half_keys, validate_por_hint, validate_por_response, ProofOfRelayString,
        ProofOfRelayValues,
    };
    use hopr_crypto_sphinx::derivation::derive_ack_key_share;
    use hopr_crypto_sphinx::shared_keys::SharedSecret;
    use hopr_crypto_types::types::Response;
    use hopr_primitive_types::traits::BytesEncodable;

    #[test]
    fn test_por_preverify_validate() -> anyhow::Result<()> {
        const AMOUNT: usize = 4;

        let secrets = (0..AMOUNT).map(|_| SharedSecret::random()).collect::<Vec<_>>();

        // Generated challenge
        let first_challenge = ProofOfRelayValues::new(&secrets[0], Some(&secrets[1]));

        // For the first relayer
        let first_por_string = ProofOfRelayString::new(&secrets[1], Some(&secrets[2])).into_encoded();

        // For the second relayer
        let second_por_string = ProofOfRelayString::new(&secrets[2], Some(&secrets[3]));

        // Computation result of the first relayer before receiving an acknowledgement from the second relayer
        let first_challenge_eth = first_challenge.ticket_challenge.to_ethereum_challenge();
        let first_result = pre_verify(&secrets[0], &first_por_string, &first_challenge_eth)
            .expect("First challenge must be plausible");

        let expected_hkc = derive_ack_key_share(&secrets[1]).to_challenge();
        assert_eq!(expected_hkc, first_result.ack_challenge);

        // Simulates the transformation done by the first relayer
        let expected_pors = ProofOfRelayString::try_from(first_por_string.as_ref())?;
        assert_eq!(
            expected_pors.next_ticket_challenge, first_result.next_ticket_challenge,
            "Forward logic must extract correct challenge for the next downstream node"
        );

        // Computes the cryptographic material that is part of the acknowledgement
        let first_ack = derive_ack_key_share(&secrets[1]);
        assert!(
            validate_por_half_keys(
                &first_challenge.ticket_challenge.to_ethereum_challenge(),
                &first_result.own_key,
                &first_ack
            ),
            "Acknowledgement must solve the challenge"
        );

        // Simulates the transformation as done by the second relayer
        let first_result_challenge_eth = first_result.next_ticket_challenge.to_ethereum_challenge();
        let second_result = pre_verify(
            &secrets[1],
            &second_por_string.into_encoded(),
            &first_result_challenge_eth,
        )
        .expect("Second challenge must be plausible");

        let second_ack = derive_ack_key_share(&secrets[2]);
        assert!(
            validate_por_half_keys(
                &first_result.next_ticket_challenge.to_ethereum_challenge(),
                &second_result.own_key,
                &second_ack
            ),
            "Second acknowledgement must solve the challenge"
        );

        assert!(
            validate_por_hint(
                &first_result.next_ticket_challenge.to_ethereum_challenge(),
                &second_result.own_share,
                &second_ack
            ),
            "Second acknowledgement must solve the challenge"
        );

        Ok(())
    }

    #[test]
    fn test_challenge_and_response_solving() -> anyhow::Result<()> {
        const AMOUNT: usize = 2;
        let secrets = (0..AMOUNT).map(|_| SharedSecret::random()).collect::<Vec<_>>();

        let first_challenge = ProofOfRelayValues::new(&secrets[0], Some(&secrets[1]));
        let ack = derive_ack_key_share(&secrets[1]);

        assert!(
            validate_por_half_keys(
                &first_challenge.ticket_challenge.to_ethereum_challenge(),
                &first_challenge.own_key,
                &ack
            ),
            "Challenge must be solved"
        );

        assert!(
            validate_por_response(
                &first_challenge.ticket_challenge.to_ethereum_challenge(),
                &Response::from_half_keys(&first_challenge.own_key, &ack)?
            ),
            "Returned response must solve the challenge"
        );

        Ok(())
    }
}
