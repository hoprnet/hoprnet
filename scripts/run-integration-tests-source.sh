#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="e2e-source-test"
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0"
  msg
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
declare wait_delay=2
declare wait_max_wait=1000

if [ "${CI:-}" = "true" ] && [ -z "${ACT:-}" ]; then
  wait_delay=10
  wait_max_wait=10
fi

declare node1_dir="/tmp/hopr-source-node-1"
declare node2_dir="/tmp/hopr-source-node-2"
declare node3_dir="/tmp/hopr-source-node-3"
declare node4_dir="/tmp/hopr-source-node-4"

declare node1_log="${node1_dir}.log"
declare node2_log="${node2_dir}.log"
declare node3_log="${node3_dir}.log"
declare node4_log="${node4_dir}.log"

declare node1_id="${node1_dir}.id"
declare node2_id="${node2_dir}.id"
declare node3_id="${node3_dir}.id"
declare node4_id="${node4_dir}.id"

declare hardhat_rpc_log="/tmp/hopr-source-hardhat-rpc.log"

function cleanup {
  trap - SIGINT SIGTERM ERR EXIT

  local EXIT_CODE=$?

  # Cleaning up everything
  if [ "$EXIT_CODE" != "0" ]; then
    log "Exited with fail, code $EXIT_CODE"
    for log_file in "${node1_log}" "${node2_log}" "${node3_log}" "${node4_log}"; do
      if [ -n "${log_file}" ] && [ -f "${log_file}" ]; then
        log "Printing last 100 lines from logs"
        tail -n 100 "${node1_log}" "${node2_log}" "${node3_log}" \
          "${node4_log}" | sed "s/^/\t/" || :
        log "Printing last 100 lines from logs DONE"
      fi
    done
  fi

  log "Wiping databases"
  rm -rf "${node1_dir}" "${node2_dir}" "${node3_dir}"

  log "Cleaning up processes"
  for port in 8545 3301 3302 3303 3304 9091 9092 9093 9094; do
    if lsof -i ":${port}" -s TCP:LISTEN; then
      lsof -i ":${port}" -s TCP:LISTEN -t | xargs kill
    fi
  done

  exit $EXIT_CODE
}

trap cleanup SIGINT SIGTERM ERR EXIT

# $1 = rest port
# $2 = node port
# $3 = admin port
# $4 = node data directory
# $5 = node log file
# $6 = node id file
# $7 = OPTIONAL: additional args to hoprd
function setup_node() {
  local rest_port=${1}
  local node_port=${2}
  local admin_port=${3}
  local dir=${4}
  local log=${5}
  local id=${6}
  local additional_args=${7:-""}

  log "Run node ${id} on rest port ${rest_port}"

  if [ -n "${additional_args}" ]; then
    log "Additional args: \"${additional_args}\""
  fi

  DEBUG="hopr*" node packages/hoprd/lib/index.js \
    --init --provider=http://127.0.0.1:8545/ \
    --testAnnounceLocalAddresses --identity="${id}" \
    --host="127.0.0.1:${node_port}" --testPreferLocalAddresses \
    --data="${dir}" --rest --restPort "${rest_port}" --announce \
    --api-token "e2e-api-token" \
    --admin --adminHost "127.0.0.1" --adminPort ${admin_port} \
    --password="e2e-test" --testUseWeakCrypto \
    ${additional_args} \
    > "${log}" 2>&1 &

  wait_for_http_port "${rest_port}" "${log}" "${wait_delay}" "${wait_max_wait}"
}

# $1 = rest port
# $2 = node log file
function fund_node() {
  local rest_port=${1}
  local log=${2}
  local api="127.0.0.1:${rest_port}"

  local eth_address
  eth_address="$(curl --silent "${api}/api/v1/address/hopr")"

  if [ -z "${eth_address}" ]; then
    log "Can't fund node - couldn't load ETH address"
    exit 1
  fi

  log "Funding 1 ETH and 1 HOPR to ${eth_address}"
  yarn hardhat faucet --config packages/ethereum/hardhat.config.ts \
    --address "${eth_address}" --network localhost --ishopraddress true
}

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
log "\tnode4"
log "\t\tdata dir: ${node4_dir} (will be removed)"
log "\t\tlog: ${node4_log}"
log "\t\tid: ${node4_id}"
# }}}

# --- Check all resources we need are free {{{
ensure_port_is_free 8545
ensure_port_is_free 3301
ensure_port_is_free 3302
ensure_port_is_free 3303
ensure_port_is_free 3304
ensure_port_is_free 9091
ensure_port_is_free 9092
ensure_port_is_free 9093
ensure_port_is_free 9094
ensure_port_is_free 9501
ensure_port_is_free 9502
ensure_port_is_free 9503
ensure_port_is_free 9504
# }}}

# --- Running Mock Blockchain --- {{{
log "Running hardhat local node"
DEVELOPMENT=true yarn hardhat node --config packages/ethereum/hardhat.config.ts \
  --network hardhat --as-network localhost --show-stack-traces > \
  "${hardhat_rpc_log}" 2>&1 &

log "Hardhat node started (127.0.0.1:8545)"
wait_for_http_port 8545 "${hardhat_rpc_log}" "${wait_delay}" "${wait_max_wait}"
# }}}

#  --- Run nodes --- {{{
setup_node 3301 9091 9501 "${node1_dir}" "${node1_log}" "${node1_id}"
setup_node 3302 9092 9502 "${node2_dir}" "${node2_log}" "${node2_id}"
setup_node 3303 9093 9503 "${node3_dir}" "${node3_log}" "${node3_id}"
setup_node 3304 9094 9504 "${node4_dir}" "${node4_log}" "${node4_id}" "--run \"info;balance\""
# }}}

#  --- Fund nodes --- {{{
fund_node 3301 "${node1_log}"
fund_node 3302 "${node2_log}"
fund_node 3303 "${node3_log}"
fund_node 3304 "${node4_log}"
# }}}

#  --- Wait for ports to be bound --- {{{
wait_for_port 9091 "${node1_log}"
wait_for_port 9092 "${node2_log}"
wait_for_port 9093 "${node3_log}"
wait_for_port 9094 "${node4_log}"
# }}}

# --- Run security tests --- {{{
${mydir}/../test/security-test.sh \
  127.0.0.1 3301 9501
#}}}

# --- Run protocol test --- {{{
${mydir}/../test/integration-test.sh \
  "localhost:3301" "localhost:3302" "localhost:3303"
# }}}

# -- Verify node4 has executed the commands {{{
# REMOVED AS THIS IS BROKEN
#echo "- Verifying node4 log output"
#grep -q "^HOPR Balance:" "${node4_log}"
#grep -q "^Running on: localhost" "${node4_log}"
#}}}
