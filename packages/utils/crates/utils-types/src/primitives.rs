use ethnum::{u256, AsU256};
use std::ops::{Add, Sub};
use std::string::ToString;
use crate::errors::GeneralError;
use crate::errors::GeneralError::ParseError;

pub const ADDRESS_LENGTH: usize = 20;

#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Address {
    addr: [u8; ADDRESS_LENGTH],
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Address {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), ADDRESS_LENGTH, "invalid length");
        let mut ret = Address {
            addr: [0u8; ADDRESS_LENGTH]
        };
        ret.addr.copy_from_slice(bytes);
        ret
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.addr)
    }

    pub fn serialize(&self) -> Box<[u8]> {
        self.addr.into()
    }

    pub fn to_string(&self) -> String {
        self.to_hex()
    }

    pub fn eq(&self, other: &Address) -> bool {
        self.addr.eq(&other.addr)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub enum BalanceType {
    Native,
    HOPR
}

#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Balance {
    value: u256,
    balance_type: BalanceType
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Balance {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn from_str(value: &str, balance_type: BalanceType) -> Self {
        Balance {
            value: u256::from_str_radix(value, 10).unwrap(),
            balance_type,
        }
    }

    pub fn balance_type(&self) -> BalanceType {
        self.balance_type
    }

    pub fn to_hex(&self) -> String { hex::encode(self.value().to_be_bytes()) }

    pub fn to_string(&self) -> String {
        self.value.to_string()
    }

    pub fn eq(&self, other: &Balance) -> bool {
        assert_eq!(self.balance_type(), other.balance_type());
        self.value().eq(other.value())
    }

    pub fn serialize_value(&self) -> Box<[u8]> {
        Box::new(self.value().to_be_bytes())
    }

    pub fn lt(&self, other: &Balance) -> bool {
        assert_eq!(self.balance_type(), other.balance_type());
        self.value().lt(other.value())
    }

    pub fn lte(&self, other: &Balance) -> bool {
        assert_eq!(self.balance_type(), other.balance_type());
        self.value().lt(other.value()) || self.value().eq(other.value())
    }

    pub fn gt(&self, other: &Balance) -> bool {
        assert_eq!(self.balance_type(), other.balance_type());
        self.value().gt(other.value())
    }

    pub fn gte(&self, other: &Balance) -> bool {
        assert_eq!(self.balance_type(), other.balance_type());
        self.value().gt(other.value()) || self.value().eq(other.value())
    }

    pub fn add(&self, other: &Balance) -> Self {
        assert_eq!(self.balance_type(), other.balance_type());
        Balance {
            value: self.value().add(other.value()),
            balance_type: self.balance_type
        }
    }

    pub fn iadd(&self, amount: u64) -> Self {
        Balance {
            value: self.value().add(amount.as_u256()),
            balance_type: self.balance_type
        }
    }

    pub fn sub(&self, other: &Balance) -> Self {
        assert_eq!(self.balance_type(), other.balance_type());
        Balance {
            value: self.value().sub(other.value()),
            balance_type: self.balance_type
        }
    }

    pub fn isub(&self, amount: u64) -> Self {
        Balance {
            value: self.value().sub(amount.as_u256()),
            balance_type: self.balance_type,
        }
    }
}

impl Balance {
    pub fn new(value: u256, balance_type: BalanceType) -> Self {
        Balance {
            value, balance_type
        }
    }

    pub fn value(&self) -> &u256 {
        &self.value
    }

    pub fn deserialize(data: &[u8], balance_type: BalanceType) -> Result<Balance, GeneralError> {
        Ok(Balance {
            value: u256::from_be_bytes(
                data.try_into().map_err(|_| ParseError)?),
            balance_type
        })
    }
}

pub const ETHEREUM_CHALLENGE_LENGTH: usize = 20;

#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct EthereumChallenge {
    challenge: [u8; ETHEREUM_CHALLENGE_LENGTH]
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl EthereumChallenge {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(data: &[u8]) -> Self {
        assert_eq!(data.len(), ETHEREUM_CHALLENGE_LENGTH);

        let mut ret = EthereumChallenge {
            challenge: [0u8; ETHEREUM_CHALLENGE_LENGTH]
        };
        ret.challenge.copy_from_slice(data);
        ret
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.challenge)
    }

    pub fn serialize(&self) -> Box<[u8]> {
        self.challenge.into()
    }

    pub fn eq(&self, other: &EthereumChallenge) -> bool {
        self.challenge.eq(&other.challenge)
    }
}

impl EthereumChallenge {
    pub fn deserialize(data: &[u8]) -> Result<EthereumChallenge, GeneralError> {
        if data.len() == ETHEREUM_CHALLENGE_LENGTH {
            Ok(EthereumChallenge::new(data))
        } else {
            Err(ParseError)
        }
    }
}

pub const HASH_LENGTH: usize = 32;

#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Hash {
    hash: [u8; HASH_LENGTH],
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Hash {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(hash: &[u8]) -> Self {
        assert_eq!(hash.len(), HASH_LENGTH, "invalid length");
        let mut ret = Hash {
            hash: [0u8; HASH_LENGTH]
        };
        ret.hash.copy_from_slice(hash);
        ret
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.hash)
    }

    pub fn serialize(&self) -> Box<[u8]> {
        self.hash.into()
    }

    pub fn eq(&self, other: &Hash) -> bool {
        self.hash.eq(&other.hash)
    }
}

// TODO: Move all Signature related stuff to core-crypto once merged
pub const SIGNATURE_LENGTH: usize = 64;

#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Signature {
    signature: [u8; SIGNATURE_LENGTH],
    recovery: u8,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Signature {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(raw_bytes: &[u8], recovery: u8) -> Signature {
        assert_eq!(raw_bytes.len(), SIGNATURE_LENGTH, "invalid length");
        assert!(recovery <= 1, "invalid recovery bit");
        let mut ret = Self {
            signature: [0u8; SIGNATURE_LENGTH],
            recovery
        };
        ret.signature.copy_from_slice(raw_bytes);
        ret
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.signature)
    }

    pub fn raw_signature(&self) -> Box<[u8]> {
        self.signature.into()
    }

    pub fn serialize(&self) -> Box<[u8]> {
        let mut compressed = Vec::from(self.signature);
        compressed[SIGNATURE_LENGTH/2] &= 0x7f;
        compressed[SIGNATURE_LENGTH/2] |= self.recovery << 7;
        compressed.into_boxed_slice()
    }
}

impl Signature {
    pub fn deserialize(signature: &[u8]) -> Result<Signature, GeneralError> {
        if signature.len() == SIGNATURE_LENGTH {
            let mut ret = Signature {
                signature: [0u8; SIGNATURE_LENGTH],
                recovery: if signature[SIGNATURE_LENGTH/2]&0x80 != 0 { 1 } else { 0 }
            };
            ret.signature.copy_from_slice(signature);
            ret.signature[SIGNATURE_LENGTH/2] &= 0x7f;

            Ok(ret)
        } else {
            Err(ParseError)
        }
    }
}


/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balance_tests() {
        let b = Balance::from_str("10", BalanceType::HOPR).unwrap();
        assert_eq!("10".to_string(), b.to_string());
    }

    #[test]
    fn signature_tests() {

    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::wasm_bindgen;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use crate::primitives::{Balance, BalanceType, EthereumChallenge, Signature};

    #[wasm_bindgen]
    impl Balance {
        pub fn deserialize_balance(data: &[u8], balance_type: BalanceType) -> JsResult<Balance> {
            ok_or_jserr!(Balance::deserialize(data, balance_type))
        }
    }

    #[wasm_bindgen]
    impl EthereumChallenge {
        pub fn deserialize_challenge(data: &[u8]) -> JsResult<EthereumChallenge> {
            ok_or_jserr!(EthereumChallenge::deserialize(data))
        }
    }

    #[wasm_bindgen]
    impl Signature {
        pub fn deserialize_signature(signature: &[u8]) -> JsResult<Signature> {
            ok_or_jserr!(Signature::deserialize(signature))
        }
    }
}
