//! This package wraps the basic functionality of logging for a hybrid WASM/Rust
//! environment. It properly encapsulates the logging facilities and uses correct
//! logging macros based on the feature turned on.
//!
//! ## WASM environment
//! Turned on by enabling the feature `wasm`.
//!
//! The crate wraps a `console.log` log message from the JavaScript environment for
//! a WASM enabled environment.
//!
//! With a `feature = "wasm"` on but under a `cfg(test)` it uses the native Rust
//! `log` crate to register the log messages. This allows turning the logging on
//! and off in the tests for easier debugging.
//! - if debugging in a crate using utils_log is necessary, the `dev-dependencies`
//! of that crate should be extended with `env_logger = "0.10.0"`
//! - the test to be debugged must contain this line as the first line of the test:
//! `let _ = env_logger::init();`
//!
//! ## Non-WASM environment
//! With the `wasm` feature not turned on the macros from the `log` crate are used.

#[cfg(feature = "wasm")]
#[macro_use]
pub mod macros;

#[cfg(feature = "wasm")]
pub mod callbacks;

pub use log::Level;

#[cfg(not(feature = "wasm"))]
pub use log::{debug, error, info, trace, warn};

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    #[allow(dead_code)]
    #[wasm_bindgen]
    pub fn utils_log_set_panic_hook() {
        // When the `console_error_panic_hook` feature is enabled, we can call the
        // `set_panic_hook` function at least once during initialization, and then
        // we will get better error messages if our code ever panics.
        //
        // For more details see
        // https://github.com/rustwasm/console_error_panic_hook#readme
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
    }

    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
    #[cfg(feature = "wee_alloc")]
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
}
