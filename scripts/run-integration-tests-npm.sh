#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1;
}

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

usage() {
  echo >&2
  echo "Usage: $0 [<npm_package_version>]" >&2
  echo
  echo -e "\twhere <npm_package_version> uses the most recent Git tag as default"
  echo >&2
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
declare mydir
declare npm_package_version

mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

# we rely on Git tags so need to fetch the tags in case they are not present
git fetch --unshallow --tags || :
npm_package_version=${1:-$(git describe --abbrev=0)}

# set log id and use shared log function for readable logs
declare HOPR_LOG_ID="e2e-npm-test"
source "${mydir}/utils.sh"

declare wait_delay=2
declare wait_max_wait=1000

if [ -n "${CI:-}" ]; then
  wait_delay=10
  wait_max_wait=10
fi

declare npm_install_dir="/tmp/hopr-npm"

declare node1_dir="/tmp/hopr-npm-node-1"
declare node2_dir="/tmp/hopr-npm-node-2"
declare node3_dir="/tmp/hopr-npm-node-3"
declare node4_dir="/tmp/hopr-npm-node-4"

declare node1_log="${node1_dir}.log"
declare node2_log="${node2_dir}.log"
declare node3_log="${node3_dir}.log"
declare node4_log="${node4_dir}.log"

declare node1_id="${node1_dir}.id"
declare node2_id="${node2_dir}.id"
declare node3_id="${node3_dir}.id"
declare node4_id="${node4_dir}.id"

declare hardhat_rpc_log="/tmp/hopr-npm-hardhat-rpc.log"

function cleanup {
  local EXIT_CODE=$?

  # Cleaning up everything
  if [ "$EXIT_CODE" != "0" ]; then
    echo "- Exited with fail, code $EXIT_CODE"
    for log_file in "${node1_log}" "${node2_log}" "${node3_log}"; do
      if [ -n "${log_file}" ] && [ -f "${log_file}" ]; then
        echo "- Printing last 100 lines from logs"
        tail -n 100 "${node1_log}" "${node2_log}" "${node3_log}"
        echo "- Printing last 100 lines from logs DONE"
      fi
    done
  fi

  echo -e "\n- Wiping databases"
  rm -rf "${node1_dir}" "${node2_dir}" "${node3_dir}" "${npm_install_dir}"

  echo "- Cleaning up processes"
  for port in 8545 3301 3302 3303 3304 9091 9092 9093 9094; do
    if lsof -i ":${port}" -s TCP:LISTEN; then
      lsof -i ":${port}" -s TCP:LISTEN -t | kill
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
# $6 = npm package version
# $7 = OPTIONAL: additions args to hoprd
function setup_node() {
  local port=${1}
  local host_port=${2}
  local dir=${3}
  local log=${4}
  local id=${5}
  local version=${6}
  local additional_args=${7:-""}

  if [ -n "${additional_args}" ]; then
    echo "- Additional args: \"${additional-args}\""
  fi

  mkdir -p "${npm_install_dir}"
  yarn --cwd "${npm_install_dir}" add @hoprnet/hoprd@${version}

  echo "- Run node ${id} on rest port ${port}"
  DEBUG="hopr*" yarn --cwd "${npm_install_dir}" hoprd \
    --init --provider=ws://127.0.0.1:8545/ \
    --testAnnounceLocalAddresses --identity="${id}" \
    --host="0.0.0.0:${host_port}" \
    --data="${dir}" --rest --restPort "${port}" --announce \
    --password="e2e-test" --testUseWeakCrypto \
    ${additional_args} \
    > "${log}" 2>&1 &

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
echo -e "\tnpm package"
echo -e "\t\tdir: ${npm_install_dir} (will be removed)"
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
ensure_port_is_free 9094
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
setup_node 3301 9091 "${node1_dir}" "${node1_log}" "${node1_id}" "${npm_package_version}"
setup_node 3302 9092 "${node2_dir}" "${node2_log}" "${node2_id}" "${npm_package_version}"
setup_node 3303 9093 "${node3_dir}" "${node3_log}" "${node3_id}" "${npm_package_version}"
setup_node 3304 9094 "${node4_dir}" "${node4_log}" "${node4_id}" "${npm_package_version}" "--run \"info;balance\""
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

# -- Verify node4 has executed the commands {{{
echo "- Verifying node4 log output"
grep -q "^HOPR Balance:" "${node4_log}" 
grep -q "^Running on: localhost" "${node4_log}" 
#}}}
