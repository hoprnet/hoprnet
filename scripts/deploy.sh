#!/bin/bash
set -e #u
shopt -s expand_aliases
#set -o xtrace

# Don't source this file twice
test -z "${DEPLOY_SOURCED:-}" && DEPLOY_SOURCED=1 || exit 0

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
# - FUNDING_PRIV_KEY: funding private key, raw
# - BS_PASSWORD: database password

if [ -z "$RPC" ]; then
  RPC=https://eth-goerli.gateway.pokt.network/v1/6021a2b6928ff9002e6c7f2f
fi

source scripts/dependencies.sh

# Get version from package.json
RELEASE=$(node -p -e "require('./packages/hoprd/package.json').version")

# Get RELEASE_NAME, from environment
get_environment

TESTNET_NAME="$RELEASE_NAME-$(echo "$VERSION_MAJ_MIN" | sed 's/\./-/g')"
TESTNET_SIZE=3

echo "Cleaning up before deploy"
cleanup

echo "Starting testnet '$TESTNET_NAME' with $TESTNET_SIZE nodes and image hoprd:$RELEASE"
start_testnet $TESTNET_NAME $TESTNET_SIZE "gcr.io/hoprassociation/hoprd:$RELEASE" 
