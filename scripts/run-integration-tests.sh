#!/bin/bash

# Runs a testnet, and tests against it.
# This relies on using GCloud as an environment for testing

source scripts/environments.sh
source scripts/testnet.sh
source scripts/cleanup.sh

# Get version from package.json
RELEASE=$(node -p -e "require('./packages/hoprd/package.json').version")

TESTNET_NAME="integration-test$(echo "$VERSION_MAJ_MIN" | sed 's/\./-/g')"
TESTNET_SIZE=3

echo "Cleaning up before deploy"
cleanup

echo "Starting a fake chain provider RPC node"
start_chain_provider

exit 1

echo "Starting testnet '$TESTNET_NAME' with $TESTNET_SIZE nodes and image hoprd:$RELEASE"
start_testnet $TESTNET_NAME $TESTNET_SIZE "gcr.io/hoprassociation/hoprd:$RELEASE" 

# TODO FUND ADDRESS

echo "Running integration test against testnet: $TESTNET_NAME"
source ./test/integration_test.sh
