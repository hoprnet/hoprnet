//! # core-packet
//!
//! This crate contains the main packet processing functionality for the HOPR protocol.
//! It implements the following important protocol building blocks:
//!
//! - HOPR specific instantiation of the SPHINX packet format
//! - Proof of Relay
//!
//! Finally, it also implements a utility function which is used to validate tickets (module `validation`).
//!
//! The currently used implementation is selected using the [`HoprSphinxSuite`] type in the `packet` module.
//!
//! The implementation can be easily extended for different elliptic curves (or even arithmetic multiplicative groups).
//! In particular, as soon as there is a way to represent `Ed448` PeerIDs, it would be straightforward to create e.g. `X448Suite`.
//!

use hopr_crypto_sphinx::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

/// Lists all errors in this crate.
pub mod errors;
/// Implements the overlay packet intermediary object.
mod packet;
/// Implements the Proof of Relay.
mod por;
/// Contains various helper types.
mod types;
/// Implements ticket validation logic.
mod validation;

#[doc(hidden)]
pub mod prelude {
    pub use super::*;
    pub use crate::packet::{
        HoprForwardedPacket, HoprIncomingPacket, HoprOutgoingPacket, HoprPacket, PacketRouting, PartialHoprPacket,
    };
    pub use crate::types::{HoprSenderId, HoprSurbId};
    pub use crate::validation::validate_unacknowledged_ticket;
}

pub use hopr_crypto_sphinx::prelude::{KeyIdMapper, ReplyOpener};

/// Currently used public key cipher suite for Sphinx.
pub type HoprSphinxSuite = X25519Suite;

/// Current Sphinx header specification for the HOPR protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HoprSphinxHeaderSpec;

impl SphinxHeaderSpec for HoprSphinxHeaderSpec {
    const MAX_HOPS: std::num::NonZeroUsize = std::num::NonZeroUsize::new(INTERMEDIATE_HOPS + 1).unwrap();
    type KeyId = KeyIdent<4>;
    type Pseudonym = HoprPseudonym;
    type RelayerData = por::ProofOfRelayString;
    type PacketReceiverData = types::HoprSenderId;
    type SurbReceiverData = por::SurbReceiverInfo;
    type PRG = hopr_crypto_types::primitives::ChaCha20;
    type UH = hopr_crypto_types::primitives::Poly1305;
}

/// Single Use Reply Block representation for HOPR protocol.
pub type HoprSurb = SURB<HoprSphinxSuite, HoprSphinxHeaderSpec>;

/// Size of the maximum packet payload.
///
/// Adjust this value to change the maximum packet size.
///
/// **DO NOT USE this value for calculations outside of this crate: use `HoprPacket::PAYLOAD_SIZE` instead!**
pub(crate) const PAYLOAD_SIZE_INT: usize = 800;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::HoprPacket;
    use hopr_crypto_sphinx::prelude::MetaPacket;

    #[test]
    fn header_and_packet_lengths() {
        let hopr_packet_len = HoprPacket::SIZE;
        assert_eq!(
            MetaPacket::<HoprSphinxSuite, HoprSphinxHeaderSpec, PAYLOAD_SIZE_INT>::PACKET_LEN + Ticket::SIZE,
            hopr_packet_len
        );

        assert!(hopr_packet_len < 1492, "HOPR packet must fit within a layer 4 packet");
    }

    #[test]
    fn packet_length() {
        let packet_len = HoprPacket::SIZE;
        assert_eq!(packet_len, 1238);
    }

    #[test]
    fn header_length() {
        let header_len = HoprSphinxHeaderSpec::HEADER_LEN;
        assert_eq!(header_len, 241);
    }

    #[test]
    fn surb_length() {
        let surb_len = HoprSurb::SIZE;
        assert_eq!(surb_len, 395);
        assert!(HoprPacket::PAYLOAD_SIZE > surb_len * 2);
    }
}
