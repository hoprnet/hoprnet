pub mod error;
pub mod file;
pub mod real;

// NOTE: this crate cannot have the `set_console_panic_hook` function, because
// the crates using this package already have it. There can be at most 1 per WASM module.
