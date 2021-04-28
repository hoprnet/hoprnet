#!/bin/bash
set -e

source 'test/e2e/0_configuration.sh'

function main {
  # Running RPC
  rpc_network
  # Running server
  start_node

  echo "🤖 (BS) Requesting address"
  HOPR_ADDRESS=$(curl -s http://127.0.0.1:3001/api/v1/address/hopr)
  echo "⛓  HOPR Address: $HOPR_ADDRESS"

  fund_node $HOPR_ADDRESS
  BALANCE=$(curl -s --data balance http://127.0.0.1:3001/api/v1/command)

  ETH_BALANCE=$(echo -e $BALANCE | grep -c "1\ xDAI" || true)
  HOPR_BALANCE=$(echo -e $BALANCE | grep -c "1\ HOPR" || true)

  if [[ $ETH_BALANCE == 1 && $HOPR_BALANCE == 1 ]]; then
    echo "✅ Node holds balance after being funded."
    exit 0
  else
    echo "⛔️ Node has an invalid balance:"
    echo -e "$BALANCE"
    exit 1 
  fi
}
