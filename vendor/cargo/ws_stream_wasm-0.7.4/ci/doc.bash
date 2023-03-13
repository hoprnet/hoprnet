#!/usr/bin/bash

# Only run on nightly.

# fail fast
#
set -e

# print each command before it's executed
#
set -x

export RUSTFLAGS="-D warnings"


cargo doc --all-features --no-deps

# doc tests aren't working on wasm for now...
#
# cargo test --doc --all-features --target wasm32-unknown-unknown
