use wasm_bindgen::prelude::*;

pub mod error;
pub mod real;
pub mod file;

/// Dummy function to test WASM.
#[wasm_bindgen]
pub fn dummy_get_one() -> String {
    String::from("1")
}

// Unit tests follow

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_dummy_get_one() {
        assert_eq!(dummy_get_one().as_str(), "1");
    }
}

// NOTE: this crate cannot have the `set_console_panic_hook` function, because
// the crates using this package already have it. There can be at most 1 per WASM module.
