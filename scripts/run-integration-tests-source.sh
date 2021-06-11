#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1;
}

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

usage() {
  echo >&2
  echo "Usage: $0" >&2
  echo
  echo >&2
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
declare mydir

mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

# set log id and use shared log function for readable logs
declare HOPR_LOG_ID="e2e-source-test"
source "${mydir}/utils.sh"

declare wait_delay=2
declare wait_max_wait=1000

if [ -n "${CI:-}" ]; then
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
  local EXIT_CODE=$?

  # Cleaning up everything
  if [ "$EXIT_CODE" != "0" ]; then
    echo "- Exited with fail, code $EXIT_CODE"
<<<<<<< HEAD
    for log_file in "${node1_log}" "${node2_log}" "${node3_log}"; do
      if [ -n "${log_file}" ] && [ -f "${log_file}" ]; then
        echo "- Printing last 100 lines from logs"
        tail -n 100 "${node1_log}" "${node2_log}" "${node3_log}"
        echo "- Printing last 100 lines from logs DONE"
      fi
    done
=======
    # echo "- Printing last 100 lines from logs"
    # tail -n 100 "${node1_log}" "${node2_log}" "${node3_log}" || :
    # echo "- Printing last 100 lines from logs DONE"
>>>>>>> 47d098493 (refactored tests)
  fi

  echo -e "\n- Wiping databases"
  rm -rf "${node1_dir}" "${node2_dir}" "${node3_dir}"

  echo "- Cleaning up processes"
<<<<<<< HEAD
  for port in 8545 3301 3302 3303 9091 9092 9093; do
    if lsof -i ":${port}" -s TCP:LISTEN; then
      kill $(lsof -i ":${port}" -s TCP:LISTEN | awk '{print $2}')
=======
  for port in 8545 3301 3302 3303 3304 9091 9092 9093; do
    if lsof -i ":${port}" | grep -q 'LISTEN' && true || false; then
      kill $(lsof -i ":${port}" | grep 'LISTEN' | awk '{print $2}')
>>>>>>> 47d098493 (refactored tests)
    fi
  done

  exit $EXIT_CODE
}

trap cleanup EXIT

# $1 = rest port
# $2 = host port
# $3 = node data directory
# $4 = node log file
# $5 = node id file
# $6 = commands to run at startup
function setup_node() {
  local port=${1}
  local host_port=${2}
  local dir=${3}
  local log=${4}
  local id=${5}
  local cmds=${6:-""}

  echo "- Run node ${id} on rest port ${port}"

  if [ -n "$cmds" ]; then
    echo "- Executing '${cmds}' and exiting"
  fi

  DEBUG="hopr*" node packages/hoprd/lib/index.js \
    --init --provider=ws://127.0.0.1:8545/ \
    --testAnnounceLocalAddresses --identity="${id}" \
    --host="0.0.0.0:${host_port}" \
<<<<<<< HEAD
    --data="${dir}" --rest --restPort "${port}" --announce \
    --password="e2e-test" --testUseWeakCrypto > \
=======
    --run "${cmds}" \
    --data="${dir}" --rest --restPort "${port}" --announce > \
>>>>>>> 47d098493 (refactored tests)
    "${log}" 2>&1 &

  wait_for_http_port "${port}" "${log}" "${wait_delay}" "${wait_max_wait}"
}

# $1 = port
# $2 = node log file
function fund_node() {
  local port=${1}
  local log=${2}
  local api="127.0.0.1:${port}"

  local eth_address
  eth_address="$(curl --silent "${api}/api/v1/address/hopr")"

  if [ -z "${eth_address}" ]; then
    echo "- Can't fund node - couldn't load ETH address"
    exit 1
  fi

  echo "- Funding 1 ETH and 1 HOPR to ${eth_address}"
  yarn hardhat faucet --config packages/ethereum/hardhat.config.ts \
    --address "${eth_address}" --network localhost --ishopraddress true
}

# --- Log test info {{{
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
echo -e "\tnode4"
echo -e "\t\tdata dir: ${node4_dir} (will be removed)"
echo -e "\t\tlog: ${node4_log}"
echo -e "\t\tid: ${node4_id}"
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
# }}}

# --- Running Mock Blockchain --- {{{
echo "- Running hardhat local node"
DEVELOPMENT=true yarn hardhat node --config packages/ethereum/hardhat.config.ts \
  --network hardhat --as-network localhost --show-stack-traces > \
  "${hardhat_rpc_log}" 2>&1 &

echo "- Hardhat node started (127.0.0.1:8545)"
wait_for_http_port 8545 "${hardhat_rpc_log}" "${wait_delay}" "${wait_max_wait}"
# }}}

#  --- Run nodes --- {{{
setup_node 3301 9091 "${node1_dir}" "${node1_log}" "${node1_id}"
setup_node 3302 9092 "${node2_dir}" "${node2_log}" "${node2_id}"
setup_node 3303 9093 "${node3_dir}" "${node3_log}" "${node3_id}"
setup_node 3304 9094 "${node4_dir}" "${node4_log}" "${node4_id}" "info;balance"
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
# }}}

# --- Run test --- {{{
${mydir}/../test/integration-test.sh \
  "localhost:3301" "localhost:3302" "localhost:3303"
# }}}
