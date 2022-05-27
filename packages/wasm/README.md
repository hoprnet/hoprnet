This directory contains Rust WASM modules for HOPR.

## When adding a new Rust WASM module

1. `wasm-pack new my-module`
2. add `my-module` member in `packages/wasm/Cargo.toml`
3. add `my-module` to `PACKAGES` in `packages/wasm/Makefile`
4. add `"@hoprnet/my-module": "link:./my-module/pkg` dependency entry to `packages/wasm/package.json`
5. run `make all` for the first time from `packages/wasm`.

## Testing
Each module can have it's own unit tests and integration tests.

- Unit tests are placed within the module file
- Integration tests are placed in `tests/` directory next to `src/`
