#!/usr/bin/env bash

set -euo pipefail

export FOUNDRY_PROFILE=build-info-v1
forge build --force

rm -rf test_artifacts
mkdir -p test_artifacts/build-info-v1
mv out/build-info/*.json test_artifacts/build-info-v1

export FOUNDRY_PROFILE=build-info-v2
forge test -vvv --ffi --force

export FOUNDRY_PROFILE=build-info-v2-bad
forge test -vvv --ffi --force

export FOUNDRY_PROFILE=build-info-v2-reference-contract
forge test -vvv --ffi --force