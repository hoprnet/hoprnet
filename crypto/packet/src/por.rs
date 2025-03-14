use hopr_crypto_sphinx::prelude::SharedSecret;
use hopr_crypto_types::prelude::{sample_secp256k1_field_element, SecretKey};
use hopr_crypto_types::types::{Challenge, HalfKey, HalfKeyChallenge, PublicKey, Response};
use hopr_primitive_types::prelude::*;
use tracing::error;

use crate::errors::{PacketError, Result};

const HASH_KEY_OWN_KEY: &str = "HASH_KEY_OWN_KEY";
const HASH_KEY_ACK_KEY: &str = "HASH_KEY_ACK_KEY";

/// Used in Proof of Relay to derive own half-key (S0)
/// The function samples a secp256k1 field element using the given `secret` via `sample_field_element`.
pub fn derive_own_key_share(secret: &SecretKey) -> HalfKey {
    sample_secp256k1_field_element(secret.as_ref(), HASH_KEY_OWN_KEY).expect("failed to sample own key share")
}

/// Used in Proof of Relay to derive the half-key of for the acknowledgement (S1)
/// The function samples a secp256k1 field element using the given `secret` via `sample_field_element`.
pub fn derive_ack_key_share(secret: &SecretKey) -> HalfKey {
    sample_secp256k1_field_element(secret.as_ref(), HASH_KEY_ACK_KEY).expect("failed to sample ack key share")
}

/// Proof of Relay secret length is twice the size of secp256k1 public key
pub const POR_SECRET_LENGTH: usize = 2 * PublicKey::SIZE_COMPRESSED;

/// Type that contains the challenge for the first ticket sent to the first relayer.
///
/// This is the first entry of the entire PoR challenge chain generated for the packet.
#[derive(Clone)]
pub struct ProofOfRelayValues([u8; Self::SIZE]);

impl ProofOfRelayValues {
    fn create(
        secret_b: &SharedSecret,
        secret_c: Option<&SharedSecret>,
        chain_length: u8,
    ) -> hopr_crypto_types::errors::Result<(Self, HalfKey)> {
        let s0 = derive_own_key_share(secret_b);
        let s1 = derive_ack_key_share(secret_c.unwrap_or(&SharedSecret::random()));

        let ack_challenge = derive_ack_key_share(secret_b).to_challenge();
        let ticket_challenge = Response::from_half_keys(&s0, &s1)?
            .to_challenge()
            .to_ethereum_challenge();

        let mut ret = [0u8; Self::SIZE];
        ret[0] = chain_length;
        ret[1..1 + HalfKeyChallenge::SIZE].copy_from_slice(ack_challenge.as_ref());
        ret[1 + HalfKeyChallenge::SIZE..].copy_from_slice(ticket_challenge.as_ref());
        Ok((Self(ret), s0))
    }

    /// Takes the secrets which the first and the second relayer are able to derive from the packet header
    /// and computes the challenge for the first ticket.
    pub fn new(
        secret_b: &SharedSecret,
        secret_c: Option<&SharedSecret>,
        chain_length: u8,
    ) -> hopr_crypto_types::errors::Result<Self> {
        Self::create(secret_b, secret_c, chain_length).map(|(v, _)| v)
    }

    /// Length of this PoR challenge chain (number of hops + 1).
    // TODO: needed to know how to price the ticket on the return path, will be fixed in #3765
    pub fn chain_length(&self) -> u8 {
        self.0[0]
    }

    /// Returns the challenge that must be solved once the acknowledgement
    /// to the packet has been received.
    ///
    /// This is the [`ProofOfRelayValues::ticket_challenge`] minus hint.
    pub fn acknowledgement_challenge(&self) -> HalfKeyChallenge {
        HalfKeyChallenge::new(&self.0[1..1 + HalfKeyChallenge::SIZE])
    }

    /// Returns the complete challenge that is present on the ticket corresponding to the
    /// packet.
    pub fn ticket_challenge(&self) -> EthereumChallenge {
        EthereumChallenge::new(&self.0[1 + HalfKeyChallenge::SIZE..])
    }
}

impl AsRef<[u8]> for ProofOfRelayValues {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<'a> TryFrom<&'a [u8]> for ProofOfRelayValues {
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> std::result::Result<Self, Self::Error> {
        value
            .try_into()
            .map(Self)
            .map_err(|_| GeneralError::ParseError("ProofOfRelayValues".into()))
    }
}

impl BytesRepresentable for ProofOfRelayValues {
    const SIZE: usize = 1 + HalfKeyChallenge::SIZE + EthereumChallenge::SIZE;
}

/// Contains the Proof of Relay challenge for the next downstream node as well as the hint that is used to
/// verify the challenge that is given to the relayer.
#[derive(Clone)]
pub struct ProofOfRelayString([u8; Self::SIZE]);

impl ProofOfRelayString {
    /// Creates an instance from the shared secrets with node+2 and node+3
    pub fn new(secret_c: &SharedSecret, secret_d: Option<&SharedSecret>) -> hopr_crypto_types::errors::Result<Self> {
        let s0 = derive_ack_key_share(secret_c); // s0_ack
        let s1 = derive_own_key_share(secret_c); // s1_own
        let s2 = derive_ack_key_share(secret_d.unwrap_or(&SharedSecret::random())); // s2_ack

        let next_ticket_challenge = Response::from_half_keys(&s1, &s2)? // (s1_own + s2_ack) * G
            .to_challenge()
            .to_ethereum_challenge();

        let hint = s0.to_challenge();

        let mut ret = [0u8; Self::SIZE];
        ret[0..EthereumChallenge::SIZE].copy_from_slice(next_ticket_challenge.as_ref());
        ret[EthereumChallenge::SIZE..].copy_from_slice(hint.as_ref());

        Ok(Self(ret))
    }

    /// Generates Proof of Relay challenges from the shared secrets of the
    /// outgoing packet.
    pub fn from_shared_secrets(secrets: &[SharedSecret]) -> hopr_crypto_types::errors::Result<Vec<ProofOfRelayString>> {
        (1..secrets.len())
            .map(|i| ProofOfRelayString::new(&secrets[i], secrets.get(i + 1)))
            .collect()
    }

    /// Challenge that must be printed on the ticket for the next downstream node.
    pub fn next_ticket_challenge(&self) -> EthereumChallenge {
        EthereumChallenge::new(&self.0[0..EthereumChallenge::SIZE])
    }

    /// Proof of Relay hint value for this node.
    pub fn hint(&self) -> HalfKeyChallenge {
        HalfKeyChallenge::new(&self.0[EthereumChallenge::SIZE..])
    }
}

impl TryFrom<&[u8]> for ProofOfRelayString {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        value
            .try_into()
            .map(Self)
            .map_err(|_| GeneralError::ParseError("ProofOfRelayString".into()))
    }
}

impl AsRef<[u8]> for ProofOfRelayString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
impl BytesRepresentable for ProofOfRelayString {
    const SIZE: usize = EthereumChallenge::SIZE + HalfKeyChallenge::SIZE;
}

/// Derivable challenge which contains the key share of the relayer as well as the secret that was used
/// to create it and the challenge for the next relayer.
#[derive(Clone)]
pub struct ProofOfRelayOutput {
    pub own_key: HalfKey,
    pub own_share: HalfKeyChallenge,
    pub next_ticket_challenge: EthereumChallenge,
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
    let pors = ProofOfRelayString::try_from(por_bytes)?;

    let own_key = derive_own_key_share(secret);
    let own_share = own_key.to_challenge();

    if Challenge::from_hint_and_share(&own_share, &pors.hint())?
        .to_ethereum_challenge()
        .eq(challenge)
    {
        Ok(ProofOfRelayOutput {
            next_ticket_challenge: pors.next_ticket_challenge(),
            ack_challenge: pors.hint(),
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
        derive_ack_key_share, pre_verify, validate_por_half_keys, validate_por_hint, validate_por_response,
        ProofOfRelayString, ProofOfRelayValues,
    };
    use hopr_crypto_sphinx::prelude::SharedSecret;
    use hopr_crypto_types::types::Response;

    #[test]
    fn test_por_preverify_validate() -> anyhow::Result<()> {
        const AMOUNT: usize = 4;

        let secrets = (0..AMOUNT).map(|_| SharedSecret::random()).collect::<Vec<_>>();

        // Generated challenge
        let first_challenge = ProofOfRelayValues::new(&secrets[0], Some(&secrets[1]), secrets.len() as u8)?;

        // For the first relayer
        let first_por_string = ProofOfRelayString::new(&secrets[1], Some(&secrets[2]))?;

        // For the second relayer
        let second_por_string = ProofOfRelayString::new(&secrets[2], Some(&secrets[3]))?;

        // Computation result of the first relayer before receiving an acknowledgement from the second relayer
        let first_challenge_eth = first_challenge.ticket_challenge();
        let first_result = pre_verify(&secrets[0], first_por_string.as_ref(), &first_challenge_eth)
            .expect("First challenge must be plausible");

        let expected_hkc = derive_ack_key_share(&secrets[1]).to_challenge();
        assert_eq!(expected_hkc, first_result.ack_challenge);

        // Simulates the transformation done by the first relayer
        let expected_pors = ProofOfRelayString::try_from(first_por_string.as_ref())?;
        assert_eq!(
            expected_pors.next_ticket_challenge(),
            first_result.next_ticket_challenge,
            "Forward logic must extract correct challenge for the next downstream node"
        );

        // Computes the cryptographic material that is part of the acknowledgement
        let first_ack = derive_ack_key_share(&secrets[1]);
        assert!(
            validate_por_half_keys(&first_challenge.ticket_challenge(), &first_result.own_key, &first_ack),
            "Acknowledgement must solve the challenge"
        );

        // Simulates the transformation as done by the second relayer
        let first_result_challenge_eth = first_result.next_ticket_challenge;
        let second_result = pre_verify(&secrets[1], second_por_string.as_ref(), &first_result_challenge_eth)
            .expect("Second challenge must be plausible");

        let second_ack = derive_ack_key_share(&secrets[2]);
        assert!(
            validate_por_half_keys(&first_result.next_ticket_challenge, &second_result.own_key, &second_ack),
            "Second acknowledgement must solve the challenge"
        );

        assert!(
            validate_por_hint(
                &first_result.next_ticket_challenge,
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

        let (first_challenge, own_key) =
            ProofOfRelayValues::create(&secrets[0], Some(&secrets[1]), secrets.len() as u8)?;
        let ack = derive_ack_key_share(&secrets[1]);

        assert!(
            validate_por_half_keys(&first_challenge.ticket_challenge(), &own_key, &ack),
            "Challenge must be solved"
        );

        assert!(
            validate_por_response(
                &first_challenge.ticket_challenge(),
                &Response::from_half_keys(&own_key, &ack)?
            ),
            "Returned response must solve the challenge"
        );

        Ok(())
    }
}
