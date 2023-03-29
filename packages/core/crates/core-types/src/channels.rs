use core_crypto::types::{Challenge, Hash, PublicKey, Signature};
use enum_iterator::{all, Sequence};
use ethnum::u256;
use serde_repr::*;
use std::ops::{Div, Mul, Sub};
use utils_types::errors::{GeneralError::ParseError, Result};
use utils_types::primitives::{Address, Balance, BalanceType, EthereumChallenge, U256};

#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;

#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;
use utils_types::traits::BinarySerializable;

/// Describes status of a channel
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr, Sequence)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub enum ChannelStatus {
    Closed = 0,
    WaitingForCommitment = 1,
    Open = 2,
    PendingToClose = 3,
}

impl ChannelStatus {
    pub fn from_byte(byte: u8) -> Option<Self> {
        all::<ChannelStatus>().find(|v| v.to_byte() == byte)
    }

    pub fn to_byte(&self) -> u8 {
        *self as u8
    }

    pub fn to_string(&self) -> String {
        match self {
            ChannelStatus::Closed => "Closed",
            ChannelStatus::WaitingForCommitment => "WaitingForCommitment",
            ChannelStatus::Open => "Open",
            ChannelStatus::PendingToClose => "PendingToClose",
        }
        .to_string()
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

/// Overall description of a channel
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct ChannelEntry {
    pub source: PublicKey,
    pub destination: PublicKey,
    pub balance: Balance,
    pub commitment: Hash,
    pub ticket_epoch: U256,
    pub ticket_index: U256,
    pub status: ChannelStatus,
    pub channel_epoch: U256,
    pub closure_time: U256,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl ChannelEntry {
    /// Generates the ticket ID using the source and destination address
    pub fn get_id(&self) -> Hash {
        generate_channel_id(&self.source.to_address(), &self.destination.to_address())
    }

    /// Checks if the closure time of this channel has passed.
    pub fn closure_time_passed(&self) -> bool {
        let now_seconds = current_timestamp() / 1000;
        self.closure_time.value().lt(&u256::from(now_seconds))
    }

    /// Calculates the remaining channel closure grace period.
    pub fn remaining_closure_time(&self) -> u64 {
        let now_seconds = u256::from(current_timestamp());
        if now_seconds.ge(self.closure_time.value()) {
            now_seconds.sub(self.closure_time.value()).as_u64()
        } else {
            0
        }
    }
}

impl BinarySerializable for ChannelEntry {
    const SIZE: usize = PublicKey::SIZE_UNCOMPRESSED
        + PublicKey::SIZE_UNCOMPRESSED
        + Balance::SIZE
        + Hash::SIZE
        + U256::SIZE
        + U256::SIZE
        + 1
        + U256::SIZE
        + U256::SIZE;

    fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() == Self::SIZE {
            let mut b = Vec::from(data);
            let source = PublicKey::deserialize(b.drain(0..PublicKey::SIZE_UNCOMPRESSED).as_ref())?;
            let destination = PublicKey::deserialize(b.drain(0..PublicKey::SIZE_UNCOMPRESSED).as_ref())?;
            let balance = Balance::deserialize(b.drain(0..Balance::SIZE).as_ref(), BalanceType::HOPR)?;
            let commitment = Hash::deserialize(b.drain(0..Hash::SIZE).as_ref())?;
            let ticket_epoch = U256::deserialize(b.drain(0..U256::SIZE).as_ref())?;
            let ticket_index = U256::deserialize(b.drain(0..U256::SIZE).as_ref())?;
            let status = ChannelStatus::from_byte(b.drain(0..1).as_ref()[0]).ok_or(ParseError)?;
            let channel_epoch = U256::deserialize(b.drain(0..U256::SIZE).as_ref())?;
            let closure_time = U256::deserialize(b.drain(0..U256::SIZE).as_ref())?;
            Ok(Self {
                source,
                destination,
                balance,
                commitment,
                ticket_epoch,
                ticket_index,
                status,
                channel_epoch,
                closure_time,
            })
        } else {
            Err(ParseError)
        }
    }

    fn serialize(&self) -> Box<[u8]> {
        let mut ret = Vec::<u8>::with_capacity(Self::SIZE);
        ret.extend_from_slice(self.source.serialize(false).as_ref());
        ret.extend_from_slice(self.destination.serialize(false).as_ref());
        ret.extend_from_slice(self.balance.serialize_value().as_ref());
        ret.extend_from_slice(self.commitment.serialize().as_ref());
        ret.extend_from_slice(self.ticket_epoch.serialize().as_ref());
        ret.extend_from_slice(self.ticket_index.serialize().as_ref());
        ret.push(self.status as u8);
        ret.extend_from_slice(self.channel_epoch.serialize().as_ref());
        ret.extend_from_slice(self.closure_time.serialize().as_ref());
        ret.into_boxed_slice()
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn generate_channel_id(source: &Address, destination: &Address) -> Hash {
    Hash::create(&[&source.serialize(), &destination.serialize()])
}

/// Contains a response upon ticket acknowledgement
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Response {
    response: [u8; Self::SIZE],
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Response {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(data: &[u8]) -> Self {
        assert_eq!(data.len(), Self::SIZE);
        let mut ret = Response {
            response: [0u8; Self::SIZE],
        };
        ret.response.copy_from_slice(data);
        ret
    }
}

impl BinarySerializable for Response {
    const SIZE: usize = 32;

    fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() == Self::SIZE {
            Ok(Response::new(data))
        } else {
            Err(ParseError)
        }
    }

    fn serialize(&self) -> Box<[u8]> {
        self.response.into()
    }
}

/// Contains the overall description of a ticket with a signature
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct Ticket {
    pub counterparty: Address,
    pub challenge: EthereumChallenge,
    pub epoch: U256,
    pub index: U256,
    pub amount: Balance,
    pub win_prob: U256,
    pub channel_epoch: U256,
    pub signature: Signature,
}

/// Prefix message with "\x19Ethereum Signed Message:\n {length} {message}" and returns its hash
/// Keccak256 is used as the underlying digest.
pub fn ethereum_signed_hash<T: AsRef<[u8]>>(message: T) -> Hash {
    const PREFIX: &str = "\x19Ethereum Signed Message:\n";

    let message = message.as_ref();
    let len = message.len();
    let len_string = len.to_string();

    let mut eth_message = Vec::with_capacity(PREFIX.len() + len_string.len() + len);
    eth_message.extend_from_slice(PREFIX.as_bytes());
    eth_message.extend_from_slice(len_string.as_bytes());
    eth_message.extend_from_slice(message);

    Hash::create(&[&eth_message])
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Ticket {
    fn serialize_unsigned_aux(
        counterparty: &Address,
        challenge: &EthereumChallenge,
        epoch: &U256,
        amount: &Balance,
        win_prob: &U256,
        index: &U256,
        channel_epoch: &U256,
    ) -> Vec<u8> {
        let mut ret = Vec::<u8>::with_capacity(Self::SIZE);
        ret.extend_from_slice(&counterparty.serialize());
        ret.extend_from_slice(&challenge.serialize());
        ret.extend_from_slice(&epoch.serialize());
        ret.extend_from_slice(&amount.serialize_value());
        ret.extend_from_slice(&win_prob.serialize());
        ret.extend_from_slice(&index.serialize());
        ret.extend_from_slice(&channel_epoch.serialize());
        ret
    }

    /// Creates a new Ticket given the raw Challenge and signs it using the given key.
    pub fn create(
        counterparty: Address,
        challenge: Challenge,
        epoch: U256,
        index: U256,
        amount: Balance,
        win_prob: U256,
        channel_epoch: U256,
        signing_key: &[u8],
    ) -> Self {
        let encoded_challenge = challenge.to_ethereum_challenge();
        let hashed_ticket = Hash::create(&[&Self::serialize_unsigned_aux(
            &counterparty,
            &encoded_challenge,
            &epoch,
            &amount,
            &win_prob,
            &index,
            &channel_epoch,
        )]);
        let msg = ethereum_signed_hash(hashed_ticket.serialize()).serialize();
        let signature = Signature::sign_message(&msg, signing_key);

        Self {
            counterparty,
            challenge: encoded_challenge,
            epoch,
            index,
            amount,
            win_prob,
            channel_epoch,
            signature,
        }
    }

    /// Serializes the ticket except the signature
    pub fn serialize_unsigned(&self) -> Box<[u8]> {
        Self::serialize_unsigned_aux(
            &self.counterparty,
            &self.challenge,
            &self.epoch,
            &self.amount,
            &self.win_prob,
            &self.index,
            &self.channel_epoch,
        )
        .into_boxed_slice()
    }

    /// Computes Ethereum signature hash of the ticket
    pub fn get_hash(&self) -> Hash {
        ethereum_signed_hash(Hash::create(&[&self.serialize_unsigned()]).serialize())
    }

    /// Recovers the signer public key from the embedded ticket signature.
    /// This is possible due this specific instantiation of the ECDSA over the secp256k1 curve.
    pub fn recover_signer(&self) -> PublicKey {
        PublicKey::from_signature(&self.get_hash().serialize(), &self.signature)
            .expect("invalid signature on ticket, public key not recoverable")
    }

    /// Verifies the signature of this ticket
    pub fn verify(&self, public_key: &PublicKey) -> bool {
        self.recover_signer().eq(public_key)
    }

    /// Computes a candidate check value to verify if this ticket is winning
    pub fn get_luck(&self, preimage: &Hash, channel_response: &Response) -> U256 {
        U256::deserialize(
            &Hash::create(&[
                &self.get_hash().serialize(),
                &preimage.serialize(),
                &channel_response.serialize(),
            ])
            .serialize(),
        )
        .unwrap()
    }

    /// Decides whether a ticket is a win or not.
    /// Note that this mimics the on-chain logic.
    /// Purpose of the function is to check the validity of ticket before we submit it to the blockchain.
    pub fn is_winning(&self, preimage: &Hash, channel_response: &Response, win_prob: &U256) -> bool {
        let luck = self.get_luck(preimage, channel_response);
        luck.value().le(win_prob.value())
    }

    /// Based on the price of this ticket, determines the path position (hop number) this ticket
    /// relates to.
    pub fn get_path_position(&self, price_per_packet: &U256, inverse_ticket_win_prob: &U256) -> u8 {
        let base_unit = price_per_packet.value().mul(inverse_ticket_win_prob.value());
        self.amount.value().div(base_unit).as_u8()
    }
}

impl BinarySerializable for Ticket {
    const SIZE: usize =
        Address::SIZE + EthereumChallenge::SIZE + 2 * U256::SIZE + Balance::SIZE + 2 * U256::SIZE + Signature::SIZE;

    fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() == Self::SIZE {
            let mut b = Vec::from(data);
            let counterparty = Address::deserialize(b.drain(0..Address::SIZE).as_ref())?;
            let challenge = EthereumChallenge::deserialize(b.drain(0..EthereumChallenge::SIZE).as_ref())?;
            let epoch = U256::deserialize(b.drain(0..U256::SIZE).as_ref())?;
            let amount = Balance::deserialize(b.drain(0..Balance::SIZE).as_ref(), BalanceType::HOPR)?;
            let win_prob = U256::deserialize(b.drain(0..U256::SIZE).as_ref())?;
            let index = U256::deserialize(b.drain(0..U256::SIZE).as_ref())?;
            let channel_epoch = U256::deserialize(b.drain(0..U256::SIZE).as_ref())?;
            let signature = Signature::deserialize(b.drain(0..Signature::SIZE).as_ref())?;

            Ok(Self {
                counterparty,
                challenge,
                epoch,
                index,
                amount,
                win_prob,
                channel_epoch,
                signature,
            })
        } else {
            Err(ParseError)
        }
    }

    fn serialize(&self) -> Box<[u8]> {
        let mut unsigned = Self::serialize_unsigned_aux(
            &self.counterparty,
            &self.challenge,
            &self.epoch,
            &self.amount,
            &self.win_prob,
            &self.index,
            &self.channel_epoch,
        );
        unsigned.extend_from_slice(&self.signature.serialize());
        unsigned.into_boxed_slice()
    }
}

#[cfg(test)]
pub mod tests {
    use core_crypto::types::{Challenge, CurvePoint, Hash, PublicKey};
    use ethnum::u256;
    use hex_literal::hex;
    use utils_types::primitives::{Address, Balance, BalanceType, U256};
    use utils_types::traits::BinarySerializable;

    use crate::channels::{ChannelEntry, ChannelStatus, Response, Ticket};

    const PUBLIC_KEY_1: [u8; 65] = hex!("0443a3958ac66a3b2ab89fcf90bc948a8b8be0e0478d21574d077ddeb11f4b1e9f2ca21d90bd66cee037255480a514b91afae89e20f7f7fa7353891cc90a52bf6e");
    const PUBLIC_KEY_2: [u8; 65] = hex!("04f16fd6701aea01032716377d52d8213497c118f99cdd1c3c621b2795cac8681606b7221f32a8c5d2ef77aa783bec8d96c11480acccabba9e8ee324ae2dfe92bb");
    const COMMITMENT: [u8; 32] = hex!("ffab46f058090de082a086ea87c535d34525a48871c5a2024f80d0ac850f81ef");

    const SGN_PRIVATE_KEY: [u8; 32] = hex!("e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8");

    #[test]
    pub fn channel_entry_test() {
        let ce1 = ChannelEntry::new(
            PublicKey::deserialize(&PUBLIC_KEY_1).unwrap(),
            PublicKey::deserialize(&PUBLIC_KEY_2).unwrap(),
            Balance::new(u256::from(10u8), BalanceType::HOPR),
            Hash::new(&COMMITMENT),
            U256::new("0"),
            U256::new("1"),
            ChannelStatus::PendingToClose,
            U256::new("3"),
            U256::new("4"),
        );

        let ce2 = ChannelEntry::deserialize(&ce1.serialize()).unwrap();
        assert_eq!(ce1, ce2, "deserialized channel entry does not match");
    }

    #[test]
    pub fn channel_status_test() {
        let cs1 = ChannelStatus::Open;
        let cs2 = ChannelStatus::from_byte(cs1.to_byte()).unwrap();

        assert!(ChannelStatus::from_byte(231).is_none());
        assert_eq!(cs1, cs2, "channel status does not match");
    }

    #[test]
    pub fn ticket_test() {
        let inverse_win_prob = u256::new(1u128); // 100 %
        let price_per_packet = u256::new(10000000000000000u128); // 0.01 HOPR
        let path_pos = 5u8;

        let curve_point = CurvePoint::deserialize(&hex!(
            "03c2aa76d6837c51337001c8b5a60473726064fc35d0a40b8f0e1f068cc8e38e10"
        ))
        .unwrap();

        let ticket1 = Ticket::create(
            Address::new(&[0u8; Address::SIZE]),
            Challenge { curve_point },
            U256::new("1"),
            U256::new("2"),
            Balance::new(
                inverse_win_prob * price_per_packet * path_pos as u128,
                BalanceType::HOPR,
            ),
            U256::from_inverse_probability(&inverse_win_prob).unwrap(),
            U256::new("4"),
            &SGN_PRIVATE_KEY,
        );

        let ticket2 = Ticket::deserialize(&ticket1.serialize()).unwrap();

        assert_eq!(ticket1, ticket2, "deserialized ticket does not match");

        let pub_key = PublicKey::from_privkey(&SGN_PRIVATE_KEY).unwrap();
        assert!(ticket1.verify(&pub_key), "failed to verify signed ticket");

        assert_eq!(
            ticket1.get_path_position(&price_per_packet.into(), &inverse_win_prob.into()),
            path_pos,
            "invalid path pos"
        );
        assert_eq!(
            ticket2.get_path_position(&price_per_packet.into(), &inverse_win_prob.into()),
            path_pos,
            "invalid path pos"
        );
    }

    #[test]
    pub fn response_test() {
        let r1 = Response::new(&[0u8; Response::SIZE]);
        let r2 = Response::deserialize(&r1.serialize()).unwrap();
        assert_eq!(r1, r2, "deserialized response does not match");
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use core_crypto::types::{Hash, PublicKey, Signature};
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::{Address, Balance, EthereumChallenge, U256};
    use utils_types::traits::{BinarySerializable, ToHex};
    use wasm_bindgen::prelude::wasm_bindgen;

    use crate::channels::{AcknowledgedTicket, ChannelEntry, ChannelStatus, Response, Ticket};

    #[wasm_bindgen]
    pub fn channel_status_to_number(status: ChannelStatus) -> u8 {
        status as u8
    }

    #[wasm_bindgen]
    pub fn number_to_channel_status(number: u8) -> Option<ChannelStatus> {
        ChannelStatus::from_byte(number)
    }

    #[wasm_bindgen]
    pub fn channel_status_to_string(status: ChannelStatus) -> String {
        status.to_string()
    }

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

    #[wasm_bindgen]
    impl ChannelEntry {
        #[wasm_bindgen(constructor)]
        pub fn new(
            source: PublicKey,
            destination: PublicKey,
            balance: Balance,
            commitment: Hash,
            ticket_epoch: U256,
            ticket_index: U256,
            status: ChannelStatus,
            channel_epoch: U256,
            closure_time: U256,
        ) -> Self {
            ChannelEntry {
                source,
                destination,
                balance,
                commitment,
                ticket_epoch,
                ticket_index,
                status,
                channel_epoch,
                closure_time,
            }
        }

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<ChannelEntry> {
            ok_or_jserr!(ChannelEntry::deserialize(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.serialize()
        }
    }

    #[wasm_bindgen]
    impl Response {
        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<Response> {
            ok_or_jserr!(Response::deserialize(data))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.serialize()
        }

        #[wasm_bindgen(js_name = "to_hex")]
        pub fn _to_hex(&self) -> String {
            self.to_hex()
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }

    #[wasm_bindgen]
    impl Ticket {
        #[wasm_bindgen(constructor)]
        pub fn new(
            counterparty: Address,
            challenge: EthereumChallenge,
            epoch: U256,
            index: U256,
            amount: Balance,
            win_prob: U256,
            channel_epoch: U256,
            signature: Signature,
        ) -> Self {
            Ticket {
                counterparty,
                challenge,
                epoch,
                index,
                amount,
                win_prob,
                channel_epoch,
                signature,
            }
        }

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(bytes: &[u8]) -> JsResult<Ticket> {
            ok_or_jserr!(Self::deserialize(bytes))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.serialize()
        }
    }
}
