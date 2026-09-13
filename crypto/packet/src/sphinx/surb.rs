use std::fmt::Formatter;

use hopr_types::{
    crypto::prelude::*,
    crypto_random::Randomizable,
    primitive::{
        hybrid_array::{Array, typenum::Unsigned},
        prelude::*,
    },
};
use subtle::ConstantTimeEq;

use super::{
    routing::{RoutingInfo, SphinxHeaderSpec},
    shared_keys::{Alpha, GroupElement, SharedKeys, SharedSecret, SphinxSuite},
};

/// Single Use Reply Block
///
/// This is delivered to the recipient, so they are able to send reply messages back
/// anonymously (via the return path inside that SURB).
///
/// [`SURB`] is always created in a pair with [`ReplyOpener`], so that the sending
/// party knows how to decrypt the data.
///
/// The SURB sent to the receiving party must be accompanied
/// by a `Pseudonym`, and once the receiving party uses that SURB to send a reply, it
/// must be accompanied by the same `Pseudonym`.
/// Upon receiving such a reply, the reply recipient (= sender of the SURB)
/// uses the `Pseudonym` to find the `ReplyOpener` created with the SURB to read the reply.
///
/// Always use [`create_surb`] to create the [`SURB`] and [`ReplyOpener`] pair.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SURB<S: SphinxSuite, H: SphinxHeaderSpec> {
    /// ID of the first relayer.
    pub first_relayer: H::KeyId,
    /// Alpha value for the header.
    pub alpha: Alpha<<S::G as GroupElement<S::E>>::AlphaLen>,
    /// Sphinx routing header.
    pub header: RoutingInfo<H>,
    /// Encryption key to use to encrypt the data for the SURB's creator.
    pub sender_key: SecretKey16,
    /// Additional data for the SURB receiver.
    pub additional_data_receiver: H::SurbReceiverData,
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> SURB<S, H> {
    /// Size of the SURB in bytes.
    pub const SIZE: usize = H::KEY_ID_SIZE.get()
        + <S::G as GroupElement<S::E>>::AlphaLen::USIZE
        + RoutingInfo::<H>::SIZE
        + SecretKey16::LENGTH
        + H::SURB_RECEIVER_DATA_SIZE;

    /// Serializes SURB into wire format.
    pub fn into_boxed(self) -> Box<[u8]> {
        let alpha_len = <S::G as GroupElement<S::E>>::AlphaLen::USIZE;

        let mut ret = vec![0u8; Self::SIZE];
        ret[..H::KEY_ID_SIZE.get()].copy_from_slice(self.first_relayer.as_ref());
        ret[H::KEY_ID_SIZE.get()..H::KEY_ID_SIZE.get() + alpha_len].copy_from_slice(self.alpha.as_ref());
        ret[H::KEY_ID_SIZE.get() + alpha_len..H::KEY_ID_SIZE.get() + alpha_len + RoutingInfo::<H>::SIZE]
            .copy_from_slice(self.header.as_ref());
        ret[H::KEY_ID_SIZE.get() + alpha_len + RoutingInfo::<H>::SIZE
            ..H::KEY_ID_SIZE.get() + alpha_len + RoutingInfo::<H>::SIZE + SecretKey16::LENGTH]
            .copy_from_slice(self.sender_key.as_ref());
        ret[H::KEY_ID_SIZE.get() + alpha_len + RoutingInfo::<H>::SIZE + SecretKey16::LENGTH
            ..H::KEY_ID_SIZE.get()
                + alpha_len
                + RoutingInfo::<H>::SIZE
                + SecretKey16::LENGTH
                + H::SURB_RECEIVER_DATA_SIZE]
            .copy_from_slice(self.additional_data_receiver.as_ref());

        ret.into_boxed_slice()
    }

    /// Computes Keccak256 hash of the SURB.
    ///
    /// The given `context` is appended to the input.
    pub fn get_hash(&self, context: &[u8]) -> Hash {
        Hash::create(&[
            self.first_relayer.as_ref(),
            self.alpha.as_ref(),
            self.sender_key.as_ref(),
            self.header.as_ref(),
            context,
        ])
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> Clone for SURB<S, H>
where
    H::KeyId: Clone,
    H::SurbReceiverData: Clone,
{
    fn clone(&self) -> Self {
        Self {
            first_relayer: self.first_relayer.clone(),
            alpha: self.alpha.clone(),
            header: self.header.clone(),
            sender_key: self.sender_key.clone(),
            additional_data_receiver: self.additional_data_receiver.clone(),
        }
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> std::fmt::Debug for SURB<S, H>
where
    H::KeyId: std::fmt::Debug,
    H::SurbReceiverData: std::fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SURB")
            .field("first_relayer", &self.first_relayer)
            .field("alpha", &self.alpha)
            .field("header", &self.header)
            .field("sender_key", &"<redacted>")
            .field("additional_data_receiver", &self.additional_data_receiver)
            .finish()
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> PartialEq for SURB<S, H>
where
    H::KeyId: PartialEq,
    H::SurbReceiverData: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.first_relayer.eq(&other.first_relayer)
            && self.alpha.eq(&other.alpha)
            && self.header.eq(&other.header)
            && self.sender_key.ct_eq(&other.sender_key).into()
            && self.additional_data_receiver.eq(&other.additional_data_receiver)
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> Eq for SURB<S, H>
where
    H::KeyId: Eq,
    H::SurbReceiverData: Eq,
{
}

impl<'a, S: SphinxSuite, H: SphinxHeaderSpec> TryFrom<&'a [u8]> for SURB<S, H> {
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let alpha = <S::G as GroupElement<S::E>>::AlphaLen::USIZE;

        if value.len() == Self::SIZE {
            Ok(Self {
                first_relayer: value[0..H::KEY_ID_SIZE.get()]
                    .try_into()
                    .map_err(|_| GeneralError::ParseError("SURB.first_relayer".into()))?,
                alpha: Array::<u8, <S::G as GroupElement<S::E>>::AlphaLen>::try_from(
                    &value[H::KEY_ID_SIZE.get()..H::KEY_ID_SIZE.get() + alpha],
                )
                .map_err(|_| GeneralError::ParseError("SURB.alpha".into()))?,
                header: value[H::KEY_ID_SIZE.get() + alpha..H::KEY_ID_SIZE.get() + alpha + RoutingInfo::<H>::SIZE]
                    .try_into()
                    .map_err(|_| GeneralError::ParseError("SURB.header".into()))?,
                sender_key: value[H::KEY_ID_SIZE.get() + alpha + RoutingInfo::<H>::SIZE
                    ..H::KEY_ID_SIZE.get() + alpha + RoutingInfo::<H>::SIZE + SecretKey16::LENGTH]
                    .try_into()
                    .map_err(|_| GeneralError::ParseError("SURB.sender_key".into()))?,
                additional_data_receiver: value
                    [H::KEY_ID_SIZE.get() + alpha + RoutingInfo::<H>::SIZE + SecretKey16::LENGTH..]
                    .try_into()
                    .map_err(|_| GeneralError::ParseError("SURB.additional_data_receiver".into()))?,
            })
        } else {
            Err(GeneralError::ParseError("SURB::SIZE".into()))
        }
    }
}

/// Entry stored locally by the [`SURB`] creator to allow decryption
/// of received responses.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReplyOpener {
    /// Encryption key the other party should use to encrypt the data for us.
    pub sender_key: SecretKey16,
    /// Shared secrets for nodes along the return path.
    pub shared_secrets: Vec<SharedSecret>,
}

impl std::fmt::Debug for ReplyOpener {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReplyOpener")
            .field("sender_key", &"<redacted>")
            .field("shared_secrets", &format!("{} <redacted>", self.shared_secrets.len()))
            .finish()
    }
}

/// Creates a pair of [`SURB`] and [`ReplyOpener`].
///
/// The former is sent to the other party, the latter is kept locally.
pub fn create_surb<S: SphinxSuite, H: SphinxHeaderSpec>(
    shared_keys: SharedKeys<S::E, S::G>,
    path: &[H::KeyId],
    additional_data_relayer: &[H::RelayerData],
    receiver_data: H::PacketReceiverData,
    additional_data_receiver: H::SurbReceiverData,
) -> hopr_types::crypto::errors::Result<(SURB<S, H>, ReplyOpener)>
where
    H::KeyId: Copy,
{
    let header = RoutingInfo::<H>::new(
        path,
        &shared_keys.secrets,
        additional_data_relayer,
        &receiver_data,
        true,
        false,
    )?;

    let sender_key = SecretKey16::random();

    let surb = SURB {
        sender_key: sender_key.clone(),
        header,
        first_relayer: *path.first().ok_or(CryptoError::InvalidInputValue("path is empty"))?,
        additional_data_receiver,
        alpha: shared_keys.alpha,
    };

    let reply_opener = ReplyOpener {
        sender_key: sender_key.clone(),
        shared_secrets: shared_keys.secrets,
    };

    Ok((surb, reply_opener))
}

/// Reconstructs the [`ReplyOpener`] belonging to a [`SURB`] from the private keys of the nodes on
/// its return path.
///
/// The opener a [`create_surb`] call produces never leaves the process that made it, and nothing
/// on the wire carries it: a reply is readable only by the SURB's creator. This function recovers
/// the same value from the SURB alone, by walking its return path the way each relay on that path
/// will — [`SharedKeys::forward_transform`] with the hop's private key, then
/// [`forward_header`](super::routing::forward_header) to reach the next hop — and collecting the
/// per-hop shared secrets. `sender_key` needs no recovery; it travels inside the SURB.
///
/// # When this is the right tool
///
/// Only for an observer that legitimately holds every private key on the return path and wants to
/// read traffic it is not the recipient of — a protocol dissector run against a test cluster.
/// A node decrypting its own replies must use the opener it kept, which is both cheaper and the
/// only option it has: it does not hold the relays' keys. Possession of those keys is the entire
/// security assumption here, and this function does not weaken it — anyone able to call it could
/// already decrypt every hop of the return path directly.
///
/// # Resolving the hops
///
/// `candidates_for` is asked, for each hop, which keypairs that key identifier might name. It may
/// return several, and the right one is then *identified* rather than guessed: the header carries
/// an authentication tag over its own contents, so transforming it with the wrong key fails with
/// [`CryptoError::TagMismatch`] and only the correct key gets through.
///
/// That matters because a key identifier is assigned by the chain and cannot be derived from a
/// key, so an observer may know every node's keypair without knowing which identifier stands for
/// which. Offering all of them and letting the tag decide costs one transformation per candidate
/// on the first hop and turns an unresolvable identifier into a resolvable one.
///
/// # Arguments
/// * `surb` - the SURB whose opener should be reconstructed.
/// * `candidates_for` - the keypairs a key identifier on the return path might name. An empty
///   iterator fails the reconstruction, as does one containing no matching key: a missing secret
///   leaves the payload undecryptable either way.
///
/// # Errors
/// [`CryptoError::InvalidInputValue`] if no candidate matches a hop, or if the path is longer than
/// [`SphinxHeaderSpec::MAX_HOPS`] — which means the header is not one this specification could
/// have produced.
pub fn reply_opener_from_surb<'a, S, H, F, I>(
    surb: &SURB<S, H>,
    mut candidates_for: F,
) -> hopr_types::crypto::errors::Result<ReplyOpener>
where
    S: SphinxSuite,
    H: SphinxHeaderSpec,
    S::P: 'a,
    F: FnMut(&H::KeyId) -> I,
    I: IntoIterator<Item = &'a S::P>,
    for<'b> &'b Alpha<<S::G as GroupElement<S::E>>::AlphaLen>: From<&'b <S::P as Keypair>::Public>,
{
    let mut alpha = surb.alpha.clone();
    let mut header = surb.header.as_ref().to_vec();
    let mut next_hop = surb.first_relayer.clone();

    // `MAX_HOPS` entries plus the final one. A header that never reports `Final` within that many
    // transformations cannot have been built by `RoutingInfo::new`, so bounding the walk turns a
    // malformed SURB into an error instead of a loop.
    let mut shared_secrets = Vec::with_capacity(H::MAX_HOPS.get());

    for _ in 0..H::MAX_HOPS.get() {
        let mut matched = None;

        for keypair in candidates_for(&next_hop) {
            let Ok((next_alpha, secret)) =
                SharedKeys::<S::E, S::G>::forward_transform(&alpha, &keypair.into(), keypair.public().into())
            else {
                continue;
            };

            // Each candidate gets its own copy: `forward_header` decrypts in place, so a failed
            // attempt would otherwise leave the header unusable for the next one.
            let mut attempt = header.clone();
            if let Ok(forwarded) = super::routing::forward_header::<H>(&secret, &mut attempt) {
                matched = Some((next_alpha, secret, forwarded));
                break;
            }
        }

        let (next_alpha, secret, forwarded) = matched.ok_or(CryptoError::InvalidInputValue(
            "no candidate keypair matches a node on the SURB's return path",
        ))?;
        shared_secrets.push(secret);

        match forwarded {
            super::routing::ForwardedHeader::Relayed {
                next_header,
                next_node,
                ..
            } => {
                alpha = next_alpha;
                header = next_header.as_ref().to_vec();
                next_hop = next_node;
            }
            super::routing::ForwardedHeader::Final { .. } => {
                return Ok(ReplyOpener {
                    sender_key: surb.sender_key.clone(),
                    shared_secrets,
                });
            }
        }
    }

    Err(CryptoError::InvalidInputValue(
        "SURB return path is longer than the maximum number of hops",
    ))
}

#[cfg(test)]
mod tests {
    use hopr_types::crypto_random::Randomizable;

    use super::{super::tests::*, *};

    #[allow(type_alias_bounds)]
    pub type HeaderSpec<S: SphinxSuite> = TestSpec<<S::P as Keypair>::Public, 4, 66>;

    fn generate_surbs<S: SphinxSuite>(keypairs: Vec<S::P>) -> anyhow::Result<(SURB<S, HeaderSpec<S>>, ReplyOpener)>
    where
        <<S as SphinxSuite>::P as Keypair>::Public: Copy,
        for<'a> &'a Alpha<<<S as SphinxSuite>::G as GroupElement<<S as SphinxSuite>::E>>::AlphaLen>:
            From<&'a <<S as SphinxSuite>::P as Keypair>::Public>,
    {
        let pub_keys = keypairs.iter().map(|kp| *kp.public()).collect::<Vec<_>>();
        let shares = S::new_shared_keys(&pub_keys)?;

        Ok(create_surb::<S, HeaderSpec<S>>(
            shares,
            &pub_keys,
            &[Default::default(); 4],
            SimplePseudonym::random(),
            Default::default(),
        )?)
    }

    // Mutually exclusive cfg to prevent type alias collision under --all-features.
    // Priority order: ed25519 > secp256k1 > x25519.
    #[cfg(feature = "ed25519")]
    use crate::sphinx::ec_groups::Ed25519Suite as CurrentSuite;
    #[cfg(all(feature = "secp256k1", not(feature = "ed25519")))]
    use crate::sphinx::ec_groups::Secp256k1Suite as CurrentSuite;
    #[cfg(all(feature = "x25519", not(any(feature = "ed25519", feature = "secp256k1"))))]
    use crate::sphinx::ec_groups::X25519Suite as CurrentSuite;

    #[test]
    fn surb_serialize_deserialize() -> anyhow::Result<()> {
        let (surb_1, _) = generate_surbs::<CurrentSuite>((0..3).map(|_| OffchainKeypair::random()).collect())?;

        let surb_1_enc = surb_1.into_boxed();

        let surb_2 = SURB::<CurrentSuite, HeaderSpec<CurrentSuite>>::try_from(surb_1_enc.as_ref())?;

        assert_eq!(surb_1_enc, surb_2.into_boxed());

        Ok(())
    }

    #[parameterized::parameterized(hops = { 1, 2, 3, 4 })]
    fn reconstructed_reply_opener_should_equal_the_one_kept_by_the_surb_creator(hops: usize) {
        (|| -> anyhow::Result<()> {
            let keypairs = (0..hops).map(|_| OffchainKeypair::random()).collect::<Vec<_>>();
            let (surb, kept) = generate_surbs::<CurrentSuite>(keypairs.clone())?;

            let reconstructed = reply_opener_from_surb(&surb, |id| keypairs.iter().find(|kp| kp.public() == id))?;

            assert_eq!(
                reconstructed.sender_key.ct_eq(&kept.sender_key).unwrap_u8(),
                1,
                "sender key must be recovered from the SURB itself"
            );
            assert_eq!(
                reconstructed.shared_secrets.len(),
                kept.shared_secrets.len(),
                "must recover one shared secret per return-path hop"
            );
            for (i, (recovered, expected)) in reconstructed
                .shared_secrets
                .iter()
                .zip(kept.shared_secrets.iter())
                .enumerate()
            {
                assert_eq!(
                    recovered.ct_eq(expected).unwrap_u8(),
                    1,
                    "shared secret for hop {i} must match the one the creator kept"
                );
            }

            Ok(())
        })()
        .expect("reconstruction must succeed for a well-formed SURB");
    }

    #[test]
    fn should_identify_the_right_hop_when_offered_every_keypair() -> anyhow::Result<()> {
        // What an observer holding the keys but not the chain's identifier table has to do: offer
        // all of them and let the header's authentication tag pick the one that fits.
        let keypairs = (0..3).map(|_| OffchainKeypair::random()).collect::<Vec<_>>();
        let (surb, kept) = generate_surbs::<CurrentSuite>(keypairs.clone())?;

        let reconstructed = reply_opener_from_surb(&surb, |_| keypairs.iter())?;

        assert_eq!(
            reconstructed.shared_secrets.len(),
            kept.shared_secrets.len(),
            "every hop must be identified without being told which key names it"
        );
        for (i, (recovered, expected)) in reconstructed
            .shared_secrets
            .iter()
            .zip(kept.shared_secrets.iter())
            .enumerate()
        {
            assert_eq!(
                recovered.ct_eq(expected).unwrap_u8(),
                1,
                "hop {i} was identified as the wrong node"
            );
        }

        Ok(())
    }

    #[test]
    fn reconstructing_a_reply_opener_should_fail_when_a_return_path_key_is_missing() -> anyhow::Result<()> {
        let keypairs = (0..3).map(|_| OffchainKeypair::random()).collect::<Vec<_>>();
        let (surb, _) = generate_surbs::<CurrentSuite>(keypairs.clone())?;

        // Everything but the last hop: a walk that gets partway and then cannot continue must
        // report that rather than return a short secret list that would silently mis-decrypt.
        let known = &keypairs[..keypairs.len() - 1];

        assert!(
            reply_opener_from_surb(&surb, |id| known.iter().find(|kp| kp.public() == id)).is_err(),
            "a return path with an unknown hop must not yield an opener"
        );

        Ok(())
    }
}
