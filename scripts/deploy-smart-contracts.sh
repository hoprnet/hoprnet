#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

test -z "${NETWORK:-}" && (echo "Missing environment variable NETWORK"; exit 1)

declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

cd "${mydir}/../"

# deploy smart contracts
yarn workspace @hoprnet/hopr-ethereum hardhat deploy --network "${NETWORK}"
