#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

test -z "${NETWORK:-}" && (echo "Missing environment variable NETWORK"; exit 1)

if [ $NETWORK == "xdai" ]; then
  test -z "${DEPLOYER_WALLET_PRIVATE_KEY:-}" && (echo "Missing environment variable DEPLOYER_WALLET_PRIVATE_KEY"; exit 1)
  test -z "${QUIKNODE_KEY:-}" && (echo "Missing environment variable QUIKNODE_KEY"; exit 1)
elif [ $NETWORK == "polygon" ]; then
  test -z "${DEPLOYER_WALLET_PRIVATE_KEY:-}" && (echo "Missing environment variable DEPLOYER_WALLET_PRIVATE_KEY"; exit 1)
  test -z "${INFURA_KEY:-}" && (echo "Missing environment variable INFURA_KEY"; exit 1)
elif [ $NETWORK == "goerli" ]; then
  test -z "${DEPLOYER_WALLET_PRIVATE_KEY:-}" && (echo "Missing environment variable DEPLOYER_WALLET_PRIVATE_KEY"; exit 1)
  test -z "${INFURA_KEY:-}" && (echo "Missing environment variable INFURA_KEY"; exit 1)
  test -z "${ETHERSCAN_KEY:-}" && (echo "Missing environment variable ETHERSCAN_KEY"; exit 1)
fi

# go to ethereum package
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

cd "${mydir}/../"

# deploy smart contracts
yarn workspace @hoprnet/hopr-ethereum hardhat deploy --network "${NETWORK}"
