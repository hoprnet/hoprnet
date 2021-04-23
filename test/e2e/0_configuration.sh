#!/bin/bash
set -e
shopt -s expand_aliases

alias hardhat="node packages/ethereum/node_modules/.bin/hardhat"
alias hoprd="node packages/hoprd/lib/index.js --admin --init --rest --provider=ws://127.0.0.1:8545/ --password=''"

function cleanup {
  # Cleaning up everything
  echo "🧽 Cleaning up processes"
  if [[ -n "$PROVIDER_PID" ]]; then kill $PROVIDER_PID; fi
  if [[ -n "$BOOTSTRAP_PID" ]]; then kill $BOOTSTRAP_PID; fi
}

# Starts a hardhat node w/an exposed RPC endpoint @ 127.0.0.1
# @param none
# @dev Sleeps for 20 seconds upon start.
function rpc_network {
  echo "⛑  Running hardhat local node"
  hardhat node --config packages/ethereum/hardhat.config.ts > /tmp/$DATAFILE-rpc.txt 2>&1 &
  PROVIDER_PID="$!"
  echo "⛑  Hardhat node started (127.0.0.1:8545)"
  echo "⏰ Waiting (20) seconds for hardhat node to deploy contracts"
  sleep 20
}

# Starts a bootstrap server, including an admin, and rest interface
# @param nonde
# @dev Sleeps for 10 seconds upon start
function bootstrap_node {
  echo "🤖 (BS) Running bootstrap node"
  DEBUG=hopr* hoprd --data=/tmp/$DATAFILE-bootstrap --runAsBootstrap > /tmp/$DATAFILE-bs.txt 2>&1 &
  BOOTSTRAP_PID="$!"
  echo "🤖 (BS) Bootstrap started (127.0.0.1:9091,3000,3001)"
  echo "⏰ Waiting (10) seconds for bootstrap node to start"
  sleep 10
}

DATAFILE=`basename "$0"`
trap cleanup EXIT