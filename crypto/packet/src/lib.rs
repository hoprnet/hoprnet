//! # core-packet
//!
//! This crate contains the main packet processing functionality for the HOPR protocol.
//! It implements the following important protocol building blocks:
//!
//! - SPHINX packet format
//! - Proof of Relay
//!
//! Finally, it also implements a utility function which is used to validate tickets (module `validation`).
//!
//! The currently used implementation is selected using the `CurrentSphinxSuite` type in the `packet` module.
//!
//! The implementation can be easily extended for different elliptic curves (or even arithmetic multiplicative groups).
//! In particular, as soon as there's a way to represent `Ed448` PeerIDs, it would be straightforward to create e.g. `X448Suite`.
//!

use hopr_crypto_sphinx::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use std::marker::PhantomData;

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

pub mod prelude {
    pub use crate::packet::{HoprPacket, PacketRouting, PartialHoprPacket};
    pub use crate::validation::validate_unacknowledged_ticket;
}

/// Pseudonyms used for the return path.
pub type HoprPseudonym = hopr_crypto_types::prelude::SimplePseudonym;

/// Currently used public key cipher suite for Sphinx.
pub type HoprSphinxSuite = X25519Suite;

/// Current Sphinx header specification for the HOPR protocol.
pub struct HoprSphinxHeaderSpec<S: SphinxSuite = HoprSphinxSuite>(PhantomData<S>);

impl<S: SphinxSuite> SphinxHeaderSpec for HoprSphinxHeaderSpec<S> {
    const MAX_HOPS: std::num::NonZeroUsize = std::num::NonZeroUsize::new(INTERMEDIATE_HOPS + 1).unwrap();
    type KeyId = KeyIdent<4>;
    type Pseudonym = HoprPseudonym;
    type RelayerData = por::ProofOfRelayString;
    type SurbReceiverData = por::ProofOfRelayValues;
    type PRG = hopr_crypto_types::primitives::ChaCha20;
    type UH = hopr_crypto_types::primitives::Poly1305;
}

/// Single Use Reply Block representation for HOPR protocol.
pub type HoprSurb = SURB<HoprSphinxSuite, HoprSphinxHeaderSpec>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::HoprPacket;
    use hopr_crypto_sphinx::prelude::MetaPacket;

    #[test]
    fn header_and_packet_lengths() {
        let hopr_packet_len = HoprPacket::SIZE;
        assert_eq!(
            MetaPacket::<HoprSphinxSuite, HoprSphinxHeaderSpec, PAYLOAD_SIZE>::PACKET_LEN + Ticket::SIZE,
            hopr_packet_len
        );

        assert!(hopr_packet_len < 1492, "HOPR packet must fit within a layer 4 packet");
    }
}
