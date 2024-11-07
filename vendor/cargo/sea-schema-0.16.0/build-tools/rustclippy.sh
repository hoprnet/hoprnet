#!/bin/bash
set -e
if [ -d ./build-tools ]; then
    targets=(
        "Cargo.toml"
        "sea-schema-derive/Cargo.toml"
    )

    for target in "${targets[@]}"; do
        echo "cargo clippy --manifest-path ${target} --fix --allow-dirty --allow-staged"
        cargo clippy --manifest-path "${target}" --fix --allow-dirty --allow-staged
    done

    tests=(`find tests -type f -name 'Cargo.toml'`)
    for example in "${tests[@]}"; do
        echo "cargo clippy --manifest-path ${example} --fix --allow-dirty --allow-staged"
        cargo clippy --manifest-path "${example}" --fix --allow-dirty --allow-staged
    done
else
    echo "Please execute this script from the repository root."
fi
