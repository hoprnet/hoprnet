use std::fmt::Formatter;
use subtle::ConstantTimeEq;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use typenum::Unsigned;

use crate::{
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
        self.first_relayer.eq(&other.first_relayer) &&
            self.alpha.eq(&other.alpha) &&
            self.header.eq(&other.header) &&
            self.sender_key.ct_eq(&other.sender_key).into() &&
            self.additional_data_receiver.eq(&other.additional_data_receiver)
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> Eq for SURB<S, H>
where
    H::KeyId: Eq,
    H::SurbReceiverData: Eq
{ }

impl<'a, S: SphinxSuite, H: SphinxHeaderSpec> TryFrom<&'a [u8]> for SURB<S, H> {
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let alpha = <S::G as GroupElement<S::E>>::AlphaLen::USIZE;

        if value.len() == Self::SIZE {
            Ok(Self {
                first_relayer: value[0..H::KEY_ID_SIZE.get()]
                    .try_into()
                    .map_err(|_| GeneralError::ParseError("SURB.first_relayer".into()))?,
                alpha: Alpha::<<S::G as GroupElement<S::E>>::AlphaLen>::from_slice(
                    &value[H::KEY_ID_SIZE.get()..H::KEY_ID_SIZE.get() + alpha],
                )
                .clone(),
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
) -> hopr_crypto_types::errors::Result<(SURB<S, H>, ReplyOpener)>
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

#[cfg(test)]
mod tests {
    use hopr_crypto_random::Randomizable;

    use super::*;
    use crate::{ec_groups::X25519Suite, tests::*};

    #[allow(type_alias_bounds)]
    pub type HeaderSpec<S: SphinxSuite> = TestSpec<<S::P as Keypair>::Public, 4, 66>;

    fn generate_surbs<S: SphinxSuite>(keypairs: Vec<S::P>) -> anyhow::Result<(SURB<S, HeaderSpec<S>>, ReplyOpener)>
    where
        <<S as SphinxSuite>::P as Keypair>::Public: Copy,
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

    #[test]
    fn surb_x25519_serialize_deserialize() -> anyhow::Result<()> {
        let (surb_1, _) = generate_surbs::<X25519Suite>((0..3).map(|_| OffchainKeypair::random()).collect())?;

        let surb_1_enc = surb_1.into_boxed();

        let surb_2 = SURB::<X25519Suite, HeaderSpec<X25519Suite>>::try_from(surb_1_enc.as_ref())?;

        assert_eq!(surb_1_enc, surb_2.into_boxed());

        Ok(())
    }
}
