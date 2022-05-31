This directory contains Rust WASM modules for HOPR.
Each WASM module corresponds to a Rust crate.

## Structure

The package itself consists of two parts:

- the Typescript code - mostly wrappers for JS functions that cannot be called from WASM modules (IO, sockets,...), sometimes called `REAL` - see [#3823](https://github.com/hoprnet/hoprnet/issues/3823).
- directories containing Rust crates (currently only `hopr-wasm-common`)

During build, all the Rust crates are build first using `make all` (see `Makefile` in `packages/wasm`), then the Typescript sources are built as well.

## Usage in Typescript code

Add dependency to `@hoprnet/hopr-wasm` to your `package.json`

```typescript
import * as wasm from '@hoprnet/hopr-wasm'

if (wasm.common.dummy_get_one() === '1') {
  console.log('It works!!')
}
```

## <a id="adding_mod"></a> Adding a new Rust WASM module

1. `cd packages/wasm && wasm-pack new my-module`, this will create a new Rust crate for WASM.
2. add `my-module` member in `packages/wasm/Cargo.toml`
3. add `my-module` to `PACKAGES` space separated list in `packages/wasm/Makefile`
4. run `make all && make install` for the first time from `packages/wasm`.
5. export your WASM Rust crate under it's alias in `packages/wasm/src/index.ts`, e.g.: `export * as my-modules from '../lib/my-module'`

## Testing

Each WASM module can have it's own unit tests and integration tests.

- Unit tests are placed within the module file
- Integration tests are placed in `tests/` directory next to `src/`

## Adding Rust WASM module support to other existing `hopr` packages

The Rust WASM code is not limited just to `@hoprnet/hopr-wasm` package. The existing tooling and structure in `@hoprnet/hopr-wasm` can be easily copied to any existing package within the monorepo.

1. Copy the `Makefile` from `packages/wasm` to `packages/<other_package>`
2. Add `"@wasm-tool/wasm-pack-plugin": "^1.1.0"` as a `devDependency` in the existing package
3. Make sure the `files` section in `package.json` contains the `lib` entry.
4. Prepend `make` to script actions in `package.json` accordingly:
   - for `build` action prepend: `make all && make install && ...`
   - for `clean` action prepend: `make clean && ...`
   - for `test` action prepend: `make test && ...`
5. Now you can add Rust crates as described in the [section above](#adding_mod).
