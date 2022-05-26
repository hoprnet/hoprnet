This directory contains Rust WASM modules for HOPR.

## When adding a new Rust WASM module
1) `wasm-pack new my-module`
2) add `my-module` member in `packages/wasm/Cargo.toml`
3) add `my-module` to `PACKAGES` variable in `packages/wasm/Makefile`
4) run `make all` for the first time from `packages/wasm`.