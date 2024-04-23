//! This module should be removed in 3.0. It introduces the missing binary format separation
//! from business objects (AcknowledgedTicket), to ensure backwards compatibility of
//! Ticket Aggregation in 2.x releases

use ethers::core::k256::AffinePoint;
use ethers::prelude::k256::elliptic_curve::sec1::FromEncodedPoint;
use ethers::prelude::k256::Scalar;
use serde::de::Visitor;
use serde::{de, Deserialize, Deserializer, Serialize};

use hopr_primitive_types::prelude::{BytesEncodable, GeneralError, IntoEndian, U256};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct Address {
    addr: [u8; 20],
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, PartialOrd, Ord, std::hash::Hash)]
pub struct Hash {
    hash: [u8; 32],
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    response: [u8; 32],
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct CurvePoint {
    affine: AffinePoint,
}

impl Default for CurvePoint {
    fn default() -> Self {
        Self {
            affine: hopr_crypto_types::types::CurvePoint::from_exponent(&U256::one().to_be_bytes())
                .unwrap()
                .into(),
        }
    }
}

impl From<AffinePoint> for CurvePoint {
    fn from(value: AffinePoint) -> Self {
        Self { affine: value }
    }
}

#[derive(Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VrfParameters {
    /// the pseudo-random point
    pub v: CurvePoint,
    pub h: Scalar,
    pub s: Scalar,
    /// helper value for smart contract
    pub h_v: CurvePoint,
    /// helper value for smart contract
    pub s_b: CurvePoint,
}

impl From<VrfParameters> for hopr_crypto_types::vrf::VrfParameters {
    fn from(value: VrfParameters) -> Self {
        Self {
            v: value.v.affine.into(),
            h: value.h,
            s: value.s,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum AcknowledgedTicketStatus {
    #[default]
    Untouched,
    BeingRedeemed {
        tx_hash: Hash,
    },
    BeingAggregated {
        start: u64,
        end: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ticket(crate::tickets::Ticket);

impl Serialize for Ticket {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0.clone().into_encoded())
    }
}

struct TicketVisitor {}

impl<'de> Visitor<'de> for TicketVisitor {
    type Value = Ticket;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_fmt(format_args!(
            "a byte-array with {} elements",
            crate::tickets::Ticket::SIZE
        ))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ticket(
            v.try_into()
                .map_err(|e: GeneralError| de::Error::custom(e.to_string()))?,
        ))
    }
}

// Use compact deserialization for tickets as they are used very often
impl<'de> Deserialize<'de> for Ticket {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(TicketVisitor {})
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcknowledgedTicket {
    #[serde(default)]
    pub status: AcknowledgedTicketStatus,
    pub ticket: Ticket,
    pub response: Response,
    pub vrf_params: VrfParameters,
    pub signer: Address,
}

impl AcknowledgedTicket {
    pub fn new(
        value: crate::tickets::TransferableWinningTicket,
        me: &hopr_primitive_types::primitives::Address,
        domain_separator: &hopr_crypto_types::types::Hash,
    ) -> Self {
        let mut response = Response::default();
        response.response.copy_from_slice(value.response.as_ref());

        let mut signer = Address::default();
        signer.addr.copy_from_slice(value.signer.as_ref());

        let vrf_params = VrfParameters {
            v: AffinePoint::from_encoded_point(value.vrf_params.v.as_compressed())
                .expect("invalid vrf params")
                .into(),
            h: value.vrf_params.h,
            s: value.vrf_params.s,
            h_v: AffinePoint::from_encoded_point(&value.vrf_params.get_h_v_witness())
                .expect("invalid vrf params")
                .into(),
            s_b: AffinePoint::from_encoded_point(
                &value
                    .vrf_params
                    .get_s_b_witness(
                        me,
                        &value.ticket.get_hash(domain_separator).into(),
                        domain_separator.as_ref(),
                    )
                    .expect("invalid vrf params"),
            )
            .expect("invalid vrf params")
            .into(),
        };

        Self {
            status: AcknowledgedTicketStatus::BeingAggregated { start: 0, end: 0 }, // values not  used
            ticket: Ticket(value.ticket),
            response,
            vrf_params,
            signer,
        }
    }
}

impl From<AcknowledgedTicket> for crate::tickets::TransferableWinningTicket {
    fn from(value: AcknowledgedTicket) -> Self {
        Self {
            ticket: value.ticket.0,
            response: value.response.response.into(),
            vrf_params: value.vrf_params.into(),
            signer: hopr_primitive_types::primitives::Address::new(&value.signer.addr),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::legacy::Ticket;
    use crate::tickets::{TicketBuilder, TransferableWinningTicket};
    use ethers::utils::hex;
    use hex_literal::hex;
    use hopr_crypto_types::prelude::{ChainKeypair, Challenge, CurvePoint, HalfKey, Hash, Keypair};
    use hopr_primitive_types::prelude::EthereumChallenge;
    use hopr_primitive_types::primitives::Address;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!(
            "14d2d952715a51aadbd4cc6bfac9aa9927182040da7b336d37d5bb7247aa7566"
        ))
        .unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!(
            "48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c"
        ))
        .unwrap();
        static ref DESTINATION: [u8; 20] = hex!("345ae204774ff2b3e8d4cac884dad3d1603b5917");
        static ref CHANNEL_DST: [u8; 32] = hex!("57dc754bb522f2fe7799e471fd6efd0b6139a2120198f15b92f4a78cb882af35");
        static ref ETHEREUM_CHALLENGE: [u8; 20] = hex!("4162339a4204a1cedf43c92049875a19cb09dd20");
        static ref RESPONSE: [u8; 32] = hex!("83c841f72b270440b7c8cd7b4f7d806a84f40ead5b04edccbb9a4c8936b91436");
    }

    #[test]
    fn test_legacy_binary_compatibility_with_2_0_8() {
        let domain_separator = Hash::from(*CHANNEL_DST);

        let ticket = TicketBuilder::default()
            .direction(&ALICE.public().to_address(), &Address::new(DESTINATION.as_ref()))
            .amount(1000000_u64)
            .index(10)
            .index_offset(2)
            .win_prob(1.0)
            .channel_epoch(2)
            .challenge(EthereumChallenge::new(ETHEREUM_CHALLENGE.as_ref()))
            .build_signed(&ALICE, &domain_separator)
            .expect("should build ticket");

        let mut signer = crate::legacy::Address::default();
        signer.addr.copy_from_slice(ALICE.public().to_address().as_ref());

        let ack_ticket = crate::legacy::AcknowledgedTicket {
            status: crate::legacy::AcknowledgedTicketStatus::BeingAggregated { start: 0, end: 0 },
            ticket: Ticket(ticket.verified_ticket().clone()),
            response: crate::legacy::Response { response: *RESPONSE },
            vrf_params: Default::default(),
            signer,
        };

        let serialized = cbor4ii::serde::to_vec(Vec::with_capacity(300), &ack_ticket).unwrap();
        let hex_encoded = hex::encode(serialized);

        // This is the serialized output from 2.0.8 with the same inputs
        let expected = "a566737461747573a16f4265696e6741676772656761746564a26573746172740063656e6400667469636b65745894cff6549a8f770afcc2ff07ff0d947178a7fb935539ecb2316ebeabff3f1740040000000000000000000f424000000000000a00000002000002ffffffffffffff4162339a4204a1cedf43c92049875a19cb09dd20b33432df13bb26810abc14161b514fb15a2027b05288ad9d8c3befd73831fce3d64808cc3c386fbf1a33263342a32434f24ea0a78b2cb6d503133a10f528378268726573706f6e7365a168726573706f6e73659820188318c8184118f7182b182704184018b718c818cd187b184f187d1880186a188418f40e18ad185b0418ed18cc18bb189a184c1889183618b91418366a7672665f706172616d73a56176a166616666696e65982102187918be1866187e18f918dc18bb18ac185518a01862189518ce18870b0702189b18fc18db182d18ce182818d9185918f21881185b1618f817189861689820000000000000000000000000000000000000000000000000000000000000000061739820000000000000000000000000000000000000000000000000000000000000000063685f76a166616666696e65982102187918be1866187e18f918dc18bb18ac185518a01862189518ce18870b0702189b18fc18db182d18ce182818d9185918f21881185b1618f817189863735f62a166616666696e65982102187918be1866187e18f918dc18bb18ac185518a01862189518ce18870b0702189b18fc18db182d18ce182818d9185918f21881185b1618f8171898667369676e6572a1646164647294182018ab183c18ad184e186c18d718c518c818da184518cd1889189218d8184518f4189c189200";
        assert_eq!(expected, hex_encoded);
    }

    #[test]
    fn test_legacy_must_serialize_deserialize_correctly() {
        let hk1 = HalfKey::try_from(hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa").as_ref())
            .unwrap();
        let hk2 = HalfKey::try_from(hex!("4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b").as_ref())
            .unwrap();

        let cp1: CurvePoint = hk1.to_challenge().try_into().unwrap();
        let cp2: CurvePoint = hk2.to_challenge().try_into().unwrap();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let domain_separator = Hash::try_from(CHANNEL_DST.as_ref()).unwrap();

        let ticket = TicketBuilder::default()
            .direction(&ALICE.public().to_address(), &BOB.public().to_address())
            .amount(1000000_u64)
            .index(10)
            .index_offset(2)
            .win_prob(1.0)
            .channel_epoch(2)
            .challenge(Challenge::from(cp_sum).to_ethereum_challenge())
            .build_signed(&ALICE, &domain_separator)
            .expect("should build ticket");

        let unack = ticket.into_unacknowledged(hk1);
        let acked = unack.acknowledge(&hk2).expect("should acknowledge");

        let transferable = acked
            .into_transferable(&BOB, &domain_separator)
            .expect("should convert to transferable");

        transferable
            .clone()
            .into_redeemable(&ALICE.public().to_address(), &domain_separator)
            .expect("should be transformable to redeemable");

        let serialized =
            crate::legacy::AcknowledgedTicket::new(transferable, &BOB.public().to_address(), &domain_separator);

        let deserialized = TransferableWinningTicket::from(serialized);

        deserialized
            .into_redeemable(&ALICE.public().to_address(), &domain_separator)
            .expect("should be transformable to redeemable");
    }
}
