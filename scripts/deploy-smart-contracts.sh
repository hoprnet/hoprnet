#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

test -z "${NETWORK:-}" && (echo "Missing environment variable NETWORK"; exit 1)

# go to ethereum package
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

cd "${mydir}/../packages/ethereum"

# deploy smart contracts
yarn hardhat deploy --network "${NETWORK}"
