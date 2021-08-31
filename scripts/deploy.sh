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
# - FUNDING_PRIV_KEY: funding private key, raw
# - BS_PASSWORD: database password

# Get version from package.json if not already set
if [ -z "${RELEASE:-}" ]; then
  RELEASE=$(node -p -e "require('./packages/hoprd/package.json').version")
fi

# get environment_id from package.json if not set externally
if [ -z "${ENVIRONMENT_ID:-}" ]; then
  ENVIRONMENT_ID=$(node -p -e "require('./packages/hoprd/package.json').hopr.environment_id")
fi


# Get RELEASE_NAME, from environment
get_environment

# TESTNET_NAME="$RELEASE_NAME-$(echo "$VERSION_MAJ_MIN" | sed 's/\./-/g')"
TESTNET_NAME="local-debug"
TESTNET_SIZE=3

echo "Cleaning up before deploy"
cleanup

echo "Starting testnet '$TESTNET_NAME' with $TESTNET_SIZE nodes and image hoprd:$RELEASE, environment id: $ENVIRONMENT_ID"
start_testnet $TESTNET_NAME $TESTNET_SIZE "gcr.io/hoprassociation/hoprd:$RELEASE" $ENVIRONMENT_ID
