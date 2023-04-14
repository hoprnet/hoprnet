use core_crypto::primitives::{DigestLike, SimpleDigest};
use core_crypto::types::{HalfKey, HalfKeyChallenge, Hash, PublicKey, Response, Signature};
use utils_types::errors;
use utils_types::errors::GeneralError::ParseError;
use utils_types::traits::BinarySerializable;
use crate::channels::Ticket;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct Acknowledgement {
    ack_signature: Signature,
    challenge_signature: Signature,
    ack_key_share: HalfKey,
    validated: bool
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Acknowledgement {
    pub fn new(ack_challenge: AcknowledgementChallenge, ack_key_share: HalfKey, private_key: &[u8]) -> Self {
        let mut digest = SimpleDigest::default();
        digest.update(&ack_challenge.serialize());
        digest.update(&ack_key_share.serialize());

        Self {
            ack_signature: Signature::sign_hash(&digest.finalize(), private_key),
            challenge_signature: ack_challenge.signature,
            ack_key_share,
            validated: true
        }
    }

    /// Validates the acknowledgement. Must be called immediately after deserialization or otherwise
    /// any operations with the deserialized acknowledgment will panic.
    pub fn validate(&mut self, own_public_key: &PublicKey, sender_public_key: &PublicKey) -> bool {
        let mut digest = SimpleDigest::default();
        digest.update(&self.ack_key_share.to_challenge().serialize());
        self.validated = self.challenge_signature.verify_hash_with_pubkey(&digest.finalize(), own_public_key);

        digest.update(&self.challenge_signature.serialize());
        digest.update(&self.ack_key_share.serialize());
        self.validated = self.validated && self.ack_signature.verify_hash_with_pubkey(&digest.finalize(), sender_public_key);

        self.validated
    }

    pub fn ack_challenge(&self) -> HalfKeyChallenge {
        assert!(self.validated, "acknowledgement not validated");
        self.ack_key_share.to_challenge()
    }
}

impl BinarySerializable<'_> for Acknowledgement {
    const SIZE: usize = Signature::SIZE + AcknowledgementChallenge::SIZE + HalfKey::SIZE;

    fn deserialize(data: &[u8]) -> errors::Result<Self> {
        let mut buf = Vec::from(data);
        if data.len() == Self::SIZE {
            let ack_signature = Signature::deserialize(buf.drain(..Signature::SIZE).as_ref())?;
            let challenge_signature = Signature::deserialize(buf.drain(..AcknowledgementChallenge::SIZE).as_ref())?;
            let ack_key_share = HalfKey::deserialize(buf.drain(..HalfKey::SIZE).as_ref())?;
            Ok(Self { ack_signature, challenge_signature, ack_key_share, validated: false })
        } else {
            Err(ParseError)
        }
    }

    fn serialize(&self) -> Box<[u8]> {
        assert!(self.validated, "acknowledgement not validated");
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.ack_signature.serialize());
        ret.extend_from_slice(&self.challenge_signature.serialize());
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
    pub fn new(ack_challenge: &HalfKeyChallenge, private_key: &[u8]) -> Self {
        let hash = hash_challenge(&ack_challenge);
        Self { ack_challenge: Some(ack_challenge.clone()), signature: Signature::sign_hash(&hash, private_key) }
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
                signature: Signature::deserialize(data)?
            })
        } else {
            Err(ParseError)
        }
    }

    fn serialize(&self) -> Box<[u8]> {
        assert!(self.ack_challenge.is_some(), "challenge is invalid");
        self.signature.serialize()
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use core_crypto::types::{Hash, PublicKey, Response};
    use crate::acknowledgment::AcknowledgedTicket;
    use crate::channels::Ticket;

    #[wasm_bindgen]
    impl AcknowledgedTicket {
        #[wasm_bindgen(constructor)]
        pub fn new(ticket: Ticket, response: Response, pre_image: Hash, signer: PublicKey) -> Self {
            AcknowledgedTicket {
                ticket,
                response,
                pre_image,
                signer,
            }
        }
    }
}