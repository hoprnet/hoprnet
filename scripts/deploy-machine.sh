#!/usr/bin/env bash

set -e #u
shopt -s expand_aliases
#set -o xtrace

source scripts/testnet.sh
source scripts/cleanup.sh

# ---- On Deployment -----
#
# This is run on pushes to master, or release/**
#
# ENV Variables:
# - GITHUB_REF: ie. `/refs/heads/mybranch`
# - FUNDING_PRIV_KEY: funding private key, raw
# - BS_PASSWORD: database password

# Get version from package.json if not already set
if [ -z "${RELEASE:-}" ]; then
  RELEASE=$(node -p -e "require('./packages/hoprd/package.json').version")
fi

HOPRD_API_TOKEN='D3f4ul+!_2021'
BS_PASSWORD='d3f4ul+!_2021'

echo "Starting single preview pr node for hoprd:$RELEASE"
start_testnet "pr-preview-$RELEASE" 1 "gcr.io/hoprassociation/hoprd:$RELEASE" "master-goerli" true
