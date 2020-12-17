#!/bin/bash
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
# - FUNDING_PRIV_KEY: funding private key, raw
# - BS_PASSWORD: database password

if [ -z "$RPC" ]; then
  RPC=https://bsc-dataseed.binance.org/
fi

source scripts/dependencies.sh

# Get version from package.json
RELEASE=$(node -p -e "require('./packages/hoprd/package.json').version")

# Get RELEASE_NAME, from environment
get_environment

TESTNET_NAME="$RELEASE_NAME-$(echo "$VERSION_MAJ_MIN" | sed 's/\./-/g')"
TESTNET_SIZE=2

echo "Cleaning up before deploy"
cleanup

echo "Starting testnet '$TESTNET_NAME' with $TESTNET_SIZE nodes"
start_testnet $TESTNET_NAME $TESTNET_SIZE "gcr.io/hoprassociation/hoprd:$RELEASE" 
