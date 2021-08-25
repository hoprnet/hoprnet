#!/bin/bash
set -e #u

source scripts/testnet.sh
source scripts/cleanup.sh

# ----- Internal integration / network test. --------

# Get version from package.json
#RELEASE=$(node -p -e "require('./packages/hoprd/package.json').version")
#IMG="gcr.io/hoprassociation/hoprd:$RELEASE"
IMG="gcr.io/hoprassociation/hoprd:latest"

echo "Cleaning up devops before running internal testnet"
cleanup
echo "Starting internal testnet (using goerli)"
start_testnet internal 1 $IMG
echo "Testnet up and running. Leaving it for 20 mins"
sleep 72000 # 20mins
echo "Testnet has run for 20m, time to kill it."
gcloud_get_logs internal-node-1 $IMG > node-1.txt
cat node-1.txt
cleanup
