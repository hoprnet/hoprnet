#!/bin/bash
set -e
shopt -s expand_aliases

alias hardhat="node packages/ethereum/node_modules/.bin/hardhat"
alias hoprd="node packages/hoprd/lib/index.js --admin --init --rest --provider=ws://127.0.0.1:8545/ --password=''"

function finish {
  # Cleaning up everything
  echo "🧽 Cleaning up processes"
  if [[ -n "$PROVIDER_PID" ]]; then kill $PROVIDER_PID; fi
}
trap finish EXIT

DATAFILE=`basename "$0"`

# Running RPC
echo "⛑ Running hardhat local node"
hardhat node --config packages/ethereum/hardhat.config.ts > /tmp/$DATAFILE-rpc.txt 2>&1 &
PROVIDER_PID="$!"

echo "⏰ Waiting (20) seconds for hardhat node to start"
sleep 20
echo "🤖 (BS) Running bootstrap node"

DEBUG=hopr* hoprd --data=/tmp/$DATAFILE-bootstrap --runAsBootstrap > /tmp/$DATAFILE-bs.txt 2>&1 &
BOOTSTRAP_PID="$!"
BOOTSTRAP_ADDRESS=$(curl localhost:3001/api/v1/address/hopr)
 
echo "⛓ Address: $BOOTSTRAP_ADDRESS"

sleep 100