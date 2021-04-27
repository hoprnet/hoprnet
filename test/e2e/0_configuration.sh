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
# @param none
# @dev Sleeps for 10 seconds upon start
function bootstrap_node {
  echo "🤖 (BS) Running bootstrap node"
  DEBUG=hopr* hoprd --data=/tmp/$DATAFILE-bootstrap --runAsBootstrap > /tmp/$DATAFILE-bs.txt 2>&1 &
  BOOTSTRAP_PID="$!"
  echo "🤖 (BS) Bootstrap started (127.0.0.1:9091,3000,3001)"
  echo "⏰ Waiting (10) seconds for bootstrap node to start"
  sleep 10
}

# Funds a HOPR node with ETH + HOPR tokens
# @param $1 - HOPR address for node
# @dev Sleeps for 20 seconds after funding
function fund_node {
  echo "💰 Funding 1 ETH and 1 HOPR to $1"
  hardhat faucet --config packages/ethereum/hardhat.config.ts --address $1 --network localhost --ishopraddress true
  echo "💰 $1 funded with 1 ETH and 1 HOPR"
  echo "⏰ Waiting (20) seconds for node to catch-up w/balance"
  sleep 20
}

DATAFILE=`basename "$0"`
trap cleanup EXIT