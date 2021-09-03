#!/usr/bin/env bash

set -e #u
shopt -s expand_aliases
#set -o xtrace

source scripts/environments.sh
source scripts/testnet.sh
source scripts/cleanup.sh

# ---- On Deployment -----
#
# This is run on pushes to master, or release/**
#
# ENV Variables:
# - GITHUB_REF: ie. `/refs/heads/mybranch`
# - RPC: provider address, ie `https://rpc-mainnet.matic.network`
# - RPC_NETWORK: provider network id, e.g. xdai
# - FUNDING_PRIV_KEY: funding private key, raw
# - BS_PASSWORD: database password

if [ -z "${RPC:-}" ] && [ "${RPC_NETWORK:-}" = "goerli" ]; then
  RPC="https://goerli.infura.io/v3/${INFURA_KEY}"
elif [ -z "${RPC:-}" ] && [ "${RPC_NETWORK:-}" = "xdai" ]; then
  RPC="https://still-patient-forest.xdai.quiknode.pro/f0cdbd6455c0b3aea8512fc9e7d161c1c0abf66a/"
elif [ -z "${RPC:-}" ] && [ "${RPC_NETWORK:-}" = "polygon" ]; then
  RPC="https://still-patient-hill.matic.quiknode.pro/b4ecf45eb108716acdc775404e5b6bb9a46c18d8/"
elif [ "${RPC_NETWORK:-}" != "xdai" ] && [ "${RPC_NETWORK:-}" != "goerli" ]; then
  echo "Missing supported RPC_NETWORK"
  exit 1
fi

# Get version from package.json if not already set
if [ -z "${RELEASE:-}" ]; then
  RELEASE=$(node -p -e "require('./packages/hoprd/package.json').version")
fi

# Get RELEASE_NAME, from environment
get_environment

TESTNET_NAME="$RELEASE_NAME-$(echo "$VERSION_MAJ_MIN" | sed 's/\./-/g')"
TESTNET_SIZE=3

echo "Cleaning up before deploy"
cleanup

echo "Starting testnet '$TESTNET_NAME' with $TESTNET_SIZE nodes and image hoprd:$RELEASE"
start_testnet $TESTNET_NAME $TESTNET_SIZE "gcr.io/hoprassociation/hoprd:$RELEASE" "${RPC}"
