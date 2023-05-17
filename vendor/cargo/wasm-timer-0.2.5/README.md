# Wasm-timer

Exports the `Instant`, `Delay`, `Interval` and `Timeout` structs.

On non-WASM targets, this re-exports the types from `tokio-timer`.
On WASM targets, this uses `web-sys` to implement their functionalities.
