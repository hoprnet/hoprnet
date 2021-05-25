#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

test -z "${PRIVATE_KEY:-}" && (echo "Missing environment variable PRIVATE_KEY"; exit 1)
test -z "${QUIKNODE_KEY:-}" && (echo "Missing environment variable QUIKNODE_KEY"; exit 1)
test -z "${NETWORK:-}" && (echo "Missing environment variable NETWORK"; exit 1)

npx lerna exec --scope @hoprnet/hopr-ethereum -- "PRIVATE_KEY=${PRIVATE_KEY}" "QUIKNODE_KEY=${QUIKNODE_KEY}" npx hardhat deploy --network "${NETWORK}"
