use crate::{HoprSphinxHeaderSpec, HoprSphinxSuite};
use hopr_crypto_sphinx::prelude::{SphinxHeaderSpec, SphinxSuite, SURB};
use hopr_primitive_types::prelude::GeneralError;
use std::marker::PhantomData;

/// Additional encoding of a packet message that can be preceded by a number of [`SURBs`](SURB).
pub struct PacketMessage<S: SphinxSuite = HoprSphinxSuite, H: SphinxHeaderSpec = HoprSphinxHeaderSpec>(
    Box<[u8]>,
    PhantomData<(S, H)>,
);

/// Convenience alias for HOPR specific [`PacketMessage`].
pub type HoprPacketMessage = PacketMessage<HoprSphinxSuite, HoprSphinxHeaderSpec>;

/// Individual parts of a [`PacketMessage`]: SURBs and the actual message.
pub type PacketParts<S, H> = (Vec<SURB<S, H>>, Box<[u8]>);

impl<S: SphinxSuite, H: SphinxHeaderSpec> PacketMessage<S, H> {
    /// Converts this instance into [`PacketParts`].
    pub fn try_into_parts(self) -> Result<PacketParts<S, H>, GeneralError> {
        let num_surbs = self.0[0] as usize;
        if num_surbs > 0 {
            let surb_end = num_surbs * SURB::<S, H>::SIZE;
            if surb_end >= self.0.len() {
                return Err(GeneralError::ParseError("HoprPacketMessage.num_surbs".into()));
            }

            let mut data = self.0.into_vec();

            let surb_data = data.drain(0..=surb_end).skip(1).collect::<Vec<_>>();

            let surbs = surb_data
                .as_slice()
                .chunks_exact(SURB::<S, H>::SIZE)
                .map(SURB::<S, H>::try_from)
                .collect::<Result<Vec<_>, _>>()?;

            Ok((surbs, data.into_boxed_slice()))
        } else {
            let mut data = self.0.into_vec();
            data.remove(0);
            Ok((Vec::with_capacity(0), data.into_boxed_slice()))
        }
    }

    /// Allocates a new instance from the given parts.
    pub fn from_parts(surbs: Vec<SURB<S, H>>, payload: &[u8]) -> Self {
        let mut ret = Vec::with_capacity(1 + surbs.len() * SURB::<S, H>::SIZE + payload.len());
        ret.push(surbs.len() as u8);
        for surb in surbs.into_iter().take(u8::MAX as usize).map(|s| s.into_boxed()) {
            ret.extend_from_slice(surb.as_ref());
        }
        ret.extend_from_slice(payload);
        Self(ret.into_boxed_slice(), PhantomData)
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> TryFrom<Box<[u8]>> for PacketMessage<S, H> {
    type Error = GeneralError;
    fn try_from(value: Box<[u8]>) -> Result<Self, Self::Error> {
        if !value.is_empty() {
            Ok(Self(value, PhantomData))
        } else {
            Err(GeneralError::ParseError("HoprPacketMessage".into()))
        }
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> AsRef<[u8]> for PacketMessage<S, H> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::anyhow;
    use bimap::BiHashMap;
    use hex_literal::hex;
    use hopr_crypto_sphinx::prelude::*;
    use hopr_crypto_types::prelude::*;
    use hopr_primitive_types::prelude::*;

    use crate::por::{ProofOfRelayString, ProofOfRelayValues};
    use crate::{HoprSphinxHeaderSpec, HoprSphinxSuite};

    lazy_static::lazy_static! {
        static ref PEERS: [(ChainKeypair, OffchainKeypair); 4] = [
            (hex!("a7c486ceccf5ab53bd428888ab1543dc2667abd2d5e80aae918da8d4b503a426"), hex!("5eb212d4d6aa5948c4f71574d45dad43afef6d330edb873fca69d0e1b197e906")),
            (hex!("9a82976f7182c05126313bead5617c623b93d11f9f9691c87b1a26f869d569ed"), hex!("e995db483ada5174666c46bafbf3628005aca449c94ebdc0c9239c3f65d61ae0")),
            (hex!("ca4bdfd54a8467b5283a0216288fdca7091122479ccf3cfb147dfa59d13f3486"), hex!("9dec751c00f49e50fceff7114823f726a0425a68a8dc6af0e4287badfea8f4a4")),
            (hex!("e306ebfb0d01d0da0952c9a567d758093a80622c6cb55052bf5f1a6ebd8d7b5c"), hex!("9a82976f7182c05126313bead5617c623b93d11f9f9691c87b1a26f869d569ed"))
        ].map(|(p1,p2)| (ChainKeypair::from_secret(&p1).expect("lazy static keypair should be valid"), OffchainKeypair::from_secret(&p2).expect("lazy static keypair should be valid")));

        static ref MAPPER: bimap::BiMap<KeyIdent, OffchainPublicKey> = PEERS
            .iter()
            .enumerate()
            .map(|(i, (_, k))| (KeyIdent::from(i as u32), k.public().clone()))
            .collect::<BiHashMap<_, _>>();
    }

    #[test]
    fn hopr_packet_message_message_only() -> anyhow::Result<()> {
        let test_msg = b"test message";
        let (surbs, msg) = HoprPacketMessage::from_parts(vec![], test_msg).try_into_parts()?;
        assert_eq!(surbs.len(), 0);
        assert_eq!(msg.as_ref(), test_msg);

        Ok(())
    }

    #[test]
    fn hopr_packet_message_surbs_only() -> anyhow::Result<()> {
        let path = PEERS.iter().map(|(_, k)| k.public().clone()).collect::<Vec<_>>();
        let path_ids =
            <BiHashMap<_, _> as KeyIdMapper<HoprSphinxSuite, HoprSphinxHeaderSpec>>::map_keys_to_ids(&*MAPPER, &path)
                .into_iter()
                .map(|v| v.ok_or(anyhow!("missing id")))
                .collect::<Result<Vec<_>, _>>()?;
        let pseudonym = SimplePseudonym::random();

        let surbs_1 = (0..2)
            .into_iter()
            .map(|_| {
                let shared_keys = HoprSphinxSuite::new_shared_keys(&path)?;
                let por_strings = ProofOfRelayString::from_shared_secrets(&shared_keys.secrets)?;
                let por_values = ProofOfRelayValues::new(
                    &shared_keys.secrets[0],
                    shared_keys.secrets.get(1),
                    shared_keys.secrets.len() as u8,
                )?;

                create_surb::<HoprSphinxSuite, HoprSphinxHeaderSpec>(
                    shared_keys,
                    &path_ids,
                    &por_strings,
                    pseudonym,
                    por_values,
                )
                .map(|(s, _)| s)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let (surbs_2, msg) = HoprPacketMessage::from_parts(surbs_1.clone(), &[]).try_into_parts()?;

        assert_eq!(surbs_2.len(), surbs_1.len());
        for (surb_1, surb_2) in surbs_1.into_iter().zip(surbs_2.into_iter()) {
            assert_eq!(surb_1.into_boxed(), surb_2.into_boxed());
        }

        assert!(msg.is_empty());

        Ok(())
    }

    #[test]
    fn hopr_packet_message_surbs_and_msg() -> anyhow::Result<()> {
        let test_msg = b"test message";
        let path = PEERS.iter().map(|(_, k)| k.public().clone()).collect::<Vec<_>>();
        let path_ids =
            <BiHashMap<_, _> as KeyIdMapper<HoprSphinxSuite, HoprSphinxHeaderSpec>>::map_keys_to_ids(&MAPPER, &path)
                .into_iter()
                .map(|v| v.ok_or(anyhow!("missing id")))
                .collect::<Result<Vec<_>, _>>()?;
        let pseudonym = SimplePseudonym::random();

        let surbs_1 = (0..2)
            .into_iter()
            .map(|_| {
                let shared_keys = HoprSphinxSuite::new_shared_keys(&path)?;
                let por_strings = ProofOfRelayString::from_shared_secrets(&shared_keys.secrets)?;
                let por_values = ProofOfRelayValues::new(
                    &shared_keys.secrets[0],
                    shared_keys.secrets.get(1),
                    shared_keys.secrets.len() as u8,
                )?;

                create_surb::<HoprSphinxSuite, HoprSphinxHeaderSpec>(
                    shared_keys,
                    &path_ids,
                    &por_strings,
                    pseudonym,
                    por_values,
                )
                .map(|(s, _)| s)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let (surbs_2, msg) = HoprPacketMessage::from_parts(surbs_1.clone(), test_msg).try_into_parts()?;

        assert_eq!(surbs_2.len(), surbs_1.len());
        for (surb_1, surb_2) in surbs_1.into_iter().zip(surbs_2.into_iter()) {
            assert_eq!(surb_1.into_boxed(), surb_2.into_boxed());
        }

        assert_eq!(msg.as_ref(), test_msg);

        Ok(())
    }
}
