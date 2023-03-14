#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="e2e-source-test"

source "${mydir}/testnet.sh"
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
declare default_api_token="e2e-API-token^^"

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

declare node1_log="${node1_dir}.log"
declare node2_log="${node2_dir}.log"
declare node3_log="${node3_dir}.log"
declare node4_log="${node4_dir}.log"
declare node5_log="${node5_dir}.log"
declare node6_log="${node6_dir}.log"
declare node7_log="${node7_dir}.log"

declare node1_id="${node1_dir}.id"
declare node2_id="${node2_dir}.id"
declare node3_id="${node3_dir}.id"
declare node4_id="${node4_dir}.id"
declare node5_id="${node5_dir}.id"
declare node6_id="${node6_dir}.id"
declare node7_id="${node7_dir}.id"

declare node1_privkey="0x1f5b172a64947589be6e279fbcbc09aca6e623a64a92aa359fae9c6613b7e801"
declare node2_privkey="0xcb9c3533beb75b996b6c77150ecda32134d13710a16121f04dc591113329cd7c"
declare node3_privkey="0x9a96a7711e2e9c9f71767bb9f248f699b29aebe7f590de8eeec0e71796b869e0"
declare node4_privkey="0x7dea49b4dbeea4dcbbb9d071bc7212347748dc3a2f16896f504417236b6adb84"
declare node5_privkey="0x800fee12d472c1a8448b786eb9e5d6c7f643c78b9727032893da9a6a55db288b"
declare node6_privkey="0x79b94be0c06dac87139c54416228dcacfb084c6884bbf4e48fff4cab8f40baa6"
declare node7_privkey="0x9b813edd8a85cffbe3cd2e242dc0992cfa04be15caa9f50b0b03b5ebcb2f770a"

declare password="e2e-test"

declare anvil_rpc_log="${tmp}/hopr-source-anvil-rpc.log"
declare anvil_cfg_file="${tmp}/hopr-source-anvil.cfg"

# anvil port
declare -a all_ports=( 8545 )
# HOPRd API ports
all_ports+=( 13301 13302 13303 13304 13305 13306 13307 )
# HOPRd p2p ports
all_ports+=( 19091 19092 19093 19094 19095 19096 19097 )

function cleanup {
  local EXIT_CODE=$?

  # at this point we don't want to fail hard anymore
  trap - SIGINT SIGTERM ERR EXIT
  set +Eeuo pipefail

  # Cleaning up everything
  log "Cleaning up processes"
  for port in ${all_ports[@]}; do
    lsof -i ":${port}" -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}
  done

  local log exit_code non_zero
  for node_log in "${node1_log}" "${node2_log}" "${node3_log}" "${node4_log}" "${node5_log}" "${node6_log}" "${node7_log}"; do
    log=$(grep -E "Process exiting with signal [0-9]" ${node_log} || echo "")

    if [ -z "${log}" ]; then
      log "${node_log}: Process did not exit properly"
      exit_code=1
    else
      exit_code=$(echo ${log} | sed -E "s/.*signal[ ]([0-9]+).*/\1/")
    fi

    if [ ${exit_code} != "0" ]; then
      non_zero=true
      log "${node_log}: terminated with non-zero exit code ${exit_code}"
    fi
  done

  log "Wiping databases"
  rm -rf "${node1_dir}" "${node2_dir}" "${node3_dir}" "${node4_dir}" "${node5_dir}" "${node6_dir}" "${node7_dir}"

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
# $2 = api token
# $3 = node port
# $4 = node data directory
# $5 = node log file
# $6 = node id file
# $7 = private key to use
# $8 = OPTIONAL: additional args to hoprd
function setup_node() {
  local api_port=${1}
  local api_token=${2}
  local node_port=${3}
  local dir=${4}
  local log=${5}
  local id=${6}
  local private_key=${7}
  local additional_args=${8:-""}

  log "Run node ${id} on API port ${api_port} -> ${log}"

  if [[ "${additional_args}" != *"--environment "* ]]; then
    additional_args="--environment anvil-localhost ${additional_args}"
  fi

  if [[ -n "${api_token}" ]]; then
    additional_args="--api-token='${api_token}' ${additional_args}"
  else
    additional_args="--disableApiAuthentication ${additional_args}"
  fi

  # Remove previous logs to make sure the regex does not match
  rm -f "${log}"

  log "Additional args: \"${additional_args}\""

  # Using a mix of CLI parameters and env variables to ensure
  # both work.
  env \
    DEBUG="hopr*" \
    NODE_ENV=development \
    HOPRD_HEARTBEAT_INTERVAL=2500 \
    HOPRD_HEARTBEAT_THRESHOLD=2500 \
    HOPRD_HEARTBEAT_VARIANCE=1000 \
    HOPRD_NETWORK_QUALITY_THRESHOLD="0.3" \
    HOPRD_ON_CHAIN_CONFIRMATIONS=2 \
    NODE_OPTIONS="--experimental-wasm-modules" \
    node packages/hoprd/lib/main.cjs \
      --data="${dir}" \
      --host="127.0.0.1:${node_port}" \
      --identity="${id}" \
      --init \
      --password="${password}" \
      --privateKey="${private_key}" \
      --api \
      --apiPort "${api_port}" \
      --testAnnounceLocalAddresses \
      --testPreferLocalAddresses \
      --testUseWeakCrypto \
      --allowLocalNodeConnections \
      --allowPrivateNodeConnections \
      ${additional_args} \
      > "${log}" 2>&1 &
}

# --- Log test info {{{
log "Test files and directories"
log "\tanvil"
log "\t\tlog: ${anvil_rpc_log}"
log "\t\tcfg: ${anvil_cfg_file}"
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
# }}}

# --- Check all resources we need are free {{{
for p in ${all_ports[@]}; do
  ensure_port_is_free "${p}"
done
# }}}

declare protocol_config="${mydir}/../packages/core/protocol-config.json"
declare deployments_summary="${mydir}/../packages/ethereum/contracts/contracts-addresses.json"

# --- Running Mock Blockchain --- {{{
${mydir}/run-local-anvil.sh -l "${anvil_rpc_log}" -c "${anvil_cfg_file}"

# read auto-generated private key from anvil configuration
declare anvil_private_key
anvil_private_key="$(jq -r ".private_keys[0]" "${anvil_cfg_file}")"
if [ -z "${anvil_private_key}" ]; then
  log "Could not find private key in anvil cfg file ${anvil_cfg_file}"
  exit 1
fi
# we export the private key so it gets picked up by other sub-shells
export PRIVATE_KEY=${anvil_private_key}

# need to mirror contract data because of anvil-deploy node only writing to localhost {{{
update_protocol_config_addresses "${protocol_config}" "${deployments_summary}" "anvil-localhost" "anvil-localhost"
update_protocol_config_addresses "${protocol_config}" "${deployments_summary}" "anvil-localhost" "anvil-localhost2"
# }}}

#  --- Run nodes --- {{{
setup_node 13301 ${default_api_token} 19091 "${node1_dir}" "${node1_log}" "${node1_id}" "${node1_privkey}" "--announce"
# use empty auth token to be able to test this in the security tests
setup_node 13302 ""                   19092 "${node2_dir}" "${node2_log}" "${node2_id}" "${node2_privkey}" "--announce"
setup_node 13303 ${default_api_token} 19093 "${node3_dir}" "${node3_log}" "${node3_id}" "${node3_privkey}" "--announce"
setup_node 13304 ${default_api_token} 19094 "${node4_dir}" "${node4_log}" "${node4_id}" "${node4_privkey}" "--testNoDirectConnections"
setup_node 13305 ${default_api_token} 19095 "${node5_dir}" "${node5_log}" "${node5_id}" "${node5_privkey}" "--testNoDirectConnections"
# should not be able to talk to the rest
setup_node 13306 ${default_api_token} 19096 "${node6_dir}" "${node6_log}" "${node6_id}" "${node6_privkey}" "--announce --environment anvil-localhost2"
# node n8 will be the only one NOT registered
setup_node 13307 ${default_api_token} 19097 "${node7_dir}" "${node7_log}" "${node7_id}" "${node7_privkey}" "--announce"
# }}}

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
fund_nodes "${node_prefix}" "${tmp}" "${password}"
# }}}

log "Waiting for port binding"

#  --- Wait for ports to be bound --- {{{
wait_for_regex ${node1_log} "STARTED NODE"
wait_for_regex ${node2_log} "STARTED NODE"
wait_for_regex ${node3_log} "STARTED NODE"
wait_for_regex ${node4_log} "STARTED NODE"
wait_for_regex ${node5_log} "STARTED NODE"
wait_for_regex ${node6_log} "STARTED NODE"
wait_for_port 19096 "127.0.0.1" "${node6_log}"
wait_for_regex ${node7_log} "STARTED NODE"
# }}}

#  --- Ensure data directories are used --- {{{
for node_dir in ${node1_dir} ${node2_dir} ${node3_dir} ${node4_dir} ${node5_dir} ${node6_dir} ${node7_dir}; do
  declare node_dir_db="${node_dir}/db/LOG"
  declare node_dir_peerstore="${node_dir}/peerstore/LOG"
  [ -f "${node_dir_db}" ] || { echo "Data file ${node_dir_db} missing"; exit 1; }
  [ -f "${node_dir_peerstore}" ] || { echo "Data file ${node_dir_peerstore} missing"; exit 1; }
done
# }}}

log "All nodes came up online"

# --- Run security tests --- {{{
${mydir}/../test/security-test.sh \
  127.0.0.1 13301 13302 "${default_api_token}"
# }}}

# --- Run protocol test --- {{{
ADDITIONAL_NODE_ADDRS="0xde913eeed23bce5274ead3de8c196a41176fbd49" \
ADDITIONAL_NODE_PEERIDS="16Uiu2HAm2VD6owCxPEZwP6Moe1jzapqziVeaTXf1h7jVzu5dW1mk" \
HOPRD_API_TOKEN="${default_api_token}" \
${mydir}/../test/integration-test.sh \
  "127.0.0.1:13301" "127.0.0.1:13302" "127.0.0.1:13303" "127.0.0.1:13304" "127.0.0.1:13305" "127.0.0.1:13306" "127.0.0.1:13307"
# }}}

