# HOPR Metrics Collection

The purpose of the `utils-log` Rust crate is to create a thin Rust WASM-compatible wrapper
over the [log](https://docs.rs/log/latest/log/) library.

This crate also implements a `JsLogger` backend, which is inserted when a crate is loaded in the WASM context.
Otherwise, the crate's default logging backend will be used (such as `env_logger`).

### Usage in Rust code

Add this crate as a dependency and `use` the desired macros.

#### Installing `JsLogger` backend in WASM context

The `JsLogger` backend should be installed as soon as the crate is loaded in the WASM context.
A suitable place would be the `xyz_initialize_crate` in crate's `lib.rs` as this function is typically called immediately
after the WASM binary has been loaded.

Here's an example:

```rust
use utils_log::logger::JsLogger;

static LOGGER: JsLogger = JsLogger {};

#[allow(dead_code)]
#[wasm_bindgen]
pub fn my_crate_initialize_crate() {
    let _ = JsLogger::install(&LOGGER, None);

    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
```

Note that the default maximum logging level is `DEBUG` (if `None` is specified in the `JsLogger::install` function).

#### Example use in Rust

```rust
use utils_log::{info,warn,error};

fn main() {
    info!("This is an example of an info level log");
    warn!("This is an example of a warn level log");
    error!("This is an example of an error level log");

    println!("Any logic can go here");
}
```
