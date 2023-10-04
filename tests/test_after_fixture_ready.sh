#!/usr/bin/env bash

# HOPR interaction tests via HOPRd API v3

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="e2e-test"
source "${mydir}/../scripts/utils.sh"
source "${mydir}/../scripts/testnet.sh"
source "${mydir}/../scripts/api.sh"

usage() {
  msg
  msg "Usage: $0 <node_api_1> <node_api_2> <node_api_3> <node_api_4> <node_api_5> <node_api_6> <node_api_7>"
  msg
  msg "Required environment variables"
  msg "------------------------------"
  msg
  msg "HOPRD_API_TOKEN\t\t\tused as api token for all nodes"
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
test -z "${1:-}" && { msg "Missing 1st parameter"; usage; exit 1; }
test -z "${2:-}" && { msg "Missing 2nd parameter"; usage; exit 1; }
test -z "${3:-}" && { msg "Missing 3rd parameter"; usage; exit 1; }
test -z "${4:-}" && { msg "Missing 4th parameter"; usage; exit 1; }
test -z "${5:-}" && { msg "Missing 5th parameter"; usage; exit 1; }
test -z "${6:-}" && { msg "Missing 6th parameter"; usage; exit 1; }
test -z "${7:-}" && { msg "Missing 7th parameter"; usage; exit 1; }
test -z "${HOPRD_API_TOKEN:-}" && { msg "Missing HOPRD_API_TOKEN"; usage; exit 1; }

declare api1="${1}"
declare api2="${2}"
declare api3="${3}"
declare api4="${4}"
declare api5="${5}"
declare api6="${6}"
declare api7="${7}"

declare api_token=${HOPRD_API_TOKEN}
declare additional_nodes_safe_addrs="${ADDITIONAL_NODE_SAFE_ADDRS:-}"
declare additional_nodes_native_addrs="${ADDITIONAL_NODE_NATIVE_ADDRS:-}"
declare msg_tag=1234

declare -a jobs

wait_for_jobs() {
  log "Waiting for ${1}"
  for j in ${jobs[@]}; do
    if ! wait -n $j; then
      log "Waiting for ${1} - FAILED job ${j}"
      exit 1
    fi;
  done;
  jobs=()
  log "Waiting for ${1} - DONE"
}

# $1 native addresses ("Ethereum addresses"), comma-separated list
# $2 peerIds, comma-separated list
register_nodes() {
  log "Registering nodes"

  make -C "${mydir}/.." register-nodes \
    network=anvil-localhost environment_type=local \
    staking_addresses="[${1}]" \
    node_addresses="[${2}]"

  log "Registering nodes finished"
}

# $1 - native addresses ("Ethereum addresses"), comma-separated list
sync_staking_accounts_in_network_registry() {
  log "Sync nodes in network registry"

  make -C "${mydir}/.." sync-eligibility \
    network=anvil-localhost environment_type=local \
    staking_addresses="[${1}]"

  log "Sync accounts in network registry finished"
}

log "Running full E2E test with ${api1}, ${api2}, ${api3}, ${api4}, ${api5}, ${api6}, ${api7}"

# real blockchain networks

# TODO: api6 becomes unavailable soon, because it crashes, restore, once network separation works properly.

validate_native_address "${api1}" "${api_token}" & jobs+=( "$!" )
validate_native_address "${api2}" "${api_token}" & jobs+=( "$!" )
validate_native_address "${api3}" "${api_token}" & jobs+=( "$!" )
validate_native_address "${api4}" "${api_token}" & jobs+=( "$!" )
validate_native_address "${api5}" "${api_token}" & jobs+=( "$!" )
# validate_native_address "${api6}" "${api_token}" & jobs+=( "$!" )
validate_native_address "${api7}" "${api_token}" & jobs+=( "$!" )
wait_for_jobs "ETH addresses exist"
echo "got here"

api_validate_balances_gt0 "${api_token}@${api1}" & jobs+=( "$!" )
api_validate_balances_gt0 "${api_token}@${api2}" & jobs+=( "$!" )
api_validate_balances_gt0 "${api_token}@${api3}" & jobs+=( "$!" )
api_validate_balances_gt0 "${api_token}@${api4}" & jobs+=( "$!" )
api_validate_balances_gt0 "${api_token}@${api5}" & jobs+=( "$!" )
# api_validate_balances_gt0 "${api_token}@${api6}" & jobs+=( "$!" )
api_validate_balances_gt0 "${api_token}@${api7}" & jobs+=( "$!" )
wait_for_jobs "Nodes and Safes are funded"

declare addr1 addr2 addr3 addr4 addr5 addr6 addr7 result
addr1="$(get_hopr_address "${api_token}@${api1}")"
addr2="$(get_hopr_address "${api_token}@${api2}")"
addr3="$(get_hopr_address "${api_token}@${api3}")"
addr4="$(get_hopr_address "${api_token}@${api4}")"
addr5="$(get_hopr_address "${api_token}@${api5}")"
addr6="INVALID"   #"$(get_hopr_address "${api_token}@${api6}")"
addr7="$(get_hopr_address "${api_token}@${api7}")"

function get_safe_address() {
  api_get_node_info "$1" | jq -r '.nodeSafe'
}

declare safe_addr1 safe_addr2 safe_addr3 safe_addr4 safe_addr5 safe_addr6 safe_addr7
safe_addr1="$(get_safe_address "${api_token}@${api1}")"
safe_addr2="$(get_safe_address "${api_token}@${api2}")"
safe_addr3="$(get_safe_address "${api_token}@${api3}")"
safe_addr4="$(get_safe_address "${api_token}@${api4}")"
safe_addr5="$(get_safe_address "${api_token}@${api5}")"
safe_addr6="INVALID"  # $(get_safe_address "${api_token}@${api6}")"
safe_addr7="$(get_safe_address "${api_token}@${api7}")"

declare node_addr1 node_addr2 node_addr3 node_addr4 node_addr5 node_addr6 node_addr7
node_addr1="$(get_native_address "${api_token}@${api1}")"
node_addr2="$(get_native_address "${api_token}@${api2}")"
node_addr3="$(get_native_address "${api_token}@${api3}")"
node_addr4="$(get_native_address "${api_token}@${api4}")"
node_addr5="$(get_native_address "${api_token}@${api5}")"
node_addr6="INVALID"  #"$(get_native_address "${api_token}@${api6}")"
node_addr7="$(get_native_address "${api_token}@${api7}")"

log "hopr addr1: ${addr1} ${safe_addr1} ${node_addr1}"
log "hopr addr2: ${addr2} ${safe_addr2} ${node_addr2}"
log "hopr addr3: ${addr3} ${safe_addr3} ${node_addr3}"
log "hopr addr4: ${addr4} ${safe_addr4} ${node_addr4}"
log "hopr addr5: ${addr5} ${safe_addr5} ${node_addr5}"
log "hopr addr6: ${addr6} ${safe_addr6} ${node_addr6}"
log "hopr addr7: ${addr7} ${safe_addr7} ${node_addr7}"

# declare safe_addrs_to_register="$safe_addr1,$safe_addr2,$safe_addr3,$safe_addr4,$safe_addr5,$safe_addr6"
# declare node_addrs_to_register="$node_addr1,$node_addr2,$node_addr3,$node_addr4,$node_addr5,$node_addr6"
declare safe_addrs_to_register="$safe_addr1,$safe_addr2,$safe_addr3,$safe_addr4,$safe_addr5"
declare node_addrs_to_register="$node_addr1,$node_addr2,$node_addr3,$node_addr4,$node_addr5"

# add nodes 1,2,3,4,5,6 plus additional nodes in register, do NOT add node 7
log "Adding nodes to register"
if ! [ -z $additional_nodes_safe_addrs ] && ! [ -z $additional_nodes_native_addrs ]; then
  safe_addrs_to_register+=",${additional_nodes_safe_addrs}"
  node_addrs_to_register+=",${additional_nodes_native_addrs}"
fi

# Register nodes in the NR, emit "Registered" events
register_nodes "${safe_addrs_to_register}" "${node_addrs_to_register}"

# Sync nodes in the NR, emit "EligibilityUpdated" events
#sync_staking_accounts_in_network_registry "${safe_addrs_to_register}"

# running withdraw and checking it results at the end of this test run
balances=$(api_get_balances ${api1})
native_balance=$(echo ${balances} | jq -r .native)
api_withdraw ${api1} "NATIVE" 10 0x858aa354db6ae5ea1217c5018c90403bde94e09e