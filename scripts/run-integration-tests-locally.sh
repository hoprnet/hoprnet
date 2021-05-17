#!/bin/bash
hoprd="node packages/hoprd/lib/index.js --init --password='' --provider=ws://127.0.0.1:8545/"
hardhat="yarn hardhat"

if [ -z "$CI" ]; then 
  DELAY=2
else
  DELAY=20
fi

# Funds a HOPR node with ETH + HOPR tokens
# @param $1 - node API
function fund_node {
  ETH="$(curl $1/api/v1/address/hopr)"
  if [ -z "$ETH" ]; then
    echo "Can't fund node - couldn't load ETH address"
    exit 1
  fi
  echo "- Funding 1 ETH and 1 HOPR to $ETH"
  $hardhat faucet --config packages/ethereum/hardhat.config.ts --address "$ETH" --network localhost --ishopraddress true
  echo "- Waiting ($DELAY) seconds for node to catch-up w/balance"
  sleep $DELAY
}

function cleanup {
  # Cleaning up everything
  echo "Printing last 100 lines from logs"
  tail -n 100 "/tmp/NODE1-log.txt" "/tmp/NODE2-log.txt" "/tmp/NODE3-log.txt" 
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
echo "- Running hardhat local node"
$hardhat node --config packages/ethereum/hardhat.config.ts > "/tmp/$DATAFILE-rpc.txt" 2>&1 &
PROVIDER_PID="$!"
echo "- Hardhat node started (127.0.0.1:8545)"
echo "- Waiting ($DELAY) seconds for hardhat node to deploy contracts"
sleep $DELAY

echo "- Run node 1"
API1="127.0.0.1:3301"
DEBUG="hopr*" $hoprd --identity="/tmp/NODE1-id" --host=0.0.0.0:9091 --data="/tmp/NODE1" --rest --restPort 3301 --announce > "/tmp/NODE1-log.txt" 2>&1 &
NODE1_PID="$!"
sleep $DELAY

echo "- Run node 2"
API2="127.0.0.1:3302"
DEBUG="hopr*" $hoprd --identity="/tmp/NODE2-id" --host=0.0.0.0:9092 --data="/tmp/NODE2" --rest --restPort 3302 --announce > "/tmp/NODE2-log.txt" 2>&1 &
NODE2_PID="$!"
sleep $DELAY

echo "- Run node 3"
# Run node 3
API3="127.0.0.1:3303"
DEBUG="hopr*" $hoprd --identity="/tmp/NODE3-id" --host=0.0.0.0:9093 --data="/tmp/NODE3" --rest --restPort 3303 --announce > "/tmp/NODE3-log.txt" 2>&1 &
NODE3_PID="$!"
sleep $DELAY


fund_node $API1
fund_node $API2
fund_node $API3

source $(realpath test/integration-test.sh)

