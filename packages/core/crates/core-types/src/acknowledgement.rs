use crate::acknowledgement::PendingAcknowledgement::{WaitingAsRelayer, WaitingAsSender};
use crate::channels::Ticket;
use core_crypto::primitives::{DigestLike, SimpleDigest};
use core_crypto::types::{HalfKey, HalfKeyChallenge, Hash, PublicKey, Response, Signature};
use utils_types::errors;
use utils_types::errors::GeneralError::ParseError;
use utils_types::traits::BinarySerializable;

/// Represents packet acknowledgement
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct Acknowledgement {
    ack_signature: Signature,
    challenge_signature: Signature,
    pub ack_key_share: HalfKey,
    validated: bool,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Acknowledgement {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(ack_challenge: AcknowledgementChallenge, ack_key_share: HalfKey, private_key: &[u8]) -> Self {
        let mut digest = SimpleDigest::default();
        digest.update(&ack_challenge.serialize());
        digest.update(&ack_key_share.serialize());

        Self {
            ack_signature: Signature::sign_hash(&digest.finalize(), private_key),
            challenge_signature: ack_challenge.signature,
            ack_key_share,
            validated: true,
        }
    }

    /// Validates the acknowledgement. Must be called immediately after deserialization or otherwise
    /// any operations with the deserialized acknowledgment will panic.
    pub fn validate(&mut self, own_public_key: &PublicKey, sender_public_key: &PublicKey) -> bool {
        let mut digest = SimpleDigest::default();
        digest.update(&self.ack_key_share.to_challenge().serialize());
        self.validated = self
            .challenge_signature
            .verify_hash_with_pubkey(&digest.finalize(), own_public_key);

        digest.update(&self.challenge_signature.serialize());
        digest.update(&self.ack_key_share.serialize());
        self.validated = self.validated
            && self
                .ack_signature
                .verify_hash_with_pubkey(&digest.finalize(), sender_public_key);

        self.validated
    }

    /// Obtains the acknowledged challenge out of this acknowledgment.
    pub fn ack_challenge(&self) -> HalfKeyChallenge {
        assert!(self.validated, "acknowledgement not validated");
        self.ack_key_share.to_challenge()
    }
}

impl BinarySerializable<'_> for Acknowledgement {
    const SIZE: usize = Signature::SIZE + AcknowledgementChallenge::SIZE + HalfKey::SIZE;

    fn deserialize(data: &[u8]) -> errors::Result<Self> {
        let mut buf = data.to_vec();
        if data.len() == Self::SIZE {
            let ack_signature = Signature::deserialize(buf.drain(..Signature::SIZE).as_ref())?;
            let challenge_signature = AcknowledgementChallenge::deserialize(buf.drain(..AcknowledgementChallenge::SIZE).as_ref())?;
            let ack_key_share = HalfKey::deserialize(buf.drain(..HalfKey::SIZE).as_ref())?;
            Ok(Self {
                ack_signature,
                challenge_signature: challenge_signature.signature,
                ack_key_share,
                validated: false,
            })
        } else {
            Err(ParseError)
        }
    }

    fn serialize(&self) -> Box<[u8]> {
        assert!(self.validated, "acknowledgement not validated");
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.ack_signature.raw_signature());
        ret.extend_from_slice(&self.challenge_signature.raw_signature());
        ret.extend_from_slice(&self.ack_key_share.serialize());
        ret.into_boxed_slice()
    }
}

/// Contains acknowledgment information and the respective ticket
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct AcknowledgedTicket {
    pub ticket: Ticket,
    pub response: Response,
    pub pre_image: Hash,
    pub signer: PublicKey,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl AcknowledgedTicket {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(ticket: Ticket, response: Response, pre_image: Hash, signer: PublicKey) -> Self {
        Self {
            ticket,
            response,
            pre_image,
            signer,
        }
    }
}

impl BinarySerializable<'_> for AcknowledgedTicket {
    const SIZE: usize = Ticket::SIZE + Response::SIZE + Hash::SIZE + PublicKey::SIZE_COMPRESSED;

    fn deserialize(data: &[u8]) -> errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut buf = data.to_vec();
            let ticket = Ticket::deserialize(buf.drain(..Ticket::SIZE).as_ref())?;
            let response = Response::deserialize(buf.drain(..Response::SIZE).as_ref())?;
            let pre_image = Hash::deserialize(buf.drain(..Hash::SIZE).as_ref())?;
            let signer = PublicKey::deserialize(buf.drain(..PublicKey::SIZE_COMPRESSED).as_ref())?;

            Ok(Self {
                ticket,
                response,
                pre_image,
                signer,
            })
        } else {
            Err(ParseError)
        }
    }

    fn serialize(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.ticket.serialize());
        ret.extend_from_slice(&self.response.serialize());
        ret.extend_from_slice(&self.pre_image.serialize());
        ret.extend_from_slice(&self.signer.serialize(true));
        ret.into_boxed_slice()
    }
}

/// Wrapper for an unacknowledged ticket
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct UnacknowledgedTicket {
    pub ticket: Ticket,
    pub own_key: HalfKey,
    pub signer: PublicKey,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl UnacknowledgedTicket {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(ticket: Ticket, own_key: HalfKey, signer: PublicKey) -> Self {
        Self {
            ticket,
            own_key,
            signer,
        }
    }

    pub fn verify_signature(&self) -> bool {
        self.ticket.verify(&self.signer)
    }

    pub fn get_challenge(&self) -> HalfKeyChallenge {
        self.own_key.to_challenge()
    }
}

impl UnacknowledgedTicket {
    pub fn verify_challenge(&self, acknowledgement: &HalfKey) -> core_crypto::errors::Result<bool> {
        Ok(Response::from_half_keys(&self.own_key, acknowledgement)?
            .to_challenge()
            .to_ethereum_challenge()
            .eq(&self.ticket.challenge))
    }

    pub fn get_response(&self, acknowledgement: &HalfKey) -> core_crypto::errors::Result<Response> {
        Response::from_half_keys(&self.own_key, acknowledgement)
    }
}

impl BinarySerializable<'_> for UnacknowledgedTicket {
    const SIZE: usize = Ticket::SIZE + HalfKey::SIZE + PublicKey::SIZE_UNCOMPRESSED;

    fn deserialize(data: &[u8]) -> errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut buf = data.to_vec();
            let ticket = Ticket::deserialize(buf.drain(..Ticket::SIZE).as_ref())?;
            let own_key = HalfKey::deserialize(buf.drain(..HalfKey::SIZE).as_ref())?;
            let signer = PublicKey::deserialize(buf.drain(..PublicKey::SIZE_COMPRESSED).as_ref())?;
            Ok(Self {
                ticket,
                own_key,
                signer,
            })
        } else {
            Err(ParseError)
        }
    }

    fn serialize(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.ticket.serialize());
        ret.extend_from_slice(&self.own_key.serialize());
        ret.extend_from_slice(&self.signer.serialize(false));
        ret.into_boxed_slice()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct AcknowledgementChallenge {
    ack_challenge: Option<HalfKeyChallenge>,
    signature: Signature,
}

fn hash_challenge(challenge: &HalfKeyChallenge) -> Box<[u8]> {
    let mut digest = SimpleDigest::default();
    digest.update(&challenge.serialize());
    digest.finalize()
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl AcknowledgementChallenge {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(ack_challenge: &HalfKeyChallenge, private_key: &[u8]) -> Self {
        let hash = hash_challenge(&ack_challenge);
        Self {
            ack_challenge: Some(ack_challenge.clone()),
            signature: Signature::sign_hash(&hash, private_key),
        }
    }

    pub fn solve(&self, secret: &[u8]) -> bool {
        self.ack_challenge
            .as_ref()
            .expect("challenge not valid")
            .eq(&HalfKey::new(secret).to_challenge())
    }

    pub fn verify(public_key: &PublicKey, signature: &Signature, challenge: &HalfKeyChallenge) -> bool {
        let hash = hash_challenge(challenge);
        signature.verify_hash_with_pubkey(&hash, public_key)
    }

    pub fn size() -> usize {
        Self::SIZE
    }

    pub fn validate(&mut self, ack_challenge: HalfKeyChallenge, public_key: &PublicKey) -> bool {
        if self.ack_challenge.is_some() || Self::verify(public_key, &self.signature, &ack_challenge) {
            self.ack_challenge = Some(ack_challenge);
            true
        } else {
            false
        }
    }
}

impl BinarySerializable<'_> for AcknowledgementChallenge {
    const SIZE: usize = Signature::SIZE;

    fn deserialize(data: &[u8]) -> errors::Result<Self> {
        if data.len() == Self::SIZE {
            Ok(AcknowledgementChallenge {
                ack_challenge: None,
                signature: Signature::deserialize(data)?,
            })
        } else {
            Err(ParseError)
        }
    }

    fn serialize(&self) -> Box<[u8]> {
        assert!(self.ack_challenge.is_some(), "challenge is invalid");
        self.signature.raw_signature()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PendingAcknowledgement {
    WaitingAsSender,
    WaitingAsRelayer(UnacknowledgedTicket),
}

impl PendingAcknowledgement {
    const SENDER_PREFIX: u8 = 0;
    const RELAYER_PREFIX: u8 = 1;
}

impl BinarySerializable<'_> for PendingAcknowledgement {
    const SIZE: usize = 1;

    fn deserialize(data: &[u8]) -> errors::Result<Self> {
        if data.len() >= Self::SIZE {
            match data[0] {
                Self::SENDER_PREFIX => Ok(WaitingAsSender),
                Self::RELAYER_PREFIX => Ok(WaitingAsRelayer(UnacknowledgedTicket::deserialize(&data[1..])?)),
                _ => Err(ParseError),
            }
        } else {
            Err(ParseError)
        }
    }

    fn serialize(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        match &self {
            WaitingAsSender => ret.push(Self::SENDER_PREFIX),
            WaitingAsRelayer(unacknowledged) => {
                ret.push(Self::RELAYER_PREFIX);
                ret.extend_from_slice(&unacknowledged.serialize());
            }
        }
        ret.into_boxed_slice()
    }
}

#[cfg(test)]
pub mod test {
    use ethnum::u256;
    use hex_literal::hex;
    use core_crypto::types::{Challenge, CurvePoint, HalfKey, PublicKey};
    use utils_types::primitives::{Address, Balance, BalanceType, U256};
    use crate::acknowledgement::{Acknowledgement, AcknowledgementChallenge, PendingAcknowledgement, UnacknowledgedTicket};
    use utils_types::traits::BinarySerializable;
    use crate::channels::Ticket;

    // TODO: Add tests to all remaining types

    #[test]
    fn test_pending_ack() {
        assert_eq!(
            PendingAcknowledgement::WaitingAsSender,
            PendingAcknowledgement::deserialize(&PendingAcknowledgement::WaitingAsSender.serialize()).unwrap()
        );
    }

    #[test]
    fn test_acknowledgement_challenge() {
        let hkc = HalfKey::new(&hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa")).to_challenge();
        let pk = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
        let pub_key = PublicKey::from_privkey(&pk).unwrap();

        let mut akc1 = AcknowledgementChallenge::new(&hkc, &pk);
        assert!(akc1.validate(hkc.clone(), &pub_key));

        let mut akc2 = AcknowledgementChallenge::deserialize(&akc1.serialize()).unwrap();
        assert!(akc2.validate(hkc.clone(), &pub_key));

        assert_eq!(akc1, akc2);
    }

    #[test]
    fn test_acknowledgement() {
        let pk_1 = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
        let pub_key_1 = PublicKey::from_privkey(&pk_1).unwrap();

        let pk_2 = hex!("4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b");
        let pub_key_2 = PublicKey::from_privkey(&pk_2).unwrap();

        let ack_key = HalfKey::new(&hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"));

        let akc1 = AcknowledgementChallenge::new(&ack_key.to_challenge(), &pk_1);

        let mut ack1 = Acknowledgement::new(akc1, ack_key, &pk_2);
        assert!(ack1.validate(&pub_key_1, &pub_key_2));

        let mut ack2 = Acknowledgement::deserialize(&ack1.serialize()).unwrap();
        assert!(ack2.validate(&pub_key_1, &pub_key_2));

        assert_eq!(ack1, ack2);
    }

    #[test]
    fn test_unacknowledged_ticket() {
        let pk_1 = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
        let pub_key_1 = PublicKey::from_privkey(&pk_1).unwrap();

        let hk1 = HalfKey::new(&hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"));
        let hk2 = HalfKey::new(&hex!("4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b"));
        let cp1: CurvePoint = hk1.to_challenge().into();
        let cp2: CurvePoint = hk2.to_challenge().into();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let inverse_win_prob = u256::new(1u128); // 100 %
        let price_per_packet = u256::new(10000000000000000u128); // 0.01 HOPR
        let path_pos = 5;

        let ticket1 = Ticket::new(
            Address::new(&[0u8; Address::SIZE]),
            Some(Challenge::from(cp_sum)),
            U256::new("1"),
            U256::new("2"),
            Balance::new(
                inverse_win_prob * price_per_packet * path_pos as u128,
                BalanceType::HOPR,
            ),
            U256::from_inverse_probability(&inverse_win_prob).unwrap(),
            U256::new("4"),
            &pk_1,
        );

        let unack1 = UnacknowledgedTicket::new(ticket1, hk1, pub_key_1);
        assert!(unack1.verify_signature());
        assert!(unack1.verify_challenge(&hk2).unwrap());

        let unack2 = UnacknowledgedTicket::deserialize(&unack1.serialize()).unwrap();
        assert_eq!(unack1, unack2);
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::acknowledgement::{AcknowledgedTicket, Acknowledgement, AcknowledgementChallenge, UnacknowledgedTicket};
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::traits::BinarySerializable;
    use wasm_bindgen::prelude::*;
    use core_crypto::types::{HalfKey, Response};

    #[wasm_bindgen]
    pub struct PendingAcknowledgement {
        w: super::PendingAcknowledgement
    }

    #[wasm_bindgen]
    impl PendingAcknowledgement {
        #[wasm_bindgen(constructor)]
        pub fn new(is_sender: bool, ticket: Option<UnacknowledgedTicket>) -> Self {
            if is_sender {
                Self {
                    w: super::PendingAcknowledgement::WaitingAsSender
                }
            } else {
                Self {
                    w: super::PendingAcknowledgement::WaitingAsRelayer(ticket.unwrap())
                }
            }
        }

        pub fn is_msg_sender(&self) -> bool {
            match &self.w {
                super::PendingAcknowledgement::WaitingAsSender => true,
                super::PendingAcknowledgement::WaitingAsRelayer(_) => false
            }
        }

        pub fn ticket(&self) -> Option<UnacknowledgedTicket> {
            match &self.w {
                super::PendingAcknowledgement::WaitingAsSender => None,
                super::PendingAcknowledgement::WaitingAsRelayer(ticket) => Some(ticket.clone())
            }
        }
    }

    #[wasm_bindgen]
    impl UnacknowledgedTicket {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<UnacknowledgedTicket> {
            ok_or_jserr!(Self::deserialize(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.serialize()
        }

        #[wasm_bindgen(js_name = "get_response")]
        pub fn _get_response(&self, acknowledgement: &HalfKey) -> JsResult<Response> {
            ok_or_jserr!(self.get_response(acknowledgement))
        }

        #[wasm_bindgen(js_name = "verify_challenge")]
        pub fn _verify_challenge(&self, acknowledgement: &HalfKey) -> JsResult<bool> {
            ok_or_jserr!(self.verify_challenge(acknowledgement))
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &UnacknowledgedTicket) -> bool { self.eq(other) }
    }

    #[wasm_bindgen]
    impl AcknowledgedTicket {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<AcknowledgedTicket> {
            ok_or_jserr!(Self::deserialize(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.serialize()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &AcknowledgedTicket) -> bool { self.eq(other) }
    }

    #[wasm_bindgen]
    impl AcknowledgementChallenge {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<AcknowledgementChallenge> {
            ok_or_jserr!(Self::deserialize(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.serialize()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &AcknowledgementChallenge) -> bool { self.eq(other) }
    }

    #[wasm_bindgen]
    impl Acknowledgement {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<Acknowledgement> {
            ok_or_jserr!(Self::deserialize(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.serialize()
        }

        #[wasm_bindgen(js_name = "eq")]
        pub fn _eq(&self, other: &Acknowledgement) -> bool { self.eq(other) }
    }
}
