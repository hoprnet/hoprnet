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

pub mod sphinx;

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
    pub use hopr_types::internal::routing::{HoprSenderId, HoprSurbId};

    pub use super::*;
    pub use crate::{
        packet::{
            HoprForwardedPacket, HoprIncomingPacket, HoprOutgoingPacket, HoprPacket, PacketRouting, PartialHoprPacket,
        },
        types::{HoprPixGroupElement, PacketSignal, PacketSignals},
        validation::validate_unacknowledged_ticket,
    };
}

use hopr_protocol_pix::{PixGroup, PixScalar};
use hopr_types::{crypto::prelude::*, internal::prelude::*, primitive::prelude::*};
use sphinx::prelude::*;
pub use sphinx::prelude::{ProtocolKeyIdMapper, ReplyOpener};

/// Currently used public key cipher suite for Sphinx.
///
/// This is currently the [`Ed25519Suite`], because it is faster than `X25519Suite`.
pub type HoprSphinxSuite = Ed25519Suite;

/// Current Sphinx header specification for the HOPR protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HoprSphinxHeaderSpec;

impl SphinxHeaderSpec for HoprSphinxHeaderSpec {
    type KeyId = HoprKeyIdent;
    type PRG = ChaCha20;
    type PacketReceiverData = HoprSenderId;
    type Pseudonym = HoprPseudonym;
    type RelayerData = por::ProofOfRelayString;
    type SurbReceiverData = types::SurbReceiverInfo;
    type UH = Poly1305;

    const MAX_HOPS: std::num::NonZeroUsize = std::num::NonZeroUsize::new(INTERMEDIATE_HOPS + 1).unwrap();
}

/// Type alias for 32-bit HOPR Offchain Public Key Identifier.
pub type HoprKeyIdent = KeyIdent<4>;

/// Single Use Reply Block representation for HOPR protocol.
pub type HoprSurb = SURB<HoprSphinxSuite, HoprSphinxHeaderSpec>;

/// Type alias for identifiable [`ReplyOpener`].
pub type HoprReplyOpener = (HoprSurbId, ReplyOpener);

/// Size of the maximum packet payload.
///
/// Adjust this value to change the maximum packet size.
/// The calculation here is based on the fact that libp2p Stream over QUIC
/// leaves space for 1460 bytes in the packet payload.
///
/// **DO NOT USE this value for calculations outside of this crate: use `HoprPacket::PAYLOAD_SIZE` instead!**
pub(crate) const PAYLOAD_SIZE_INT: usize = DefaultSphinxPacketSize::USIZE - 1; // minus padding byte

/// Current specification of the PIX protocol in HOPR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HoprPixSpec;

#[cfg(not(feature = "bjj"))]
impl hopr_protocol_pix::PixSpec for HoprPixSpec {
    type AddressPrivateKey = ChainKeypair;
    type Cipher = ChaCha20;
    type Curve = Secp256k1;
    type DepositAddress = Address;
    type Digest = Blake3;
    type Pseudonym = SimplePseudonym;

    const HASH_TO_SCALAR_SUITE_ID: &'static [u8] = b"Secp256k1_XMD:BLAKE3_SSWU_RO_";

    fn group_to_deposit_address(group: PixGroup<Self>) -> Option<Self::DepositAddress> {
        PublicKey::try_from(group.to_affine()).ok().map(|pk| pk.to_address())
    }

    fn scalar_to_private_key(scalar: PixScalar<Self>) -> Option<Self::AddressPrivateKey> {
        ChainKeypair::from_secret(scalar.to_bytes().as_ref()).ok()
    }
}

#[cfg(feature = "bjj")]
impl hopr_protocol_pix::PixSpec for HoprPixSpec {
    type AddressPrivateKey = BjjKeypair;
    type Cipher = ChaCha20;
    type Curve = BabyJubJub;
    type DepositAddress = BjjPublicKey;
    type Digest = Blake3;
    type Pseudonym = SimplePseudonym;

    const HASH_TO_SCALAR_SUITE_ID: &'static [u8] = b"BabyJubJub_XMD:BLAKE3_SSWU_RO_";

    fn group_to_deposit_address(group: PixGroup<Self>) -> Option<Self::DepositAddress> {
        BjjPublicKey::try_from(group).ok()
    }

    fn scalar_to_private_key(scalar: PixScalar<Self>) -> Option<Self::AddressPrivateKey> {
        BjjKeypair::from_secret(scalar.to_bytes().as_ref()).ok()
    }
}

/// HOPR-specific encrypted partial SSA share type from the PIX protocol.
pub type HoprEncryptedPartialSsaShare = hopr_protocol_pix::EncryptedPartialSsaShare<HoprPixSpec>;

/// HOPR-specific [`hopr_protocol_pix::ShareResolution`].
#[cfg(not(feature = "bjj"))]
pub type HoprShareResolution = hopr_protocol_pix::ShareResolution<SimplePseudonym, ChainKeypair>;
#[cfg(feature = "bjj")]
pub type HoprShareResolution = hopr_protocol_pix::ShareResolution<SimplePseudonym, BjjKeypair>;

/// HOPR-specific [`hopr_protocol_pix::SsaCommitmentState`].
#[cfg(not(feature = "bjj"))]
pub type HoprSsaCommitmentState = hopr_protocol_pix::SsaCommitmentState<SimplePseudonym, Address>;
#[cfg(feature = "bjj")]
pub type HoprSsaCommitmentState = hopr_protocol_pix::SsaCommitmentState<SimplePseudonym, BjjPublicKey>;

/// HOPR-specific PIX scalar type.
///
/// This is the normalized form of `hopr_protocol_pix::PixScalar<HoprPixSpec>`
/// (i.e. `<<HoprPixSpec as PixSpec>::Curve as CurveArithmetic>::Scalar`),
/// re-exported here so downstream crates can name it without depending on
/// directly.
///
/// This also avoids a Rust compiler issue due to deep nesting of PixScalar<HoprPixSpec> when used
/// itself as another generic argument.
#[cfg(not(feature = "bjj"))]
pub type HoprPixScalar = crypto_traits::elliptic_curve::Scalar<Secp256k1>;
#[cfg(feature = "bjj")]
pub type HoprPixScalar = BabyJubJubScalar;

/// HOPR-specific PIX group element representation type.
///
/// This is the normalized (concrete) form of `hopr_protocol_pix::PixGroupRepr<HoprPixSpec>`
/// (i.e. `<PixGroup<HoprPixSpec> as GroupEncoding>::Repr`), re-exported here as the concrete
/// type.
///
/// Using the concrete type instead of the associated-type projection avoids a Rust coherence
/// error (E0119): implementing `From`/`TryFrom` for a new-type wrapping the projection conflicts
/// with the blanket `impl<T> From<T> for T` because the compiler cannot prove the unresolved
/// projection is distinct from the wrapper type.
#[cfg(not(feature = "bjj"))]
pub type HoprPixGroupRepr = crypto_traits::elliptic_curve::array::Array<u8, crypto_traits::elliptic_curve::consts::U33>;
#[cfg(feature = "bjj")]
pub type HoprPixGroupRepr = BabyJubJubCompressedPoint;

#[cfg(test)]
mod tests {
    use sphinx::prelude::MetaPacket;

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
            hopr_packet_len <= 1492 - 31,
            "HOPR packet of {hopr_packet_len} bytes must fit within a layer 4 packet with libp2p overhead"
        );
    }

    #[test]
    fn packet_length() {
        let packet_len = HoprPacket::SIZE;
        assert_eq!(packet_len, 422 + PAYLOAD_SIZE_INT);
    }

    #[test]
    fn header_length() {
        let header_len = HoprSphinxHeaderSpec::HEADER_LEN;
        assert_eq!(header_len, 241);
    }

    #[test]
    fn surb_length() {
        let surb_len = HoprSurb::SIZE;
        assert_eq!(surb_len, 401);
        assert!(HoprPacket::PAYLOAD_SIZE > surb_len * 2);
    }

    #[test]
    fn max_surbs_per_packet_must_be_at_least_2() {
        const _: () = {
            assert!(HoprPacket::MAX_SURBS_IN_PACKET >= 2);
        };
    }
}
