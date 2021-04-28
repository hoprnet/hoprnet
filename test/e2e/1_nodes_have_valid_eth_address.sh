#!/bin/bash
set -e

source 'test/e2e/0_configuration.sh'

function main {
  # Running RPC
  rpc_network
  # Running server
  start_node

  echo "🤖 (BS) Requesting address"
  ETH_ADDRESS=$(curl -s http://127.0.0.1:3001/api/v1/address/eth)
  echo "⛓  ETH Address: $ETH_ADDRESS"
  IS_VALID_ETH_ADDRESS=$(node -e \
    "const ethers = require('ethers'); console.log(ethers.utils.isAddress('$ETH_ADDRESS'))")

  if [ $IS_VALID_ETH_ADDRESS == 'true' ]; then
    echo "✅ Node outputs a valid address: $IS_VALID_ETH_ADDRESS"
    exit 0
  else
    echo "⛔️ Node outputs an invalid address: $ETH_ADDRESS"
    exit 1 
  fi
}




