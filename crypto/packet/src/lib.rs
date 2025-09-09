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
//! In particular, as soon as there is a way to represent `Ed448` PeerIDs, it would be straightforward to create e.g.
//! `X448Suite`.
//!
//! This crate implements [RFC-0003](https://github.com/hoprnet/rfc/tree/main/rfcs/RFC-0003-hopr-packet-protocol).

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
    pub use crate::{
        packet::{
            HoprForwardedPacket, HoprIncomingPacket, HoprOutgoingPacket, HoprPacket, PacketRouting, PartialHoprPacket,
        },
        types::{HoprSenderId, HoprSurbId, PacketSignal, PacketSignals},
        validation::validate_unacknowledged_ticket,
    };
}

pub use hopr_crypto_sphinx::prelude::{KeyIdMapper, ReplyOpener};

/// Currently used public key cipher suite for Sphinx.
///
/// This is currently the [`Ed25519Suite`], because it is faster than [`X25519Suite`].
pub type HoprSphinxSuite = Ed25519Suite;

/// Current Sphinx header specification for the HOPR protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HoprSphinxHeaderSpec;

impl SphinxHeaderSpec for HoprSphinxHeaderSpec {
    type KeyId = KeyIdent<4>;
    type PRG = hopr_crypto_types::primitives::ChaCha20;
    type PacketReceiverData = types::HoprSenderId;
    type Pseudonym = HoprPseudonym;
    type RelayerData = por::ProofOfRelayString;
    type SurbReceiverData = por::SurbReceiverInfo;
    type UH = hopr_crypto_types::primitives::Poly1305;

    const MAX_HOPS: std::num::NonZeroUsize = std::num::NonZeroUsize::new(INTERMEDIATE_HOPS + 1).unwrap();
}

/// Single Use Reply Block representation for HOPR protocol.
pub type HoprSurb = SURB<HoprSphinxSuite, HoprSphinxHeaderSpec>;

/// Type alias for identifiable [`ReplyOpener`].
pub type HoprReplyOpener = (types::HoprSurbId, ReplyOpener);

/// Size of the maximum packet payload.
///
/// Adjust this value to change the maximum packet size.
/// The calculation here is based on the fact that libp2p Stream over QUIC
/// leaves space for 1460 bytes in the packet payload.
///
/// **DO NOT USE this value for calculations outside of this crate: use `HoprPacket::PAYLOAD_SIZE` instead!**
pub(crate) const PAYLOAD_SIZE_INT: usize = 1021;

#[cfg(test)]
mod tests {
    use hopr_crypto_sphinx::prelude::MetaPacket;

    use super::*;
    use crate::packet::HoprPacket;

    #[test]
    fn header_and_packet_lengths() {
        let hopr_packet_len = HoprPacket::SIZE;
        assert_eq!(
            MetaPacket::<HoprSphinxSuite, HoprSphinxHeaderSpec, PAYLOAD_SIZE_INT>::PACKET_LEN + Ticket::SIZE,
            hopr_packet_len
        );

        assert!(
            hopr_packet_len <= 1492 - 32, // 32 bytes was measured as the libp2p QUIC overhead
            "HOPR packet of {hopr_packet_len} bytes must fit within a layer 4 packet with libp2p overhead"
        );
    }

    #[test]
    fn packet_length() {
        let packet_len = HoprPacket::SIZE;
        assert_eq!(packet_len, 438 + PAYLOAD_SIZE_INT);
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

    #[test]
    fn max_surbs_per_packet_must_be_at_least_2() {
        const _: () = {
            assert!(HoprPacket::MAX_SURBS_IN_PACKET >= 2);
        };
    }
}
