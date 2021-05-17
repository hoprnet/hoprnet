#!/bin/bash

alias hoprd="node packages/hoprd/lib/index.js --init --password=' --provider=ws://127.0.0.1:8545/"

# Funds a HOPR node with ETH + HOPR tokens
# @param $1 - HOPR address for node
# @dev Sleeps for 20 seconds after funding
function fund_node {
  echo "💰 Funding 1 ETH and 1 HOPR to $1"
  hardhat faucet --config packages/ethereum/hardhat.config.ts --address "$1" --network localhost --ishopraddress true
  echo "💰 $1 funded with 1 ETH and 1 HOPR"
  echo "⏰ Waiting (20) seconds for node to catch-up w/balance"
  sleep 20
}

function cleanup {
  # Cleaning up everything
  echo "Printing last 100 lines from logs"
  tail -n 20 "/tmp/NODE1-log.txt" "/tmp/NODE2-log.txt" "/tmp/NODE3-log.txt" 
  echo "Wiping databases"
  rm -rf /tmp/NODE1
  rm -rf /tmp/NODE2
  rm -rf /tmp/NODE3
  echo "Cleaning up processes"
  #test -n "$PROVIDER_PID" && kill "$PROVIDER_PID"
  test -n "$NODE1_PID" && kill "$NODE1_PID"
  test -n "$NODE2_PID" && kill "$NODE2_PID"
  test -n "$NODE3_PID" && kill "$NODE3_PID"
}
trap cleanup EXIT

# Running RPC
echo "⛑ Running hardhat local node"
hardhat node --config packages/ethereum/hardhat.config.ts > "/tmp/$DATAFILE-rpc.txt" 2>&1 &
PROVIDER_PID="$!"
echo "⛑ Hardhat node started (127.0.0.1:8545)"
echo "⏰ Waiting (20) seconds for hardhat node to deploy contracts"
sleep 20

echo "Run node 1"
API1="127.0.0.1:33001"
DEBUG="hopr*" hoprd --identity="/tmp/NODE1-id" --host=0.0.0.0:9091 --data="/tmp/NODE1" --rest --restPort 33001 > "/tmp/NODE1-log.txt" 2>&1 &
NODE1_PID="$!"

echo "Run node 2"
API2="127.0.0.1:33002"
DEBUG="hopr*" hoprd --identity="/tmp/NODE2-id" --host=0.0.0.0:9092 --data="/tmp/NODE2" --rest --restPort 33002 > "/tmp/NODE2-log.txt" 2>&1 &
NODE2_PID="$!"

echo "Run node 3"
# Run node 3
API3="127.0.0.1:33003"
DEBUG="hopr*" hoprd --identity="/tmp/NODE2-id" --host=0.0.0.0:9093 --data="/tmp/NODE3" --rest --restPort 33003 > "/tmp/NODE3-log.txt" 2>&1 &
NODE3_PID="$!"

sleep 10

source $(realpath test/integration-test.sh)

