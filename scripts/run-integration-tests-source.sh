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
declare api_token="e2e-API-token^^"

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
declare tmp="$(find_tmp_dir)"

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

  local log exit_code non_zero
  for node_log in "${node1_log}" "${node2_log}" "${node3_log}" "${node4_log}" "${node5_log}" "${node6_log}" "${node7_log}"; do
    log=$(wait_for_regex ${node_log} "Process exiting with signal [0-9]")

    if [ -z "${log}" ]; then
      log "${node_log}"
      log "Process did not exit properly"
      exit 1
    fi

    exit_code=$(echo ${log} | sed -E "s/.*signal[ ]([0-9]+).*/\1/")
    if [ ${exit_code} != "0" ]; then
      non_zero=true
      log "${node_log}"
      log "terminated with non-zero exit code ${exit_code}"
    fi
  done

  if [ ${non_zero} ]; then
    exit 1
  else
    exit $EXIT_CODE
  fi
}

if [ "${skip_cleanup}" != "1" ] && [ "${skip_cleanup}" != "true" ]; then
  trap cleanup SIGINT SIGTERM ERR EXIT
fi

# $1 = api port
# $2 = node port
# $3 = admin port
# $4 = node data directory
# $5 = node log file
# $6 = node id file
# $7 = OPTIONAL: additional args to hoprd
function setup_node() {
  local api_port=${1}
  local node_port=${2}
  local admin_port=${3}
  local dir=${4}
  local log=${5}
  local id=${6}
  local additional_args=${7:-""}

  log "Run node ${id} on API port ${api_port} -> ${log}"

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
    --api-token "${api_token}" \
    --data="${dir}" \
    --host="127.0.0.1:${node_port}" \
    --identity="${id}" \
    --init \
    --password="${password}" \
    --api \
    --apiPort "${api_port}" \
    --testAnnounceLocalAddresses \
    --testPreferLocalAddresses \
    --testUseWeakCrypto \
    --testNoUPNP \
    --allowLocalNodeConnections \
    --allowPrivateNodeConnections \
    --heartbeatInterval 2500 \
    --heartbeatVariance 1000 \
    ${additional_args} \
    > "${log}" 2>&1 &
}

# $1 = node log file
# $2 = private key, must be hex string of length 66
# $3 = health check port
# $4 = data directory
# $5 = OPTIONAL: additional args to ct daemon
function setup_ct_node() {
  local log=${1}
  local private_key=${2}
  local health_check_port=${3}
  local dir=${4}
  local additional_args=${5:-""}

  log "Run CT node -> ${log}"

  if [[ "${additional_args}" != *"--environment "* ]]; then
    additional_args="--environment hardhat-localhost ${additional_args}"
  fi
  log "Additional args: \"${additional_args}\""

  DEBUG="hopr*" NODE_ENV=development node packages/cover-traffic-daemon/lib/index.js \
    --privateKey "${private_key}" \
    --dbFile "${ct_db_file}" \
    --data="${dir}" \
    --healthCheck \
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
rm -Rf \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost" \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost2"
# }}}

# --- Running Mock Blockchain --- {{{
log "Running hardhat local node"
HOPR_ENVIRONMENT_ID="hardhat-localhost" \
TS_NODE_PROJECT=${mydir}/../packages/ethereum/tsconfig.hardhat.json \
yarn workspace @hoprnet/hopr-ethereum hardhat node \
  --network hardhat \
  --show-stack-traces > \
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

# static address because static private key
declare ct_node1_address="0xDe913EeED23Bce5274eAD3dE8c196A41176fbd49"

#  --- Run nodes --- {{{
setup_node 13301 19091 19501 "${node1_dir}" "${node1_log}" "${node1_id}" "--announce"
setup_node 13302 19092 19502 "${node2_dir}" "${node2_log}" "${node2_id}" "--announce --testNoAuthentication"
setup_node 13303 19093 19503 "${node3_dir}" "${node3_log}" "${node3_id}" "--announce"
setup_node 13304 19094 19504 "${node4_dir}" "${node4_log}" "${node4_id}" "--testNoDirectConnections"
setup_node 13305 19095 19505 "${node5_dir}" "${node5_log}" "${node5_id}" "--testNoDirectConnections"
setup_node 13306 19096 19506 "${node6_dir}" "${node6_log}" "${node6_id}" "--announce --run \"info;balance\""
# should not be able to talk to the rest
setup_node 13307 19097 19507 "${node7_dir}" "${node7_log}" "${node7_id}" "--announce --environment hardhat-localhost2"
setup_ct_node "${ct_node1_log}" "0xa08666bca1363cb00b5402bbeb6d47f6b84296f3bba0f2f95b1081df5588a613" 20000 "${ct_node1_dir}" 
# }}}

log "CT node1 address: ${ct_node1_address}"

# DO NOT MOVE THIS STEP
#  --- Wait until private key has been created or recovered --- {{{
wait_for_regex ${node1_log} "please fund this node"
wait_for_regex ${node2_log} "please fund this node"
wait_for_regex ${node3_log} "please fund this node"
wait_for_regex ${node4_log} "please fund this node"
wait_for_regex ${node5_log} "please fund this node"
wait_for_regex ${node6_log} "please fund this node"
wait_for_regex ${node7_log} "please fund this node"
# }}}

log "Funding nodes"
#  --- Fund nodes --- {{{
HOPR_ENVIRONMENT_ID=hardhat-localhost \
TS_NODE_PROJECT=${mydir}/../packages/ethereum/tsconfig.hardhat.json \
yarn workspace @hoprnet/hopr-ethereum hardhat faucet \
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

#  --- Ensure data directories are used --- {{{
for node_dir in ${node1_dir} ${node2_dir} ${node3_dir} ${node4_dir} ${node5_dir}; do
  declare node_dir_db="${node_dir}/db/LOG"
  declare node_dir_peerstore="${node_dir}/peerstore/LOG"
  [ -f "${node_dir_db}" ] || { echo "Data file ${node_dir_db} missing"; exit 1; }
  [ -f "${node_dir_peerstore}" ] || { echo "Data file ${node_dir_peerstore} missing"; exit 1; }
done
# }}}

log "All nodes came up online"

# --- Run security tests --- {{{
${mydir}/../test/security-test.sh \
  127.0.0.1 13301 13302 19501 19502 "${api_token}"
# }}}

# --- Run protocol test --- {{{
HOPRD_API_TOKEN="${api_token}" ${mydir}/../test/integration-test.sh \
  "localhost:13301" "localhost:13302" "localhost:13303" "localhost:13304" "localhost:13305" "localhost:13306" "localhost:13307"
# }}}

# -- Verify node6 has executed the commands {{{
log "Verifying node6 log output"
grep -E "HOPR Balance: +20000 txHOPR" "${node6_log}"
grep -E "ETH Balance: +10 xDAI" "${node6_log}"
grep -E "Running on: hardhat" "${node6_log}"
log "Output of node6 correct"
# }}}

# -- CT test {{{
${mydir}/../test/ct-test.sh \
  "${ct_node1_log}" "127.0.0.1" 20000
# }}}
