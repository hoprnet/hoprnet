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
  msg "Usage: $0 [-h|--help] [-s|--skip-cleanup]"
  msg
  msg "The cleanup process can be skipped by using '--skip-cleanup'."
  msg
}

# verify and set parameters
declare wait_delay=2
declare wait_max_wait=1000
declare skip_cleanup="false"

while (( "$#" )); do
  case "$1" in
    -h|--help)
      # return early with help info when requested
      usage
      exit 0
      ;;
    -s|--skip-cleanup)
      skip_cleanup="true"
      shift
      ;;
    -*|--*=)
      usage
      exit 1
      ;;
    *)
      shift
      ;;
  esac
done

if [ "${CI:-}" = "true" ] && [ -z "${ACT:-}" ]; then
  wait_delay=10
  wait_max_wait=10
fi

# find usable tmp dir
declare tmp="/tmp"
[[ -d "${tmp}" && -h "${tmp}" ]] && tmp="/var/tmp"
[[ -d "${tmp}" && -h "${tmp}" ]] && { msg "Neither /tmp or /var/tmp can be used for writing logs"; exit 1; }

declare node_prefix="hopr-source-node"

declare node1_dir="${tmp}/${node_prefix}-1"
declare node2_dir="${tmp}/${node_prefix}-2"
declare node3_dir="${tmp}/${node_prefix}-3"
declare node4_dir="${tmp}/${node_prefix}-4"
declare node5_dir="${tmp}/${node_prefix}-5"
declare node6_dir="${tmp}/${node_prefix}-6"
declare node7_dir="${tmp}/${node_prefix}-7"

declare ct_node1_dir="${tmp}/${node_prefix}-ct1"

declare node1_log="${node1_dir}.log"
declare node2_log="${node2_dir}.log"
declare node3_log="${node3_dir}.log"
declare node4_log="${node4_dir}.log"
declare node5_log="${node5_dir}.log"
declare node6_log="${node6_dir}.log"
declare node7_log="${node7_dir}.log"

declare ct_node1_log="${ct_node1_dir}.log"

declare node1_id="${node1_dir}.id"
declare node2_id="${node2_dir}.id"
declare node3_id="${node3_dir}.id"
declare node4_id="${node4_dir}.id"
declare node5_id="${node5_dir}.id"
declare node6_id="${node6_dir}.id"
declare node7_id="${node7_dir}.id"

declare password="e2e-test"
declare ct_private_key="0x123456"

declare ct_db_file="${tmp}/hopr-ct-db.json"

declare hardhat_rpc_log="${tmp}/hopr-source-hardhat-rpc.log"

function cleanup {
  local EXIT_CODE=$?

  # at this point we don't want to fail hard anymore
  trap - SIGINT SIGTERM ERR EXIT
  set +Eeuo pipefail

  # Cleaning up everything
  log "Wiping databases"
  rm -rf "${node1_dir}" "${node2_dir}" "${node3_dir}" "${node4_dir}" "${node5_dir}" "${node6_dir}" "${node7_dir}" "${ct_node1_dir}"

  log "Cleaning up processes"
  for port in 8545 13301 13302 13303 13304 13305 13306 13307 19091 19092 19093 19094 19095 19096 19097 20000; do
    lsof -i ":${port}" -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}
  done

  exit $EXIT_CODE
}

if [ "${skip_cleanup}" != "1" ] && [ "${skip_cleanup}" != "true" ]; then
  trap cleanup SIGINT SIGTERM ERR EXIT
fi

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

  log "Run node ${id} on rest port ${rest_port} -> ${log}"

  if [[ "${additional_args}" != *"--environment "* ]]; then
    additional_args="--environment hardhat-localhost ${additional_args}"
  fi

  log "Additional args: \"${additional_args}\""

  # Set NODE_ENV=development to rebuild hopr-admin next files
  # at runtime. Necessary to start multiple instances of hoprd
  # in parallel
  DEBUG="hopr*" NODE_ENV=development node packages/hoprd/lib/index.js \
    --admin \
    --adminHost "127.0.0.1" \
    --adminPort ${admin_port} \
    --announce \
    --api-token "e2e-API-token^^" \
    --data="${dir}" \
    --host="127.0.0.1:${node_port}" \
    --identity="${id}" \
    --init \
    --password="${password}" \
    --rest \
    --restPort "${rest_port}" \
    --testAnnounceLocalAddresses \
    --testPreferLocalAddresses \
    --testUseWeakCrypto \
    ${additional_args} \
    > "${log}" 2>&1 &
}

# $1 = node log file
# $2 = health check port
# $3 = OPTIONAL: additional args to ct daemon
function setup_ct_node() {
  local log=${1}
  local health_check_port=${2}
  local additional_args=${3:-""}

  log "Run CT node -> ${log}"

  if [[ "${additional_args}" != *"--environment "* ]]; then
    additional_args="--environment hardhat-localhost ${additional_args}"
  fi
  log "Additional args: \"${additional_args}\""

  DEBUG="hopr*" NODE_ENV=development node packages/cover-traffic-daemon/lib/index.js \
    --privateKey "${ct_private_key}" \
    --dbFile "${ct_db_file}" \
    --healthCheckHost "127.0.0.1" \
    --healthCheckPort "${health_check_port}" \
    ${additional_args} \
     > "${log}" 2>&1 &
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
log "\tnode5"
log "\t\tdata dir: ${node5_dir} (will be removed)"
log "\t\tlog: ${node5_log}"
log "\t\tid: ${node5_id}"
log "\tnode6"
log "\t\tdata dir: ${node6_dir} (will be removed)"
log "\t\tlog: ${node6_log}"
log "\t\tid: ${node6_id}"
log "\tnode7"
log "\t\tdata dir: ${node7_dir} (will be removed)"
log "\t\tlog: ${node7_log}"
log "\t\tid: ${node7_id}"
log "\tct_node1"
log "\t\tdata dir: ${ct_node1_dir} (will be removed)"
log "\t\tlog: ${ct_node1_log}"
# }}}

# --- Check all resources we need are free {{{
ensure_port_is_free 8545
ensure_port_is_free 13301
ensure_port_is_free 13302
ensure_port_is_free 13303
ensure_port_is_free 13304
ensure_port_is_free 13305
ensure_port_is_free 13306
ensure_port_is_free 13307
ensure_port_is_free 19091
ensure_port_is_free 19092
ensure_port_is_free 19093
ensure_port_is_free 19094
ensure_port_is_free 19095
ensure_port_is_free 19096
ensure_port_is_free 19097
ensure_port_is_free 20000
# }}}

# --- Cleanup old contract deployments {{{
log "Removing artifacts from old contract deployments"
rm -Rfv \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost" \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost2"
# }}}

# --- Running Mock Blockchain --- {{{
log "Running hardhat local node"
HOPR_ENVIRONMENT_ID="hardhat-localhost" yarn workspace @hoprnet/hopr-ethereum hardhat node \
  --network hardhat --show-stack-traces > \
  "${hardhat_rpc_log}" 2>&1 &

wait_for_regex ${hardhat_rpc_log} "Started HTTP and WebSocket JSON-RPC server"
log "Hardhat node started (127.0.0.1:8545)"

# need to mirror contract data because of hardhat-deploy node only writing to localhost
cp -R \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost/localhost" \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost/hardhat"
cp -R \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost" \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost2"
# }}}

#  --- Run nodes --- {{{
setup_node 13301 19091 19501 "${node1_dir}" "${node1_log}" "${node1_id}"
setup_node 13302 19092 19502 "${node2_dir}" "${node2_log}" "${node2_id}" "--testNoAuthentication"
setup_node 13303 19093 19503 "${node3_dir}" "${node3_log}" "${node3_id}"
setup_node 13304 19094 19504 "${node4_dir}" "${node4_log}" "${node4_id}"
setup_node 13305 19095 19505 "${node5_dir}" "${node5_log}" "${node5_id}"
setup_node 13306 19096 19506 "${node6_dir}" "${node6_log}" "${node6_id}" "--run \"info;balance\""
setup_node 13307 19097 19507 "${node7_dir}" "${node7_log}" "${node7_id}" "--environment hardhat-localhost2" # should not be able to talk to the rest
setup_ct_node "${ct_node1_log}" 20000
# }}}

log "Waiting for nodes startup"

#  --- Wait until started --- {{{
# Wait until node has recovered its private key
wait_for_regex ${node1_log} "using blockchain address"
wait_for_regex ${node2_log} "using blockchain address"
wait_for_regex ${node3_log} "using blockchain address"
wait_for_regex ${node4_log} "using blockchain address"
wait_for_regex ${node5_log} "using blockchain address"
wait_for_regex ${node6_log} "using blockchain address"
wait_for_regex ${node7_log} "using blockchain address"
declare ct_node1_address=$(wait_for_regex ${ct_node1_log} "Address: " | cut -d " " -f 4)
# }}}

log "CT node1 address: ${ct_node1_address}"

log "Funding nodes"

#  --- Fund nodes --- {{{
HOPR_ENVIRONMENT_ID=hardhat-localhost yarn workspace @hoprnet/hopr-ethereum hardhat faucet \
  --identity-prefix "${node_prefix}" \
  --identity-directory "${tmp}" \
  --use-local-identities \
  --network hardhat \
  --address "${ct_node1_address}" \
  --password "${password}"
# }}}

log "Waiting for port binding"

#  --- Wait for ports to be bound --- {{{
wait_for_regex ${node1_log} "STARTED NODE"
wait_for_regex ${node2_log} "STARTED NODE"
wait_for_regex ${node3_log} "STARTED NODE"
wait_for_regex ${node4_log} "STARTED NODE"
wait_for_regex ${node5_log} "STARTED NODE"
# no need to wait for node 6 since that will stop right away
wait_for_port 19097 "127.0.0.1" "${node7_log}"
# }}}

log "All nodes came up online"

# --- Run security tests --- {{{
${mydir}/../test/security-test.sh \
  127.0.0.1 13301 19501 19502
# }}}

# --- Run protocol test --- {{{
${mydir}/../test/integration-test.sh \
  "localhost:13301" "localhost:13302" "localhost:13303" "localhost:13304" "localhost:13305" "localhost:13306" "localhost:13307"
# }}}

# -- Verify node6 has executed the commands {{{
log "Verifying node6 log output"
grep -E "HOPR Balance: +10 txHOPR" "${node6_log}"
grep -E "ETH Balance: +1 xDAI" "${node6_log}"
grep -E "Running on: hardhat" "${node6_log}"
# }}}

# -- CT test {{{
${mydir}/../test/ct-test.sh \
  "${ct_node1_log}" "127.0.0.01" 20000
# }}}
