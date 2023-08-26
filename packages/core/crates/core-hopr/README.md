# core-hopr

Implements the main public interface for the core HOPR library - that is the `Hopr` type.

Tha application representing the entire core functionality, including:

- p2p
- transport
- packet construction
- mixing
- strategies
- crypto

## Detail

The package contains a core application package definition represented by the `CoreApp` object consisting of:

1. a collection of tools that can be used by the external caller to trigger events in the core application (`HoprTools`)
2. an aggregated callable future that encompasses all internal individually spawned processes running in the background

To minimize and focus the WASM interface into the `core-hopr` package, all external interfaces used either by the `HoprTools`
or any other sub-component are located inside the [`adaptors`](./src/adaptors/) directory. This desing allows for creating
relatively isolated components in the `core` package itself, but retains the ability to glue the pure non-wasm Rust code
into the `wasm-bindgen` enabled Rust layer in a single place.

The `core-hopr` applications then aggregates and manages the construction, setup, binding and priming of individual components,
offering a fully prepared setup for the user JS layer.
