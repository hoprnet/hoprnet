#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

build_ts_utils() {
    yarn workspace @hoprnet/hopr-utils run build
}

build_ts(){
    yarn workspace @hoprnet/hopr-ethereum run build:sol:types
    tsc --build tsconfig.build.json
    yarn workspace @hoprnet/hoprd run buildAdmin
}

build_rs() {
    yarn workspaces foreach -p --exclude hoprnet --exclude hopr-docs run build:wasm
}

build_ts_utils
build_ts & build_rs &
wait