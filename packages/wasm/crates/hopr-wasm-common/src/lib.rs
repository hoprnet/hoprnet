pub mod real;
mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Dummy function to test WASM.
#[wasm_bindgen]
pub fn dummy_get_one() -> String {
    String::from("1")
}

// Unit tests follow

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;
    use super::*;

    #[wasm_bindgen_test]
    fn test_dummy_get_one() {
        assert_eq!(dummy_get_one().as_str(), "1");
    }
}