use std::ops::{Add, Sub};
use ethnum::u256;

pub struct Balance {
    value: u256,
    symbol: String
}

impl Balance {

    pub fn value(&self) -> &u256 {
        &self.value
    }

    pub fn symbol(&self) -> &str {
        self.symbol.as_str()
    }

    pub fn from_str(value: &str, symbol: &str) -> Result<Self,String> {
        Ok(Balance {
            value: u256::from_str_radix(value,10).map_err(|_| "failed to parse")?,
            symbol: symbol.to_string()
        })
    }

    pub fn add(&self, other: &Balance) -> Self {
        assert_eq!(self.symbol(), other.symbol());
        Balance {
            value: self.value().add(other.value()),
            symbol: self.symbol.clone()
        }
    }

    pub fn iadd(&self, amount: u64) -> Self {
        Balance {
            value: self.value().add(u256::from(amount)),
            symbol: self.symbol.clone()
        }
    }

    pub fn sub(&self, other: &Balance) -> Self {
        assert_eq!(self.symbol(), other.symbol());
        Balance {
            value: self.value().sub(other.value()),
            symbol: self.symbol.clone()
        }
    }

    pub fn isub(&self, amount: u64) -> Self {
        Balance {
            value: self.value().sub(u256::from(amount)),
            symbol: self.symbol.clone()
        }
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.value().to_be_bytes())
    }

    pub fn serialize_value(&self) -> Box<[u8]> {
        Box::new(self.value.to_be_bytes())
    }

    pub fn deserialize(data: &[u8], symbol: &str) -> Result<Self, String> {
        Ok(Balance {
            value: u256::from_be_bytes(data.try_into().map_err(|_| "conversion error".to_string())?),
            symbol: symbol.to_string()
        })
    }

    pub fn lt(&self, other: &Balance) -> bool {
        assert_eq!(self.symbol(), other.symbol());
        self.value().lt(other.value())
    }

    pub fn lte(&self, other: &Balance) -> bool {
        assert_eq!(self.symbol(), other.symbol());
        self.value().lt(other.value()) || self.value().eq(other.value())
    }

    pub fn gt(&self, other: &Balance) -> bool {
        assert_eq!(self.symbol(), other.symbol());
        self.value().gt(other.value())
    }

    pub fn gte(&self, other: &Balance) -> bool {
        assert_eq!(self.symbol(), other.symbol());
        self.value().gt(other.value()) || self.value().eq(other.value())
    }

    pub fn eq(&self, other: &Balance) -> bool {
        assert_eq!(self.symbol(), other.symbol());
        self.value().eq(other.value())
    }
}

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;

}

/// Module for WASM wrappers of Rust code
#[cfg(feature = "wasm")]
pub mod wasm {
    use std::fmt::Display;
    use std::str::FromStr;
    use ethnum::u256;
    use wasm_bindgen::prelude::*;

    fn as_jsvalue<T>(v: T) -> JsValue where T: Display {
        JsValue::from(v.to_string())
    }

    #[wasm_bindgen]
    pub struct Balance {
        #[wasm_bindgen(skip)]
        pub w: super::Balance
    }

    impl From<super::Balance> for Balance {
        fn from(b: crate::primitives::Balance) -> Self {
            Balance {
                w: b
            }
        }
    }

    #[wasm_bindgen]
    impl Balance {
        #[wasm_bindgen(constructor)]
        pub fn new(value: &str, symbol: &str) -> Result<Balance, JsValue> {
            Ok(Self {
                w: super::Balance {
                    value: u256::from_str(value).map_err(as_jsvalue)?,
                    symbol: symbol.to_string()
                }
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

