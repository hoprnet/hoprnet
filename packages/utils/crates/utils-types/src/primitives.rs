use ethnum::{u256, AsU256};
use std::ops::{Add, Sub};

pub trait BaseBalance {
    const SYMBOL: &'static str;

    fn value(&self) -> &u256;

    fn symbol(&self) -> &str {
        Self::SYMBOL
    }

    fn to_hex(&self) -> String {
        hex::encode(self.value().to_be_bytes())
    }

    fn serialize_value(&self) -> Box<[u8]> {
        Box::new(self.value().to_be_bytes())
    }

    fn lt(&self, other: &impl BaseBalance) -> bool {
        assert_eq!(self.symbol(), other.symbol());
        self.value().lt(other.value())
    }

    fn lte(&self, other: &impl BaseBalance) -> bool {
        assert_eq!(self.symbol(), other.symbol());
        self.value().lt(other.value()) || self.value().eq(other.value())
    }

    fn gt(&self, other: &impl BaseBalance) -> bool {
        assert_eq!(self.symbol(), other.symbol());
        self.value().gt(other.value())
    }

    fn gte(&self, other: &impl BaseBalance) -> bool {
        assert_eq!(self.symbol(), other.symbol());
        self.value().gt(other.value()) || self.value().eq(other.value())
    }

    fn eq(&self, other: &impl BaseBalance) -> bool {
        assert_eq!(self.symbol(), other.symbol());
        self.value().eq(other.value())
    }
}

#[derive(Clone)]
pub struct Balance {
    value: u256,
}

impl BaseBalance for Balance {
    const SYMBOL: &'static str = "mHOPR";

    fn value(&self) -> &u256 {
        &self.value
    }
}

impl ToString for Balance {
    fn to_string(&self) -> String {
        self.value.to_string()
    }
}

impl Balance {
    pub fn from_u64(value: u64) -> Self {
        Balance {
            value: value.as_u256(),
        }
    }

    pub fn zero() -> Self {
        Self::from_u64(0)
    }

    pub fn from_str(value: &str) -> Result<Self, String> {
        Ok(Balance {
            value: u256::from_str_radix(value, 10).map_err(|_| "failed to parse")?,
        })
    }

    pub fn add(&self, other: &Balance) -> Self {
        assert_eq!(self.symbol(), other.symbol());
        Balance {
            value: self.value().add(other.value()),
        }
    }

    pub fn iadd(&self, amount: u64) -> Self {
        Balance {
            value: self.value().add(amount.as_u256()),
        }
    }

    pub fn sub(&self, other: &Balance) -> Self {
        assert_eq!(self.symbol(), other.symbol());
        Balance {
            value: self.value().sub(other.value()),
        }
    }

    pub fn isub(&self, amount: u64) -> Self {
        Balance {
            value: self.value().sub(amount.as_u256()),
        }
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        Ok(Balance {
            value: u256::from_be_bytes(
                data.try_into()
                    .map_err(|_| "conversion error".to_string())?,
            ),
        })
    }
}

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balance_tests() {
        let b = Balance::from_str("10").unwrap();
        assert_eq!("10".to_string(), b.to_string());
    }
}

/// Module for WASM wrappers of Rust code
#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    use ethnum::u256;
    use std::str::FromStr;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;

    use crate::primitives::BaseBalance;

    #[wasm_bindgen]
    pub struct Balance {
        #[wasm_bindgen(skip)]
        pub w: super::Balance,
    }

    impl From<super::Balance> for Balance {
        fn from(b: crate::primitives::Balance) -> Self {
            Balance { w: b }
        }
    }

    #[wasm_bindgen]
    impl Balance {
        #[wasm_bindgen(constructor)]
        pub fn new(value: &str) -> JsResult<Balance> {
            Ok(Self {
                w: super::Balance {
                    value: ok_or_jserr!(u256::from_str(value))?,
                },
            })
        }

        pub fn value(&self) -> String {
            self.w.value().to_string()
        }

        pub fn symbol(&self) -> String {
            self.w.symbol().to_string()
        }

        pub fn add(&self, other: &Balance) -> Balance {
            self.w.add(&other.w).into()
        }

        pub fn sub(&self, other: &Balance) -> Balance {
            self.w.sub(&other.w).into()
        }

        pub fn to_hex(&self) -> String {
            self.w.to_hex()
        }

        pub fn serialize_value(&self) -> Box<[u8]> {
            self.w.serialize_value()
        }

        pub fn lt(&self, other: &Balance) -> bool {
            self.w.lt(&other.w)
        }

        pub fn lte(&self, other: &Balance) -> bool {
            self.w.lte(&other.w)
        }

        pub fn gt(&self, other: &Balance) -> bool {
            self.w.gt(&other.w)
        }

        pub fn gte(&self, other: &Balance) -> bool {
            self.w.gte(&other.w)
        }

        pub fn eq(&self, other: &Balance) -> bool {
            self.w.eq(&other.w)
        }
    }
}
