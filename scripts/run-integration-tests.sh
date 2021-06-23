#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

# Runs a testnet, and tests against it.
# This relies on using GCloud as an environment for testing

source scripts/environments.sh
source scripts/testnet.sh
source scripts/cleanup.sh

# Get version from package.json
#'hoprd:1.71.0-next.149'
declare RELEASE
RELEASE=$(HOPR_PACKAGE=hoprd ./scripts/get-package-version.sh)

TESTNET_NAME="integration-test$(echo "$VERSION_MAJ_MIN" | sed 's/\./-/g')"
TESTNET_SIZE=5

#echo "Cleaning up before deploy"
#cleanup

#echo "Starting a fake chain provider RPC node"
#start_chain_provider
#exit 1

#echo "Starting testnet '$TESTNET_NAME' with $TESTNET_SIZE nodes and image hoprd:$RELEASE"
#start_testnet $TESTNET_NAME $TESTNET_SIZE "gcr.io/hoprassociation/hoprd:$RELEASE" 

# TODO FUND ADDRESS

echo "Running integration test against testnet: $TESTNET_NAME"
NODE1="$(vm_name 'node-1' $TESTNET_NAME)"
NODE2="$(vm_name 'node-2' $TESTNET_NAME)"
NODE3="$(vm_name 'node-3' $TESTNET_NAME)"
NODE4="$(vm_name 'node-4' $TESTNET_NAME)"
NODE5="$(vm_name 'node-5' $TESTNET_NAME)"
API1="$(gcloud_get_ip $NODE1)"
API2="$(gcloud_get_ip $NODE2)"
API3="$(gcloud_get_ip $NODE3)"
API4="$(gcloud_get_ip $NODE4)"
API5="$(gcloud_get_ip $NODE5)"
source $(realpath test/integration-test.sh)

#echo "Cleaning up after deploy"
#cleanup
