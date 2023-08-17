use crate::acknowledgement::PendingAcknowledgement::{WaitingAsRelayer, WaitingAsSender};
use crate::channels::Ticket;
use core_crypto::errors::CryptoError::{InvalidChallenge, SignatureVerification};
use core_crypto::keypairs::OffchainKeypair;
use core_crypto::types::{HalfKey, HalfKeyChallenge, OffchainPublicKey, OffchainSignature, Response};
use serde::{Deserialize, Serialize};
use utils_types::errors;
use utils_types::errors::GeneralError::ParseError;
use utils_types::primitives::Address;
use utils_types::traits::BinarySerializable;

/// Represents packet acknowledgement
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct Acknowledgement {
    ack_signature: OffchainSignature,
    pub ack_key_share: HalfKey,
    validated: bool,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Acknowledgement {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(ack_key_share: HalfKey, node_keypair: &OffchainKeypair) -> Self {
        Self {
            ack_signature: OffchainSignature::sign_message(&ack_key_share.to_bytes(), node_keypair),
            ack_key_share,
            validated: true,
        }
    }

    /// Validates the acknowledgement. Must be called immediately after deserialization or otherwise
    /// any operations with the deserialized acknowledgment will panic.
    pub fn validate(&mut self, sender_node_key: &OffchainPublicKey) -> bool {
        self.validated = self
            .ack_signature
            .verify_message(&self.ack_key_share.to_bytes(), sender_node_key);

        self.validated
    }

    /// Obtains the acknowledged challenge out of this acknowledgment.
    pub fn ack_challenge(&self) -> HalfKeyChallenge {
        assert!(self.validated, "acknowledgement not validated");
        self.ack_key_share.to_challenge()
    }
}

impl BinarySerializable for Acknowledgement {
    const SIZE: usize = OffchainSignature::SIZE + HalfKey::SIZE;

    fn from_bytes(data: &[u8]) -> errors::Result<Self> {
        let mut buf = data.to_vec();
        if data.len() == Self::SIZE {
            let ack_signature = OffchainSignature::from_bytes(buf.drain(..OffchainSignature::SIZE).as_ref())?;
            let ack_key_share = HalfKey::from_bytes(buf.drain(..HalfKey::SIZE).as_ref())?;
            Ok(Self {
                ack_signature,
                ack_key_share,
                validated: false,
            })
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        assert!(self.validated, "acknowledgement not validated");
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.ack_signature.to_bytes());
        ret.extend_from_slice(&self.ack_key_share.to_bytes());
        ret.into_boxed_slice()
    }
}

/// Contains acknowledgment information and the respective ticket
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct AcknowledgedTicket {
    pub ticket: Ticket,
    pub response: Response,
    pub signer: Address,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl AcknowledgedTicket {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(ticket: Ticket, response: Response, signer: Address) -> Self {
        assert_ne!(
            ticket.counterparty, signer,
            "signer must be different from the ticket counterparty"
        );
        Self {
            ticket,
            response,
            signer,
        }
    }
}

impl BinarySerializable for AcknowledgedTicket {
    const SIZE: usize = Ticket::SIZE + Response::SIZE + Address::SIZE;

    fn from_bytes(data: &[u8]) -> errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut buf = data.to_vec();
            let ticket = Ticket::from_bytes(buf.drain(..Ticket::SIZE).as_ref())?;
            let response = Response::from_bytes(buf.drain(..Response::SIZE).as_ref())?;
            let signer = Address::from_bytes(buf.drain(..Address::SIZE).as_ref())?;

            Ok(Self {
                ticket,
                response,
                signer,
            })
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.ticket.to_bytes());
        ret.extend_from_slice(&self.response.to_bytes());
        ret.extend_from_slice(&self.signer.to_bytes());
        ret.into_boxed_slice()
    }
}

impl AcknowledgedTicket {
    /// Verifies if the embedded ticket has been signed by the given issuer and also
    /// that the challenge on the embedded response matches the challenge on the ticket.
    pub fn verify(&self, issuer: &Address) -> core_crypto::errors::Result<()> {
        (self.ticket.verify(issuer).map(|_| true)?
            && self
                .response
                .to_challenge()
                .to_ethereum_challenge()
                .eq(&self.ticket.challenge))
        .then_some(())
        .ok_or(InvalidChallenge)
    }
}

impl std::fmt::Display for AcknowledgedTicket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcknowledgedTicket")
            .field("ticket", &self.ticket)
            .field("response", &self.response)
            .field("signer", &self.signer)
            .finish()
    }
}

/// Wrapper for an unacknowledged ticket
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct UnacknowledgedTicket {
    pub ticket: Ticket,
    pub own_key: HalfKey,
    pub signer: Address,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl UnacknowledgedTicket {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(ticket: Ticket, own_key: HalfKey, signer: Address) -> Self {
        Self {
            ticket,
            own_key,
            signer,
        }
    }

    pub fn get_challenge(&self) -> HalfKeyChallenge {
        self.own_key.to_challenge()
    }
}

impl UnacknowledgedTicket {
    /// Verifies if signature on the embedded ticket using the embedded public key.
    pub fn verify_signature(&self) -> core_crypto::errors::Result<()> {
        self.ticket.verify(&self.signer)
    }

    /// Verifies if the challenge on the embedded ticket matches the solution
    /// from the given acknowledgement and the embedded half key.
    pub fn verify_challenge(&self, acknowledgement: &HalfKey) -> core_crypto::errors::Result<()> {
        self.get_response(acknowledgement)?
            .to_challenge()
            .to_ethereum_challenge()
            .eq(&self.ticket.challenge)
            .then_some(())
            .ok_or(SignatureVerification)
    }

    pub fn get_response(&self, acknowledgement: &HalfKey) -> core_crypto::errors::Result<Response> {
        Response::from_half_keys(&self.own_key, acknowledgement)
    }
}

impl BinarySerializable for UnacknowledgedTicket {
    const SIZE: usize = Ticket::SIZE + HalfKey::SIZE + Address::SIZE;

    fn from_bytes(data: &[u8]) -> errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut buf = data.to_vec();
            let ticket = Ticket::from_bytes(buf.drain(..Ticket::SIZE).as_ref())?;
            let own_key = HalfKey::from_bytes(buf.drain(..HalfKey::SIZE).as_ref())?;
            let signer = Address::from_bytes(buf.drain(..Address::SIZE).as_ref())?;
            Ok(Self {
                ticket,
                own_key,
                signer,
            })
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.ticket.to_bytes());
        ret.extend_from_slice(&self.own_key.to_bytes());
        ret.extend_from_slice(&self.signer.to_bytes());
        ret.into_boxed_slice()
    }
}

/// Contains either unacknowledged ticket if we're waiting for the acknowledgement as a relayer
/// or information if we wait for the acknowledgement as a sender.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PendingAcknowledgement {
    /// We're waiting for acknowledgement as a sender
    WaitingAsSender,
    /// We're waiting for the acknowledgement as a relayer with a ticket
    WaitingAsRelayer(UnacknowledgedTicket),
}

impl PendingAcknowledgement {
    const SENDER_PREFIX: u8 = 0;
    const RELAYER_PREFIX: u8 = 1;
}

impl BinarySerializable for PendingAcknowledgement {
    const SIZE: usize = 1;

    fn from_bytes(data: &[u8]) -> errors::Result<Self> {
        if data.len() >= Self::SIZE {
            match data[0] {
                Self::SENDER_PREFIX => Ok(WaitingAsSender),
                Self::RELAYER_PREFIX => Ok(WaitingAsRelayer(UnacknowledgedTicket::from_bytes(&data[1..])?)),
                _ => Err(ParseError),
            }
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        match &self {
            WaitingAsSender => ret.push(Self::SENDER_PREFIX),
            WaitingAsRelayer(unacknowledged) => {
                ret.push(Self::RELAYER_PREFIX);
                ret.extend_from_slice(&unacknowledged.to_bytes());
            }
        }
        ret.into_boxed_slice()
    }
}

#[cfg(test)]
pub mod test {
    use crate::acknowledgement::{AcknowledgedTicket, Acknowledgement, PendingAcknowledgement, UnacknowledgedTicket};
    use crate::channels::Ticket;
    use core_crypto::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use core_crypto::types::{Challenge, CurvePoint, HalfKey, Hash, OffchainPublicKey, Response};
    use ethnum::u256;
    use hex_literal::hex;
    use utils_types::primitives::{Address, Balance, BalanceType, U256};
    use utils_types::traits::BinarySerializable;

    fn mock_ticket(pk: &ChainKeypair) -> Ticket {
        let inverse_win_prob = u256::new(1u128); // 100 %
        let price_per_packet = u256::new(10000000000000000u128); // 0.01 HOPR
        let path_pos = 5;

        Ticket::new(
            Address::new(&[0u8; Address::SIZE]),
            U256::new("1"),
            Balance::new(
                (inverse_win_prob * price_per_packet * path_pos as u128).into(),
                BalanceType::HOPR,
            ),
            U256::from_inverse_probability(inverse_win_prob.into()).unwrap(),
            U256::new("4"),
            pk,
        )
    }

    #[test]
    fn test_pending_ack_sender() {
        assert_eq!(
            PendingAcknowledgement::WaitingAsSender,
            PendingAcknowledgement::from_bytes(&PendingAcknowledgement::WaitingAsSender.to_bytes()).unwrap()
        );
    }

    #[test]
    fn test_acknowledgement() {
        let pk_2 = OffchainKeypair::from_secret(&hex!(
            "4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b"
        ))
        .unwrap();
        let pub_key_2 = OffchainPublicKey::from_privkey(pk_2.secret().as_ref()).unwrap();

        let ack_key = HalfKey::new(&hex!(
            "3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"
        ));

        let mut ack1 = Acknowledgement::new(ack_key, &pk_2);
        assert!(ack1.validate(&pub_key_2));

        let mut ack2 = Acknowledgement::from_bytes(&ack1.to_bytes()).unwrap();
        assert!(ack2.validate(&pub_key_2));

        assert_eq!(ack1, ack2);
    }

    #[test]
    fn test_unacknowledged_ticket() {
        let pk_1 = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))
        .unwrap();
        let pub_key_1 = pk_1.public().0.clone();

        let hk1 = HalfKey::new(&hex!(
            "3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"
        ));
        let hk2 = HalfKey::new(&hex!(
            "4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b"
        ));
        let cp1: CurvePoint = hk1.to_challenge().into();
        let cp2: CurvePoint = hk2.to_challenge().into();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let mut ticket1 = mock_ticket(&pk_1);
        ticket1.set_challenge(Challenge::from(cp_sum).to_ethereum_challenge(), &pk_1);

        let unack1 = UnacknowledgedTicket::new(ticket1, hk1, pub_key_1.to_address());
        assert!(unack1.verify_signature().is_ok());
        assert!(unack1.verify_challenge(&hk2).is_ok());

        let unack2 = UnacknowledgedTicket::from_bytes(&unack1.to_bytes()).unwrap();
        assert_eq!(unack1, unack2);

        let pending_ack_1 = PendingAcknowledgement::WaitingAsRelayer(unack1);
        let pending_ack_2 = PendingAcknowledgement::from_bytes(&pending_ack_1.to_bytes()).unwrap();
        assert_eq!(pending_ack_1, pending_ack_2);
    }

    #[test]
    fn test_acknowledged_ticket() {
        let pk = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))
        .unwrap();
        let pub_key = pk.public().0.clone();
        let resp = Response::new(&hex!(
            "4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b"
        ));

        let mut ticket1 = mock_ticket(&pk);
        ticket1.set_challenge(resp.to_challenge().to_ethereum_challenge(), &pk);

        let akt_1 = AcknowledgedTicket::new(ticket1, resp, Hash::create(&[&hex!("deadbeef")]), pub_key.to_address());
        assert!(akt_1.verify(&pub_key.to_address()).is_ok());

        let akt_2 = AcknowledgedTicket::from_bytes(&akt_1.to_bytes()).unwrap();
        assert_eq!(akt_1, akt_2);
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::acknowledgement::{AcknowledgedTicket, Acknowledgement, UnacknowledgedTicket};
    use core_crypto::types::{HalfKey, Response};
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::Address;
    use utils_types::traits::BinarySerializable;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct PendingAcknowledgement {
        w: super::PendingAcknowledgement,
    }

    #[wasm_bindgen]
    impl PendingAcknowledgement {
        #[wasm_bindgen(constructor)]
        pub fn new(is_sender: bool, ticket: Option<UnacknowledgedTicket>) -> Self {
            if is_sender {
                Self {
                    w: super::PendingAcknowledgement::WaitingAsSender,
                }
            } else {
                Self {
                    w: super::PendingAcknowledgement::WaitingAsRelayer(ticket.unwrap()),
                }
            }
        }

        pub fn is_msg_sender(&self) -> bool {
            match &self.w {
                super::PendingAcknowledgement::WaitingAsSender => true,
                super::PendingAcknowledgement::WaitingAsRelayer(_) => false,
            }
        }

        pub fn ticket(&self) -> Option<UnacknowledgedTicket> {
            match &self.w {
                super::PendingAcknowledgement::WaitingAsSender => None,
                super::PendingAcknowledgement::WaitingAsRelayer(ticket) => Some(ticket.clone()),
            }
        }

        pub fn deserialize(data: &[u8]) -> JsResult<PendingAcknowledgement> {
            Ok(Self {
                w: ok_or_jserr!(super::PendingAcknowledgement::from_bytes(data))?,
            })
        }

        pub fn serialize(&self) -> Box<[u8]> {
            self.w.to_bytes()
        }
    }

    #[wasm_bindgen]
    impl UnacknowledgedTicket {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<UnacknowledgedTicket> {
            ok_or_jserr!(Self::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "get_response")]
        pub fn _get_response(&self, acknowledgement: &HalfKey) -> JsResult<Response> {
            ok_or_jserr!(self.get_response(acknowledgement))
        }

        #[wasm_bindgen(js_name = "verify_challenge")]
        pub fn _verify_challenge(&self, acknowledgement: &HalfKey) -> JsResult<bool> {
            ok_or_jserr!(self.verify_challenge(acknowledgement).map(|_| true))
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &UnacknowledgedTicket) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl AcknowledgedTicket {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<AcknowledgedTicket> {
            ok_or_jserr!(Self::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &AcknowledgedTicket) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "verify")]
        pub fn _verify(&self, issuer: &Address) -> JsResult<bool> {
            ok_or_jserr!(self.verify(issuer).map(|_| true))
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl Acknowledgement {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<Acknowledgement> {
            ok_or_jserr!(Self::from_bytes(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &Acknowledgement) -> bool {
            self.eq(other)
        }

        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> Self {
            self.clone()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }
}
