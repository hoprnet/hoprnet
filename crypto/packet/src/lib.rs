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

/// Implements the overlay packet intermediary object.
pub mod chain;
/// Enumerates all errors in this crate.
pub mod errors;
/// Implements SPHINX packet format.
pub mod packet;
/// Implements the Proof of Relay.
pub mod por;
/// Implements ticket validation logic.
pub mod validation;

/// Currently used public key cipher suite for Sphinx.
pub type CurrentSphinxSuite = hopr_crypto_sphinx::ec_groups::X25519Suite;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::ChainPacketComponents;
    use crate::packet::{packet_length, MetaPacket};
    use crate::por::POR_SECRET_LENGTH;
    use hopr_crypto_sphinx::routing::header_length;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;

    #[test]
    fn header_and_packet_lengths() {
        let header_len = header_length::<CurrentSphinxSuite>(INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0);
        assert_eq!(MetaPacket::<CurrentSphinxSuite>::HEADER_LEN, header_len); // 394 bytes

        let packet_len = packet_length::<CurrentSphinxSuite>(INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0);
        assert_eq!(MetaPacket::<CurrentSphinxSuite>::PACKET_LEN, packet_len); // 968 bytes

        let hopr_packet_len = ChainPacketComponents::SIZE;
        assert_eq!(
            MetaPacket::<CurrentSphinxSuite>::PACKET_LEN + Ticket::SIZE,
            hopr_packet_len
        ); // 1116 bytes

        assert!(hopr_packet_len < 1492, "HOPR packet must fit within a layer 4 packet");
    }
}
