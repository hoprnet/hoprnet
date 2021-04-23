#!/bin/bash
set -e

source 'test/e2e/0_configuration.sh'

# Running RPC
rpc_network
# Running bootstrap server
bootstrap_node

echo "🤖 (BS) Requesting bootstrap address"
BOOTSTRAP_HOPR_ADDRESS=$(curl -s http://127.0.0.1:3001/api/v1/address/hopr)
echo "⛓  HOPR Address: $BOOTSTRAP_HOPR_ADDRESS"

fund_node $BOOTSTRAP_HOPR_ADDRESS
BOOTSTRAP_BALANCE=$(curl -s --data balance http://127.0.0.1:3001/api/v1/command)

ETH_BALANCE=$(echo -e $BOOTSTRAP_BALANCE | grep -c "1\ xDAI" || true)
HOPR_BALANCE=$(echo -e $BOOTSTRAP_BALANCE | grep -c "1\ HOPR" || true)

if [[ $ETH_BALANCE == 1 && $HOPR_BALANCE == 1 ]]; then
  echo "✅ Node holds balance after being funded."
  exit 0
else
  echo "⛔️ Node has an invalid balance:"
  echo -e "$BOOTSTRAP_BALANCE"
  exit 1 
fi