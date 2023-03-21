# HOPR Metrics Collection

The purpose of the `utils-log` Rust crate is to create a thin Rust WASM-compatible wrapper
over the [log](https://docs.rs/log/latest/log/) library.

The goal of this wrapper is to hide the internal switching mechanism for WASM and non-WASM related environments. This crate has no meaning for the JavaScript code, as it is a Rust code layer only.

### Usage in Rust code

Add this crate as a dependency and `use` the desired macros.
 
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
