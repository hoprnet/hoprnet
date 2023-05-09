use multiaddr::{Multiaddr, Protocol};
use core_crypto::types::PublicKey;
use utils_types::errors::GeneralError::ParseError;
use utils_types::primitives::{Address, U256};
use utils_types::traits::{BinarySerializable, PeerIdLike};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct AccountEntry {
    pub public_key: PublicKey,
    multiaddr: Option<Multiaddr>,
    pub updated_block: u32
}

impl AccountEntry {
    const MAX_MULTI_ADDR_LENGTH: usize = 200;
    const MA_LENGTH_PREFIX: usize = std::mem::size_of::<u32>();

    pub fn get_address(&self) -> Address {
        self.public_key.to_address()
    }

    pub fn get_peer_id_str(&self) -> String {
        self.public_key.to_peerid_str()
    }

    pub fn get_multiaddress_str(&self) -> Option<String> {
        self.multiaddr.map(|m| m.to_string())
    }

    pub fn has_announced(&self) -> bool {
        self.multiaddr.is_some()
    }

    pub fn contains_routing_info(&self) -> bool {
        match &self.multiaddr {
            None => false,
            Some(ma) => {
                ma.protocol_stack()
                    .find(|p| p == "ip4" || p == "ip6")
                    .is_some() &&
                ma.protocol_stack()
                    .find(|p| p == "tcp")
                    .is_some()
            }
        }
    }
}

impl BinarySerializable<'_> for AccountEntry {
    const SIZE: usize = PublicKey::SIZE_UNCOMPRESSED + Self::MA_LENGTH_PREFIX +
        Self::MAX_MULTI_ADDR_LENGTH + 4;

    fn deserialize(data: &[u8]) -> utils_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut buf = data.to_vec();
            let public_key = PublicKey::deserialize(buf.drain(..PublicKey::SIZE_UNCOMPRESSED).as_ref())?;
            let ma_len = u32::from_be_bytes(buf.drain(..MA_LENGTH_PREFIX).as_ref().try_into().unwrap());
            let multiaddr = if ma_len > 0 {
                Some(Multiaddr::try_from(buf.drain(..ma_len).collect::<Vec<u8>>()).map_err(|_| ParseError)?)
            } else {
                None
            };
            let updated_block = u32::from_be_bytes(buf.drain(..std::mem::size_of::<u32>()).as_ref().try_into().unwrap());
            Ok(Self {
                public_key, multiaddr, updated_block
            })
        } else {
            Err(ParseError)
        }
    }

    fn serialize(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.public_key.serialize(false));
        match &self.multiaddr {
            None => {
                ret.extend_from_slice(&(0 as u32).to_be_bytes());
                ret.extend_from_slice(&[0u8; Self::MAX_MULTI_ADDR_LENGTH]);
            },
            Some(ma) => {
                let ma_bytes = ma.to_vec();
                assert!(ma_bytes.len() <= Self::MAX_MULTI_ADDR_LENGTH, "multi address too long");
                ret.extend_from_slice(&(ma_bytes.len() as u32).to_be_bytes());
                ret.extend_from_slice(&ma_bytes);
            }
        }
        ret.extend_from_slice(&self.updated_block.to_be_bytes());
        ret.into_boxed_slice()
    }
}