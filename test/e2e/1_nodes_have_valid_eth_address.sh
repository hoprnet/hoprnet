#!/bin/bash
set -e

source 'test/e2e/0_configuration.sh'

# Running RPC
rpc_network
# Running bootstrap server
bootstrap_node

echo "🤖 (BS) Requesting bootstrap address"
BOOTSTRAP_ADDRESS=$(curl -s http://127.0.0.1:3001/api/v1/address/eth)
echo "⛓  ETH Address: $BOOTSTRAP_ADDRESS"
IS_VALID_ETH_ADDRESS=$(node -e \
  "const ethers = require('ethers'); console.log(ethers.utils.isAddress('$BOOTSTRAP_ADDRESS'))")

if [ $IS_VALID_ETH_ADDRESS == 'true' ]; then
  echo "✅ Node outputs a valid address: $IS_VALID_ETH_ADDRESS"
  exit 0
else
  echo "⛔️ Node outputs an invalid address: $BOOTSTRAP_ADDRESS"
  exit 1 
fi



