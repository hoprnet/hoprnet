use std::fmt::Formatter;

use hopr_types::{crypto::prelude::*, primitive::prelude::*};
use tracing::instrument;

use crate::{
    errors::{PacketError, Result},
    sphinx::prelude::SharedSecret,
};

const HASH_KEY_OWN_KEY: &str = "HASH_KEY_OWN_KEY";
const HASH_KEY_ACK_KEY: &str = "HASH_KEY_ACK_KEY";

/// Used in Proof of Relay to derive own half-key (S0)
/// The function samples a secp256k1 field element using the given `secret` via `sample_field_element`.
fn derive_own_key_share(secret: &SecretKey) -> HalfKey {
    sample_secp256k1_field_element(secret.as_ref(), HASH_KEY_OWN_KEY).expect("failed to sample own key share")
}

/// Used in Proof of Relay to derive the half-key of for the acknowledgement (S1)
/// The function samples a secp256k1 field element using the given `secret` via `sample_field_element`.
pub fn derive_ack_key_share(secret: &SecretKey) -> HalfKey {
    sample_secp256k1_field_element(secret.as_ref(), HASH_KEY_ACK_KEY).expect("failed to sample ack key share")
}

/// Type that contains the challenge for the first ticket sent to the first relayer.
///
/// This is the first entry of the entire PoR challenge chain generated for the packet.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProofOfRelayValues(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; Self::SIZE]);

impl ProofOfRelayValues {
    fn new(chain_len: u8, ack_challenge: &HalfKeyChallenge, ticket_challenge: &EthereumChallenge) -> Self {
        let mut ret = [0u8; Self::SIZE];
        ret[0] = chain_len;
        ret[1..1 + HalfKeyChallenge::SIZE].copy_from_slice(ack_challenge.as_ref());
        ret[1 + HalfKeyChallenge::SIZE..].copy_from_slice(ticket_challenge.as_ref());
        Self(ret)
    }

    /// Length of this PoR challenge chain (number of hops + 1).
    // TODO: needed to know how to price the ticket on the return path, will be fixed in #3765
    pub fn chain_length(&self) -> u8 {
        self.0[0]
    }

    /// Returns the challenge that must be solved once the acknowledgement
    /// to the packet has been received.
    ///
    /// This is the [`ProofOfRelayValues::ticket_challenge`] minus the Hint.
    pub fn acknowledgement_challenge(&self) -> HalfKeyChallenge {
        HalfKeyChallenge::new(&self.0[1..1 + HalfKeyChallenge::SIZE])
    }

    /// Returns the complete challenge that is present on the ticket corresponding to the
    /// packet.
    pub fn ticket_challenge(&self) -> EthereumChallenge {
        EthereumChallenge(Address::new(&self.0[1 + HalfKeyChallenge::SIZE..]))
    }
}

impl std::fmt::Debug for ProofOfRelayValues {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ProofOfRelayValues")
            .field(&self.chain_length())
            .field(&const_hex::encode(&self.0[1..1 + HalfKeyChallenge::SIZE]))
            .field(&const_hex::encode(&self.0[1 + HalfKeyChallenge::SIZE..]))
            .finish()
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
#[derive(Clone, PartialEq, Eq)]
pub struct ProofOfRelayString([u8; Self::SIZE]);

impl ProofOfRelayString {
    fn new(next_ticket_challenge: &EthereumChallenge, hint: &HalfKeyChallenge) -> Self {
        let mut ret = [0u8; Self::SIZE];
        ret[0..EthereumChallenge::SIZE].copy_from_slice(next_ticket_challenge.as_ref());
        ret[EthereumChallenge::SIZE..].copy_from_slice(hint.as_ref());
        Self(ret)
    }

    /// Challenge that must be printed on the ticket for the next downstream node.
    pub fn next_ticket_challenge(&self) -> EthereumChallenge {
        EthereumChallenge(Address::new(&self.0[0..EthereumChallenge::SIZE]))
    }

    /// Proof of Relay hint value for this node. In case this node is a sender
    /// of the packet, it contains the acknowledgement challenge.
    pub fn acknowledgement_challenge_or_hint(&self) -> HalfKeyChallenge {
        HalfKeyChallenge::new(&self.0[EthereumChallenge::SIZE..])
    }
}

impl std::fmt::Debug for ProofOfRelayString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ProofOfRelayString")
            .field(&const_hex::encode(&self.0[0..EthereumChallenge::SIZE]))
            .field(&const_hex::encode(&self.0[EthereumChallenge::SIZE..]))
            .finish()
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
    pub next_ticket_challenge: EthereumChallenge,
    pub ack_challenge: HalfKeyChallenge,
}

/// Verifies that an incoming packet contains all values that are necessary to reconstruct the response to redeem the
/// incentive for relaying the packet.
///
/// # Arguments
/// * `secret` shared secret with the creator of the packet
/// * `pors` `ProofOfRelayString` as included within the packet
/// * `challenge` the ticket challenge of the incoming ticket
#[instrument(level = "trace", skip_all, err)]
pub fn pre_verify(
    secret: &SharedSecret,
    pors: &ProofOfRelayString,
    challenge: &EthereumChallenge,
) -> Result<ProofOfRelayOutput> {
    let own_key = derive_own_key_share(secret);
    let own_share = own_key.to_challenge()?;

    if Challenge::from_hint_and_share(&own_share, &pors.acknowledgement_challenge_or_hint())?
        .to_ethereum_challenge()
        .eq(challenge)
    {
        Ok(ProofOfRelayOutput {
            next_ticket_challenge: pors.next_ticket_challenge(),
            ack_challenge: pors.acknowledgement_challenge_or_hint(),
            own_key,
        })
    } else {
        Err(PacketError::PoRVerificationError)
    }
}

/// Contains Proof of Relay values and a solution to the first acknowledgement challenge.
///
/// This is useful when additional pre-conditioning is needed on the acknowledgement
/// sent by the first relayer, such as in PIX.
pub type ProofOfRelayValuesWithSolution = (ProofOfRelayValues, HalfKey);

/// Helper function that generates proof of relay for the given path.
pub fn generate_proof_of_relay(
    secrets: &[SharedSecret],
) -> Result<(Vec<ProofOfRelayString>, ProofOfRelayValuesWithSolution)> {
    if secrets.is_empty() {
        return Err(PacketError::LogicError("no shared secrets".into()));
    }

    let mut last_ack_key_share = None;
    let mut por_strings = Vec::with_capacity(secrets.len());
    let mut por_values = None;

    let first_ack_key_share = derive_ack_key_share(&secrets[0]); // s0_ack

    for i in 0..secrets.len() {
        let hint = last_ack_key_share.unwrap_or(first_ack_key_share).to_challenge()?;

        let next_ticket_challenge = if let Some(next_secret) = secrets.get(i + 1) {
            let s1 = derive_own_key_share(&secrets[i]); // s1_own
            let s2 = derive_ack_key_share(next_secret); // s2_ack

            last_ack_key_share = Some(s2);

            Response::from_half_keys(&s1, &s2)? // (s1_own + s2_ack) * G
                .to_challenge()?
                .to_ethereum_challenge()
        } else {
            // NOTE: we do not generate a random ack_key_share to create the challenge for performance reasons
            // This means for 0-hop packets, the solution to the Proof of Relay is unknown, because
            // we do not even try to solve it in such case.
            EthereumChallenge(hopr_types::crypto_random::random_bytes::<{ Address::SIZE }>().into())
        };

        if i > 0 {
            por_strings.push(ProofOfRelayString::new(&next_ticket_challenge, &hint));
        } else {
            por_values = Some(ProofOfRelayValues::new(
                secrets.len() as u8,
                &hint,
                &next_ticket_challenge,
            ));
        }
    }

    Ok((
        por_strings,
        (
            // Cannot panic due to the first check
            por_values.expect("there must be shared secrets at this point"),
            first_ack_key_share,
        ),
    ))
}

#[cfg(test)]
mod tests {
    use hopr_types::crypto_random::Randomizable;

    use super::*;

    impl ProofOfRelayValues {
        fn create(
            secret_b: &SharedSecret,
            secret_c: Option<&SharedSecret>,
            chain_length: u8,
        ) -> hopr_types::crypto::errors::Result<(Self, HalfKey)> {
            let s0 = derive_own_key_share(secret_b);
            let s1 = derive_ack_key_share(secret_c.unwrap_or(&SharedSecret::random()));

            let ack_challenge = derive_ack_key_share(secret_b).to_challenge()?;
            let ticket_challenge = Response::from_half_keys(&s0, &s1)?
                .to_challenge()?
                .to_ethereum_challenge();

            Ok((Self::new(chain_length, &ack_challenge, &ticket_challenge), s0))
        }
    }
    impl ProofOfRelayString {
        /// Creates an instance from the shared secrets with node+2 and node+3
        fn create(
            secret_c: &SharedSecret,
            secret_d: Option<&SharedSecret>,
        ) -> hopr_types::crypto::errors::Result<Self> {
            let s0 = derive_ack_key_share(secret_c); // s0_ack
            let s1 = derive_own_key_share(secret_c); // s1_own
            let s2 = derive_ack_key_share(secret_d.unwrap_or(&SharedSecret::random())); // s2_ack

            let next_ticket_challenge = Response::from_half_keys(&s1, &s2)? // (s1_own + s2_ack) * G
                .to_challenge()?
                .to_ethereum_challenge();

            let hint = s0.to_challenge()?;
            Ok(Self::new(&next_ticket_challenge, &hint))
        }

        /// Generates Proof of Relay challenges from the shared secrets of the
        /// outgoing packet.
        fn from_shared_secrets(
            secrets: &[SharedSecret],
        ) -> hopr_types::crypto::errors::Result<Vec<ProofOfRelayString>> {
            (1..secrets.len())
                .map(|i| ProofOfRelayString::create(&secrets[i], secrets.get(i + 1)))
                .collect()
        }
    }

    /// Checks if the given acknowledgement solves the given challenge.
    fn validate_por_half_keys(ethereum_challenge: &EthereumChallenge, own_key: &HalfKey, ack: &HalfKey) -> bool {
        Response::from_half_keys(own_key, ack)
            .map(|response| validate_por_response(ethereum_challenge, &response))
            .unwrap_or(false)
    }

    /// Checks if the given response solves the given challenge.
    fn validate_por_response(ethereum_challenge: &EthereumChallenge, response: &Response) -> bool {
        response
            .to_challenge()
            .is_ok_and(|c| c.to_ethereum_challenge().eq(ethereum_challenge))
    }

    /// Checks if the given acknowledgement solves the given challenge.
    fn validate_por_hint(ethereum_challenge: &EthereumChallenge, own_share: &HalfKeyChallenge, ack: &HalfKey) -> bool {
        Challenge::from_own_share_and_half_key(own_share, ack)
            .map(|c| c.to_ethereum_challenge().eq(ethereum_challenge))
            .unwrap_or(false)
    }

    #[test]
    fn test_generate_proof_of_relay() -> anyhow::Result<()> {
        for hops in 0..=3 {
            let secrets = (0..=hops).map(|_| SharedSecret::random()).collect::<Vec<_>>();

            let por_strings = ProofOfRelayString::from_shared_secrets(&secrets)?;
            let por_values = ProofOfRelayValues::create(&secrets[0], secrets.get(1), secrets.len() as u8)?.0;

            let (gen_por_strings, (gen_por_values, gen_por_sol)) = generate_proof_of_relay(&secrets)?;

            // The solution to the first Proof of Relay must be the acknowledgement key share derived from the first
            // shared secret.
            assert_eq!(gen_por_sol, derive_ack_key_share(&secrets[0]));

            // The chain length should be the number of nodes in the path (hops + 1).

            assert_eq!(gen_por_values.chain_length(), (hops + 1) as u8);

            // The acknowledgement challenge in the PoR values must be solved by the generated solution.
            assert_eq!(gen_por_values.acknowledgement_challenge(), gen_por_sol.to_challenge()?);

            // The ticket challenge is randomly generated for 0-hop, so cannot compare them
            if hops > 0 {
                // For paths with at least one hop, the generated PoR values should match the expected values.
                assert_eq!(por_values, gen_por_values);

                // The ticket challenge of the first node must be solved by its own key share and the next node's
                // acknowledgement key share.
                assert!(validate_por_half_keys(
                    &gen_por_values.ticket_challenge(),
                    &derive_own_key_share(&secrets[0]),
                    &derive_ack_key_share(&secrets[1])
                ));

                // pre_verify should correctly transition from the current node's challenge to the next node's PoR data.
                let res = pre_verify(&secrets[0], &gen_por_strings[0], &gen_por_values.ticket_challenge())?;
                // The extracted next ticket challenge must match the one in the PoR string.
                assert_eq!(res.next_ticket_challenge, gen_por_strings[0].next_ticket_challenge());

                // The extracted acknowledgement challenge must match the hint/challenge in the PoR string.
                assert_eq!(
                    res.ack_challenge,
                    gen_por_strings[0].acknowledgement_challenge_or_hint()
                );

                // The derived own key must match the expected one for the first hop.
                assert_eq!(res.own_key, derive_own_key_share(&secrets[0]));
            }

            // The number of Proof of Relay strings should match the number of hops.
            assert_eq!(por_strings.len(), gen_por_strings.len());

            for i in 0..por_strings.len() {
                // Each PoR string's hint should match the expected one.
                assert_eq!(
                    por_strings[i].acknowledgement_challenge_or_hint(),
                    gen_por_strings[i].acknowledgement_challenge_or_hint()
                );
                // Each hint must be the challenge form of the corresponding node's acknowledgement key share.
                assert_eq!(
                    gen_por_strings[i].acknowledgement_challenge_or_hint(),
                    derive_ack_key_share(&secrets[i + 1]).to_challenge()?
                );

                // The ticket challenge is randomly generated, so cannot compare them
                if i != por_strings.len() - 1 {
                    // The generated next ticket challenge should match the expected one.
                    assert_eq!(
                        por_strings[i].next_ticket_challenge(),
                        gen_por_strings[i].next_ticket_challenge()
                    );

                    // Each hop's ticket challenge must be solved by its own key share and the next hop's
                    // acknowledgement key share.
                    assert!(validate_por_half_keys(
                        &gen_por_strings[i].next_ticket_challenge(),
                        &derive_own_key_share(&secrets[i + 1]),
                        &derive_ack_key_share(&secrets[i + 2])
                    ));

                    // pre_verify should work for all hops, correctly extracting the next challenge and verifying the
                    // current one.
                    let res = pre_verify(
                        &secrets[i + 1],
                        &gen_por_strings[i + 1],
                        &gen_por_strings[i].next_ticket_challenge(),
                    )?;
                    // Verifies that the forwarded challenge matches the next one in the chain.
                    assert_eq!(
                        res.next_ticket_challenge,
                        gen_por_strings[i + 1].next_ticket_challenge()
                    );
                    // Verifies that the extracted acknowledgement challenge matches the next hint.
                    assert_eq!(
                        res.ack_challenge,
                        gen_por_strings[i + 1].acknowledgement_challenge_or_hint()
                    );
                    // Verifies that the correct own key is derived for each intermediate hop.
                    assert_eq!(res.own_key, derive_own_key_share(&secrets[i + 1]));
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_por_preverify_validate() -> anyhow::Result<()> {
        const AMOUNT: usize = 4;

        let secrets = (0..AMOUNT).map(|_| SharedSecret::random()).collect::<Vec<_>>();

        // Generated challenge
        let first_challenge = ProofOfRelayValues::create(&secrets[0], Some(&secrets[1]), secrets.len() as u8)?.0;

        // For the first relayer
        let first_por_string = ProofOfRelayString::create(&secrets[1], Some(&secrets[2]))?;

        // For the second relayer
        let second_por_string = ProofOfRelayString::create(&secrets[2], Some(&secrets[3]))?;

        // Computation result of the first relayer before receiving an acknowledgement from the second relayer
        let first_challenge_eth = first_challenge.ticket_challenge();
        let first_result = pre_verify(&secrets[0], &first_por_string, &first_challenge_eth)
            .expect("First challenge must be plausible");

        let expected_hkc = derive_ack_key_share(&secrets[1]).to_challenge()?;
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
        let second_result = pre_verify(&secrets[1], &second_por_string, &first_result_challenge_eth)
            .expect("Second challenge must be plausible");

        let second_ack = derive_ack_key_share(&secrets[2]);
        assert!(
            validate_por_half_keys(&first_result.next_ticket_challenge, &second_result.own_key, &second_ack),
            "Second acknowledgement must solve the challenge"
        );

        assert!(
            validate_por_hint(
                &first_result.next_ticket_challenge,
                &second_result.own_key.to_challenge()?,
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
