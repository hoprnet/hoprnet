use ethnum::u256;

pub struct PublicKey {
    data: Box<[u8]>
}

pub struct Balance {
    value: u256,
    symbol: &'static str
}

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;

}

/// Module for WASM wrappers of Rust code
#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct Balance {
        #[wasm_bindgen(skip)]
        pub w: super::Balance
    }
}

