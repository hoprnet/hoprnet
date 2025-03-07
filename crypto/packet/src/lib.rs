//! # core-packet
//!
//! This crate contains the main packet processing functionality for the HOPR protocol.
//! It implements the following important protocol building blocks:
//!
//! - SPHINX packet format (module [packet])
//! - Proof of Relay (module [por])
//!
//! Finally, it also implements a utility function which is used to validate tickets (module `validation`).
//! The ticket validation functionality is dependent on `chain-db`.
//!
//! The currently used implementation is selected using the `CurrentSphinxSuite` type in the `packet` module.
//!
//! The implementation can be easily extended for different elliptic curves (or even arithmetic multiplicative groups).
//! In particular, as soon as there's way to represent `Ed448` PeerIDs, it would be easy to create e.g. `X448Suite`.
//!

use hopr_crypto_sphinx::routing::SphinxHeaderSpec;
use hopr_crypto_sphinx::shared_keys::SphinxSuite;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use std::marker::PhantomData;

/// Implements the overlay packet intermediary object.
pub mod chain;
/// Lists all errors in this crate.
pub mod errors;
/// Implements SPHINX packet format.
pub mod packet;
/// Implements the Proof of Relay.
pub mod por;
/// Implements ticket validation logic.
pub mod validation;

/// Currently used public key cipher suite for Sphinx.
pub type CurrentSphinxSuite = hopr_crypto_sphinx::ec_groups::X25519Suite;

pub struct HoprFullSphinxHeader<S: SphinxSuite = CurrentSphinxSuite>(PhantomData<S>);
impl<S: SphinxSuite> SphinxHeaderSpec for HoprFullSphinxHeader<S> {
    const MAX_HOPS: std::num::NonZeroUsize = std::num::NonZeroUsize::new(INTERMEDIATE_HOPS + 1).unwrap();
    const KEY_ID_SIZE: std::num::NonZeroUsize = std::num::NonZeroUsize::new(<S::P as Keypair>::Public::SIZE).unwrap();
    type KeyId = <S::P as Keypair>::Public;
    const RELAYER_DATA_SIZE: usize = por::POR_SECRET_LENGTH;
    type RelayerData = [u8; por::POR_SECRET_LENGTH];
    const LAST_HOP_DATA_SIZE: usize = 0;
    type LastHopData = [u8; 0];
}


pub struct HoprReducedSphinxHeader<S: SphinxSuite = CurrentSphinxSuite>(PhantomData<S>);
impl<S: SphinxSuite> SphinxHeaderSpec for HoprReducedSphinxHeader<S> {
    const MAX_HOPS: std::num::NonZeroUsize = std::num::NonZeroUsize::new(INTERMEDIATE_HOPS + 1).unwrap();
    const KEY_ID_SIZE: std::num::NonZeroUsize = std::num::NonZeroUsize::new(KeyIdent::SIZE).unwrap();
    type KeyId = KeyIdent;
    const RELAYER_DATA_SIZE: usize = por::POR_SECRET_LENGTH;
    type RelayerData = [u8; por::POR_SECRET_LENGTH];
    const LAST_HOP_DATA_SIZE: usize = 16;
    type LastHopData = [u8; 16];
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::ChainPacketComponents;
    use crate::packet::MetaPacket;

    #[test]
    fn header_and_packet_lengths() {
        let header_len = HoprFullSphinxHeader::<CurrentSphinxSuite>::HEADER_LEN;
        assert_eq!(
            MetaPacket::<CurrentSphinxSuite, HoprFullSphinxHeader>::HEADER_LEN,
            header_len
        ); // 394 bytes

        //let packet_len = packet_length::<CurrentSphinxSuite>(INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0);
        //assert_eq!(MetaPacket::<CurrentSphinxSuite, HoprSphinxHeader>::PACKET_LEN, packet_len); // 968 bytes

        let hopr_packet_len = ChainPacketComponents::SIZE;
        assert_eq!(
            MetaPacket::<CurrentSphinxSuite, HoprFullSphinxHeader>::PACKET_LEN + Ticket::SIZE,
            hopr_packet_len
        ); // 1116 bytes

        assert!(hopr_packet_len < 1492, "HOPR packet must fit within a layer 4 packet");
    }
}
