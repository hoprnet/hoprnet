#!/bin/bash
set -e #u

source scripts/testnet.sh
source scripts/cleanup.sh

# ----- Nightly integration / network test. --------

if [ -z "$RPC" ]; then
  RPC=https://bsc-dataseed.binance.org/
fi

# Get version from package.json
RELEASE=$(node -p -e "require('./packages/hoprd/package.json').version")
IMG="gcr.io/hoprassociation/hoprd:$RELEASE"

source scripts/dependencies.sh
#echo "Cleaning up devops before running nightly testnet"
#cleanup
echo "Starting nightly testnet"
start_testnet nightly 2 $IMG
echo "Testnet up and running. Leaving it for 20 mins"
sleep 72000 # 20mins
echo "Testnet has run for 20m, time to kill it."
gcloud_get_logs nightly-bootstrap $IMG > bootstrap-logs.txt 
gcloud_get_logs nightly-node-2 $IMG > node-2.txt 
cat bootstrap-logs.txt
cleanup

