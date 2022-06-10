#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

build_ts_utils() {
    tsc -p ${mydir}/packages/utils/tsconfig.json
}

build_ts(){
    yarn workspace @hoprnet/hopr-ethereum run build:sol:types
    tsc --build tsconfig.build.json
    yarn workspace @hoprnet/hoprd run buildAdmin
}

build_rs() {
    rustup target add wasm32-unknown-unknown
    cargo build --release --target wasm32-unknown-unknown
    yarn workspaces foreach -p --exclude hoprnet --exclude hopr-docs run build:wasm
}

# First build hopr-utils package because hardhat uses it
build_ts_utils
build_rs
build_ts