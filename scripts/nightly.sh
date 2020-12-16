#!/bin/bash
set -e #u

source scripts/testnet.sh
source scripts/cleanup.sh

# ----- Nightly integration / network test. --------

# Get version from package.json
RELEASE=$(node -p -e "require('./packages/hoprd/package.json').version")
IMG="gcr.io/hoprassociation/hoprd:$RELEASE"

source scripts/dependencies.sh
echo "Cleaning up devops before running nightly testnet"
cleanup
echo "Starting nightly testnet"
start_testnet nightly 2 $IMG
sleep 72000 # 20mins
gcloud_get_logs nightly-bootstrap $IMG > bootstrap-logs.txt 
gcloud_get_logs nightly-node-2 $IMG > node-2.txt 
cat bootstrap-logs.txt
cleanup

