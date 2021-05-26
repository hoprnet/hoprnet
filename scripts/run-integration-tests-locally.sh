#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && (echo "This script should only be executed."; exit 1)

# set log id and use shared log function for readable logs
declare HOPR_LOG_ID="e2e-test-run-local"
source "$(dirname $(readlink -f $0))/lib/utils.sh"

declare hoprd="node packages/hoprd/lib/index.js --init --password='' --provider=ws://127.0.0.1:8545/ --testAnnounceLocalAddresses"
declare hardhat="yarn hardhat"

if [ -z "${CI:-}" ]; then
  DELAY=2
  MAX_WAITS=1000
else
  DELAY=10
  MAX_WAITS=10
fi

declare node1_dir="/tmp/hopr-node-1"
declare node2_dir="/tmp/hopr-node-2"
declare node3_dir="/tmp/hopr-node-3"
declare node1_log="${node1_dir}.log"
declare node2_log="${node2_dir}.log"
declare node3_log="${node3_dir}.log"
declare node1_id="${node1_dir}.id"
declare node2_id="${node2_dir}.id"
declare node3_id="${node3_dir}.id"

declare node1_pid node2_pid node3_pid

declare hardhat_rpc_log="/tmp/hopr-hardhat-rpc.log"

declare hardhat_pid

function check_port() {
  if [[ "$(lsof -i ":$1" | grep -c 'LISTEN')" -ge 1 ]]; then
    log "Port is not free $1"
    log "Process: $(lsof -i ":$1" | grep 'LISTEN')"
    exit 1
  fi
}

# $1 = Port to wait for
# $2 = optional file to tail for debug info
function wait_for_port() {
  i=0
  until [ "$(lsof -i ":$1" | grep -c "LISTEN")" -gt 0  ]; do
    log "Waiting ($DELAY) for port $1"
    if [ -n "${2:-}" ]; then
      log "Last 5 logs:"
      tail -n 5 $2
    fi
    sleep $DELAY
    ((i=i+1))
    if [ $i -gt $MAX_WAITS ]; then
      exit 1
    fi
  done
}

# Funds a HOPR node with ETH + HOPR tokens
# @param $1 - node API
function fund_node {
  local ETH

  ETH="$(curl --silent $1/api/v1/address/hopr)"

  if [ -z "$ETH" ]; then
    log "Can't fund node - couldn't load ETH address"
    exit 1
  fi

  log "Funding 1 ETH and 1 HOPR to $ETH"
  $hardhat faucet --config packages/ethereum/hardhat.config.ts --address "$ETH" --network localhost --ishopraddress true
}

function cleanup {
  local EXIT_CODE=$?

  # Cleaning up everything
  if [ "$EXIT_CODE" != "0" ]; then
    log "Exited with fail, code $EXIT_CODE"
    log "Printing last 100 lines from logs"
    tail -n 100 "${node1_log}" "${node2_log}" "${node3_log}"
    log "DONE Printing last 100 lines from logs"
  fi

  log "Wiping databases"
  rm -rf "${node1_dir}" "${node2_dir}" "${node3_dir}"

  log "Cleaning up processes"
  test -n "${node1_pid:-}" && kill "${node1_pid}"
  test -n "${node2_pid:-}" && kill "${node2_pid}"
  test -n "${node3_pid:-}" && kill "${node3_pid}"
  test -n "${hardhat_pid:-}" && kill "${hardhat_pid}"

  exit $EXIT_CODE
}

trap cleanup EXIT

# --- Log test info {{{
log "Test files and directories"
log "\thardhat"
log "\t\tlog: ${hardhat_rpc_log}"
log "\tnode1"
log "\t\tdata dir: ${node1_dir} (will be removed)"
log "\t\tlog: ${node1_log}"
log "\t\tid: ${node1_id}"
log "\tnode2"
log "\t\tdata dir: ${node2_dir} (will be removed)"
log "\t\tlog: ${node2_log}"
log "\t\tid: ${node2_id}"
log "\tnode3"
log "\t\tdata dir: ${node3_dir} (will be removed)"
log "\t\tlog: ${node3_log}"
log "\t\tid: ${node3_id}"
# }}}

# --- Check all resources we need are free {{{
check_port 8545
check_port 3301
check_port 3302
check_port 3303
check_port 9091
check_port 9092
check_port 9093
# }}}

# --- Running Mock Blockchain --- {{{
log "Running hardhat local node"
$hardhat node --config packages/ethereum/hardhat.config.ts > "${hardhat_rpc_log}" 2>&1 &
hardhat_pid="$!"
log "Hardhat node started (127.0.0.1:8545) with PID $hardhat_pid"
wait_for_port 8545 "${hardhat_rpc_log}"
# }}}

#  --- Run nodes --- {{{
log "Run node 1"
declare api1="127.0.0.1:3301"
DEBUG="hopr*" $hoprd --identity="${node1_id}" --host=0.0.0.0:9091 --data="${node1_dir}" --rest --restPort 3301 --announce > \
  "${node1_log}" 2>&1 &
node1_pid="$!"
wait_for_port 3301 "${node1_log}"

log "Run node 2"
declare api2="127.0.0.1:3302"
DEBUG="hopr*" $hoprd --identity="${node2_id}" --host=0.0.0.0:9092 --data="${node2_dir}" --rest --restPort 3302 --announce > \
  "${node2_log}" 2>&1 &
node2_pid="$!"
wait_for_port 3302 "${node2_log}"

log "Run node 3"
declare api3="127.0.0.1:3303"
DEBUG="hopr*" $hoprd --identity="${node3_id}" --host=0.0.0.0:9093 --data="${node3_dir}" --rest --restPort 3303 --announce > \
  "${node3_log}" 2>&1 &
node3_pid="$!"
wait_for_port 3303 "${node3_log}"
# }}}

# --- Fund Nodes --- {{{
fund_node "$api1"
fund_node "$api2"
fund_node "$api3"
# }}}

# --- Wait for everything to be ready, then run tests --- {{{
wait_for_port 9091
wait_for_port 9092
wait_for_port 9093

bash "$(dirname $(readlink -f $0))/../test/integration-test.sh" \
  "${api1}" "${api2}" "${api3}"
# }}}
