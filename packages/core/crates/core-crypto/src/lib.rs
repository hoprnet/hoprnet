mod utils;

pub(crate) mod dummy_rng;

pub mod derivation;
pub mod parameters;
pub mod shared_keys;
pub mod primitives;
pub mod prp;
pub mod prg;
pub mod errors;

#[allow(dead_code)]
#[wasm_bindgen]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[macro_use]
extern crate static_assertions;
extern crate core;

use wasm_bindgen::prelude::wasm_bindgen;

// Static assertions on cryptographic parameters

const_assert!(parameters::SECRET_KEY_LENGTH >= 32);
//const_assert_eq!(constants::SECRET_KEY_LENGTH, keys::KeyBytes::)

