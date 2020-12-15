#!/bin/bash
set -e #u
shopt -s expand_aliases
#set -o xtrace

source scripts/cd/environments.sh
source scripts/cd/testnet.sh


hoprd_image() {
  # For example ...hoprd:1.0.1-next-1234
  echo "gcr.io/hoprassociation/hoprd:$RELEASE"
}

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
  RPC=https://rpc-mainnet.matic.network
fi

# -- Setup Dependencies --
ethers --version || npm install -g @ethersproject/cli

# Get version from package.json
RELEASE=$(node -p -e "require('./packages/hoprd/package.json').version")

# Get RELEASE_NAME, from environment
get_environment



start_testnet $RELEASE_NAME 1 $(hoprd_image)
