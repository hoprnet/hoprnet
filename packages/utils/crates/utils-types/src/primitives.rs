use ethnum::u256;

pub struct PublicKey {
    data: Box<[u8]>
}

pub struct Balance {
    value: u256,
    symbol: String
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
            self.w.value.to_string()
        }

        pub fn symbol(&self) -> String {
            self.w.symbol.to_owned()
        }
    }
}

