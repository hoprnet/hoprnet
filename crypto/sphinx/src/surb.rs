use hopr_crypto_types::prelude::SecretKey;
use hopr_crypto_types::types::Pseudonym;
use hopr_primitive_types::prelude::GeneralError;
use std::marker::PhantomData;
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
    pub sender_key: SecretKey,
    /// Additional data for the SURB receiver.
    pub additional_data_receiver: H::SurbReceiverData,
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> SURB<S, H> {
    pub const SIZE: usize = H::KEY_ID_SIZE.get()
        + <S::G as GroupElement<S::E>>::AlphaLen::USIZE
        + RoutingInfo::<H>::SIZE
        + H::TAG_SIZE
        + SecretKey::LENGTH
        + H::SURB_RECEIVER_DATA_SIZE;

    pub fn into_boxed(self) -> Box<[u8]> {
        let mut ret = vec![0u8; Self::SIZE];
        ret.extend_from_slice(self.first_relayer.as_ref());
        ret.extend_from_slice(self.alpha.as_ref());
        ret.extend_from_slice(self.header.routing_information.as_ref());
        ret.extend_from_slice(self.header.mac.as_ref());
        ret.extend_from_slice(self.sender_key.as_ref());
        ret.extend_from_slice(self.additional_data_receiver.as_ref());

        debug_assert_eq!(ret.len(), Self::SIZE);

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
                    .map_err(|_| GeneralError::ParseError("SURB".into()))?,
                alpha: Alpha::<<S::G as GroupElement<S::E>>::AlphaLen>::from_slice(
                    &value[H::KEY_ID_SIZE.get()..H::KEY_ID_SIZE.get() + alpha],
                )
                .clone(),
                header: RoutingInfo::<H> {
                    routing_information: value
                        [H::KEY_ID_SIZE.get() + alpha..H::KEY_ID_SIZE.get() + alpha + H::HEADER_LEN]
                        .into(),
                    mac: value[H::KEY_ID_SIZE.get() + alpha + H::HEADER_LEN
                        ..H::KEY_ID_SIZE.get() + alpha + H::HEADER_LEN + H::TAG_SIZE]
                        .into(),
                    _h: PhantomData,
                },
                sender_key: value[H::KEY_ID_SIZE.get() + alpha + H::HEADER_LEN + H::TAG_SIZE
                    ..H::KEY_ID_SIZE.get() + alpha + H::HEADER_LEN + H::TAG_SIZE + SecretKey::LENGTH]
                    .try_into()
                    .map_err(|_| GeneralError::ParseError("SURB".into()))?,
                additional_data_receiver: value
                    [H::KEY_ID_SIZE.get() + alpha + H::HEADER_LEN + H::TAG_SIZE + SecretKey::LENGTH..]
                    .try_into()
                    .map_err(|_| GeneralError::ParseError("SURB".into()))?,
            })
        } else {
            Err(GeneralError::ParseError("SURB".into()))
        }
    }
}

/// Entry stored locally by the [`SURB`] creator to allow decryption
/// of received responses.
#[derive(Clone)]
pub struct LocalSURBEntry {
    /// Encryption key the other party should use to encrypt the data for us.
    pub sender_key: SecretKey,
    /// Shared keys for nodes along the return path.
    pub shared_keys: Vec<SharedSecret>,
}

/// Creates a pair of [`SURB`] and [`LocalSURBEntry`].
///
/// The former is sent to the other party, the latter is kept locally.
pub fn create_surb<S: SphinxSuite, H: SphinxHeaderSpec>(
    shared_keys: SharedKeys<S::E, S::G>,
    path: &[H::KeyId],
    additional_data_relayer: &[H::RelayerData],
    additional_data_last_hop: H::LastHopData,
    additional_data_receiver: H::SurbReceiverData,
) -> hopr_crypto_types::errors::Result<(SURB<S, H>, LocalSURBEntry)>
where
    H::KeyId: Copy,
{
    let header = RoutingInfo::<H>::new(
        path,
        &shared_keys.secrets,
        additional_data_relayer,
        additional_data_last_hop,
    )?;

    let sender_key = SecretKey::random();

    let surb = SURB {
        sender_key: sender_key.clone(),
        header,
        first_relayer: path[0],
        additional_data_receiver,
        alpha: shared_keys.alpha,
    };

    let local_surb = LocalSURBEntry {
        sender_key: sender_key.clone(),
        shared_keys: shared_keys.secrets,
    };

    Ok((surb, local_surb))
}

/// Represents an additional message delivered to the recipient of a Sphinx packet.
///
/// This message serves as an indication of what is included in the packet payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SphinxRecipientMessage<P: Pseudonym> {
    /// The packet payload contains only data from a forward message.
    DataOnly,
    /// The packet payload contains only data from a reply message.
    ReplyOnly(P),
    /// The packet payload contains a SURB followed by data.
    DataWithSurb(P),
    /// The packet contains only multiple SURBs with no more data.
    SurbsOnly(u8, P),
}

impl<P: Pseudonym> SphinxRecipientMessage<P> {
    pub fn num_surbs(&self) -> u8 {
        match self {
            Self::DataOnly => 0,
            Self::ReplyOnly(_) => 0,
            Self::DataWithSurb(_) => 1,
            Self::SurbsOnly(n, _) => *n,
        }
    }
}
