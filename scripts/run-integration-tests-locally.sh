#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

declare hoprd="node packages/hoprd/lib/index.js --init --password='' --provider=ws://127.0.0.1:8545/ --testAnnounceLocalAddresses"
declare hardhat="yarn hardhat"

if [ -z "${CI:-}" ]; then
  DELAY=2
else
  DELAY=20
fi

declare node1_dir node2_dir node3_dir

node1_dir="/tmp/hopr-node-1"
node2_dir="/tmp/hopr-node-2"
node3_dir="/tmp/hopr-node-3"

declare node1_log="${node1_dir}.log"
declare node2_log="${node2_dir}.log"
declare node3_log="${node3_dir}.log"

declare node1_id="${node1_dir}.id"
declare node2_id="${node2_dir}.id"
declare node3_id="${node3_dir}.id"

declare node1_pid node2_pid node3_pid

declare hardhat_rpc_log
hardhat_rpc_log="/tmp/hopr-hardhat-rpc-XXXXXX.log"

function check_port() {
  if [[ "$(lsof -i ":$1" | grep -c 'LISTEN')" -ge 1 ]]; then
    echo "Port is not free $1"
    echo "Process: $(lsof -i ":$1" | grep 'LISTEN')"
    exit 1
  fi
}

# Funds a HOPR node with ETH + HOPR tokens
# @param $1 - node API
function fund_node {
  local ETH
  ETH="$(curl --silent $1/api/v1/address/hopr)"

  if [ -z "$ETH" ]; then
    echo "- Can't fund node - couldn't load ETH address"
    exit 1
  fi

  echo "- Funding 1 ETH and 1 HOPR to $ETH"
  $hardhat faucet --config packages/ethereum/hardhat.config.ts --address "$ETH" --network localhost --ishopraddress true

  echo "- Waiting ($DELAY) seconds for node to catch-up w/balance"
  sleep $DELAY
}

function cleanup {
  local EXIT_CODE=$?

  # Cleaning up everything
  if [ "$EXIT_CODE" != "0" ]; then
    echo "- Exited with fail, code $EXIT_CODE"
    echo "- Printing last 100 lines from logs"
    tail -n 100 "${node1_log}" "${node2_log}" "${node3_log}"
    echo "- DONE Printing last 100 lines from logs"
  fi

  echo -e "\n- Wiping databases"
  rm -rf "${node1_dir}" "${node2_dir}" "${node3_dir}"

  echo "- Cleaning up processes"
  test -n "${node1_pid:-}" && kill "${node1_pid}"
  test -n "${node2_pid:-}" && kill "${node2_pid}"
  test -n "${node3_pid:-}" && kill "${node3_pid}"
  test -n "${HARDHAT_PID:-}" && kill "${HARDHAT_PID}"

  exit $EXIT_CODE
}

trap cleanup EXIT

echo "- Test files and directories"
echo -e "\thardhat"
echo -e "\t\tlog: ${hardhat_rpc_log}"
echo -e "\tnode1"
echo -e "\t\tdata dir: ${node1_dir} (will be removed)"
echo -e "\t\tlog: ${node1_log}"
echo -e "\t\tid: ${node1_id}"
echo -e "\tnode2"
echo -e "\t\tdata dir: ${node2_dir} (will be removed)"
echo -e "\t\tlog: ${node2_log}"
echo -e "\t\tid: ${node2_id}"
echo -e "\tnode3"
echo -e "\t\tdata dir: ${node3_dir} (will be removed)"
echo -e "\t\tlog: ${node3_log}"
echo -e "\t\tid: ${node3_id}"

# Running RPC
echo "- Running hardhat local node"
declare HARDHAT_PID
check_port 8545
$hardhat node --config packages/ethereum/hardhat.config.ts --network hardhat > "${hardhat_rpc_log}" 2>&1 &
HARDHAT_PID="$!"

echo "- Hardhat node started (127.0.0.1:8545)"
echo "- Waiting ($DELAY) seconds for hardhat node to deploy contracts"
sleep $DELAY

echo "- Run node 1"
declare API1="127.0.0.1:3301"
DEBUG="hopr*" $hoprd --identity="${node1_id}" --host=0.0.0.0:9091 --data="${node1_dir}" --rest --restPort 3301 --announce > \
  "${node1_log}" 2>&1 &
node1_pid="$!"
sleep $DELAY

echo "- Run node 2"
declare API2="127.0.0.1:3302"
DEBUG="hopr*" $hoprd --identity="${node2_id}" --host=0.0.0.0:9092 --data="${node2_dir}" --rest --restPort 3302 --announce > \
  "${node2_log}" 2>&1 &
node2_pid="$!"
sleep $DELAY

echo "- Run node 3"
declare API3="127.0.0.1:3303"
DEBUG="hopr*" $hoprd --identity="${node3_id}" --host=0.0.0.0:9093 --data="${node3_dir}" --rest --restPort 3303 --announce > \
  "${node3_log}" 2>&1 &
node3_pid="$!"
sleep $DELAY

fund_node "$API1"
fund_node "$API2"
fund_node "$API3"

sleep $DELAY
sleep $DELAY

source $(realpath test/integration-test.sh)
