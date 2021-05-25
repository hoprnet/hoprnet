#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

test -z "${PRIVATE_KEY:-}" && (echo "Missing environment variable PRIVATE_KEY"; exit 1)
test -z "${QUIKNODE_KEY:-}" && (echo "Missing environment variable QUIKNODE_KEY"; exit 1)
test -z "${NETWORK:-}" && (echo "Missing environment variable NETWORK"; exit 1)

# go to ethereum package
dir=$(dirname $(readlink -f $0))
cd "${dir}/../packages/ethereum"

# deploy smart contracts
yarn deploy --network "${NETWORK}"
