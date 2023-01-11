
/// Pure Rust function
pub fn foo() -> i32 {
    42
}

/// Unit tests of pure Rust code
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foo() {
        assert_eq!(42, foo());
    }
}

/// Module for WASM wrappers of Rust code
#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn foo() -> i32 {
        super::foo()
    }
}

