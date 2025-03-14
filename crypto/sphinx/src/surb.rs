use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use typenum::Unsigned;

use crate::routing::{RoutingInfo, SphinxHeaderSpec};
use crate::shared_keys::{Alpha, GroupElement, SharedKeys, SharedSecret, SphinxSuite};

/// Single Use Reply Block
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
    pub const SIZE: usize = H::KEY_ID_SIZE.get()
        + <S::G as GroupElement<S::E>>::AlphaLen::USIZE
        + RoutingInfo::<H>::SIZE
        + SecretKey16::LENGTH
        + H::SURB_RECEIVER_DATA_SIZE;

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
                sender_key: value[H::KEY_ID_SIZE.get() + alpha + H::HEADER_LEN + H::TAG_SIZE
                    ..H::KEY_ID_SIZE.get() + alpha + H::HEADER_LEN + H::TAG_SIZE + SecretKey16::LENGTH]
                    .try_into()
                    .map_err(|_| GeneralError::ParseError("SURB.sender_key".into()))?,
                additional_data_receiver: value
                    [H::KEY_ID_SIZE.get() + alpha + H::HEADER_LEN + H::TAG_SIZE + SecretKey16::LENGTH..]
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
pub struct ReplyOpener {
    /// Encryption key the other party should use to encrypt the data for us.
    pub sender_key: SecretKey16,
    /// Shared keys for nodes along the return path.
    pub shared_keys: Vec<SharedSecret>,
}

/// Creates a pair of [`SURB`] and [`ReplyOpener`].
///
/// The former is sent to the other party, the latter is kept locally.
pub fn create_surb<S: SphinxSuite, H: SphinxHeaderSpec>(
    shared_keys: SharedKeys<S::E, S::G>,
    path: &[H::KeyId],
    additional_data_relayer: &[H::RelayerData],
    pseudonym: H::Pseudonym,
    additional_data_receiver: H::SurbReceiverData,
) -> hopr_crypto_types::errors::Result<(SURB<S, H>, ReplyOpener)>
where
    H::KeyId: Copy,
{
    let header = RoutingInfo::<H>::new(path, &shared_keys.secrets, additional_data_relayer, pseudonym, true)?;

    let sender_key = SecretKey16::random();

    let surb = SURB {
        sender_key: sender_key.clone(),
        header,
        first_relayer: path[0],
        additional_data_receiver,
        alpha: shared_keys.alpha,
    };

    let local_surb = ReplyOpener {
        sender_key: sender_key.clone(),
        shared_keys: shared_keys.secrets,
    };

    Ok((surb, local_surb))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ec_groups::X25519Suite;
    use crate::tests::*;

    #[allow(type_alias_bounds)]
    pub type HeaderSpec<S: SphinxSuite> = TestSpec<<S::P as Keypair>::Public, 4, 66>;

    fn generate_surbs<S: SphinxSuite>(keypairs: Vec<S::P>) -> anyhow::Result<(SURB<S, HeaderSpec<S>>, ReplyOpener)>
    where
        <<S as SphinxSuite>::P as Keypair>::Public: Copy,
    {
        let pub_keys = keypairs.iter().map(|kp| kp.public().clone()).collect::<Vec<_>>();
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
