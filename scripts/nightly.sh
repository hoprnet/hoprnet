#!/bin/bash
set -e #u

source scripts/testnet.sh
source scripts/cleanup.sh

# ----- Nightly integration / network test. --------

# Get version from package.json
RELEASE=$(node -p -e "require('./packages/hoprd/package.json').version")

source scripts/dependencies.sh
start_testnet nightly 5 "gcr.io/hoprassociation/hoprd:$RELEASE" 

sleep 72000 # 20mins

# TODO download logs

cleanup

