#!/usr/bin/env bash
# HOPR interaction tests via HOPRd API v2

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

# $1 = node id
# $2 = node api endpoint
redeem_tickets() {
  local node_id="${1}"
  local node_api="${2}"
  local rejected redeemed prev_redeemed

  # First get the inital ticket statistics for reference
  result=$(api_get_ticket_statistics ${node_api} "winProportion")
  log "Node ${node_id} ticket information (before redemption) -- ${result}"
  rejected=$(echo "${result}" | jq -r .rejected)
  redeemed=$(echo "${result}" | jq -r .redeemed)
  [[ ${rejected} -gt 0 ]] && { msg "rejected tickets count on node ${node_id} is ${rejected}"; exit 1; }
  last_redeemed="${redeemed}"

  # Trigger a redemption run, but cap it at 20 seconds. We only want to measure
  # progress, not redeeem all tickets which takes too long.
  log "Node ${node_id} should redeem all tickets"
  result=$(api_redeem_tickets ${node_api} 20)
  log "--${result}"

  # Get ticket statistics again and compare with previous state. Ensure we
  # redeemed tickets.
  result=$(api_get_ticket_statistics ${node_api} "winProportion")
  log "Node ${node_id} ticket information (after redemption) -- ${result}"
  rejected=$(echo "${result}" | jq -r .rejected)
  redeemed=$(echo "${result}" | jq -r .redeemed)
  [[ ${rejected} -gt 0 ]] && { msg "rejected tickets count on node ${node_id} is ${rejected}"; exit 1; }
  [[ ${redeemed} -gt 0 && ${redeemed} -gt ${last_redeemed} ]] || { msg "redeemed tickets count on node ${node_id} is ${redeemed}, previously ${last_redeemed}"; exit 1; }
  last_redeemed="${redeemed}"

  # Trigger another redemption run, but cap it at 20 seconds. We only want to measure
  # progress, not redeeem all tickets which takes too long.
  log "Node ${node_id} should redeem all tickets (again to ensure re-run of operation)"
  # add 60 second timeout
  result=$(api_redeem_tickets ${node_api} 20)
  log "--${result}"

  # Get final ticket statistics
  result=$(api_get_ticket_statistics ${node_api} "winProportion")
  log "Node ${node_id} ticket information (after second redemption) -- ${result}"
  rejected=$(echo "${result}" | jq -r .rejected)
  redeemed=$(echo "${result}" | jq -r .redeemed)
  [[ ${rejected} -gt 0 ]] && { msg "rejected tickets count on node ${node_id} is ${rejected}"; exit 1; }
  [[ ${redeemed} -gt 0 && ${redeemed} -gt ${last_redeemed} ]] || { msg "redeemed tickets count on node ${node_id} is ${redeemed}, previously ${last_redeemed}"; exit 1; }
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

validate_native_address "${api1}" "${api_token}" & jobs+=( "$!" )
validate_native_address "${api2}" "${api_token}" & jobs+=( "$!" )
validate_native_address "${api3}" "${api_token}" & jobs+=( "$!" )
validate_native_address "${api4}" "${api_token}" & jobs+=( "$!" )
validate_native_address "${api5}" "${api_token}" & jobs+=( "$!" )
validate_native_address "${api6}" "${api_token}" & jobs+=( "$!" )
validate_native_address "${api7}" "${api_token}" & jobs+=( "$!" )
for j in ${jobs[@]}; do wait -n $j; done; jobs=()
log "ETH addresses exist"

# TODO: Update check of safe's token balance instead of HOPR node's token balance
# api_validate_node_balance_gt0 "${api1}"
# api_validate_node_balance_gt0 "${api2}"
# api_validate_node_balance_gt0 "${api3}"
# api_validate_node_balance_gt0 "${api4}"
# api_validate_node_balance_gt0 "${api5}"
# api_validate_node_balance_gt0 "${api6}"
# api_validate_node_balance_gt0 "${api7}"
# log "Nodes are funded"

declare addr1 addr2 addr3 addr4 addr5 addr6 addr7 result
addr1="$(get_hopr_address "${api_token}@${api1}")"
addr2="$(get_hopr_address "${api_token}@${api2}")"
addr3="$(get_hopr_address "${api_token}@${api3}")"
addr4="$(get_hopr_address "${api_token}@${api4}")"
addr5="$(get_hopr_address "${api_token}@${api5}")"
addr6="$(get_hopr_address "${api_token}@${api6}")"
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
safe_addr6="$(get_safe_address "${api_token}@${api6}")"
safe_addr7="$(get_safe_address "${api_token}@${api7}")"

declare node_addr1 node_addr2 node_addr3 node_addr4 node_addr5 node_addr6 node_addr7
node_addr1="$(get_native_address "${api_token}@${api1}")"
node_addr2="$(get_native_address "${api_token}@${api2}")"
node_addr3="$(get_native_address "${api_token}@${api3}")"
node_addr4="$(get_native_address "${api_token}@${api4}")"
node_addr5="$(get_native_address "${api_token}@${api5}")"
node_addr6="$(get_native_address "${api_token}@${api6}")"
node_addr7="$(get_native_address "${api_token}@${api7}")"

log "hopr addr1: ${addr1} ${safe_addr1} ${node_addr1}"
log "hopr addr2: ${addr2} ${safe_addr2} ${node_addr2}"
log "hopr addr3: ${addr3} ${safe_addr3} ${node_addr3}"
log "hopr addr4: ${addr4} ${safe_addr4} ${node_addr4}"
log "hopr addr5: ${addr5} ${safe_addr5} ${node_addr5}"
log "hopr addr6: ${addr6} ${safe_addr6} ${node_addr6}"
log "hopr addr7: ${addr7} ${safe_addr7} ${node_addr7}"

declare safe_addrs_to_register="$safe_addr1,$safe_addr2,$safe_addr3,$safe_addr4,$safe_addr5,$safe_addr6"
declare node_addrs_to_register="$node_addr1,$node_addr2,$node_addr3,$node_addr4,$node_addr5,$node_addr6"

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

# this 2 functions are runned at the end of the tests when withdraw transaction should clear on blockchain and we don't have to block and wait for it
check_native_withdraw_results() {
  local initial_native_balance="${1}"

  balances=$(api_get_balances ${api1})
  new_native_balance=$(echo ${balances} | jq -r .native)
  [[ "${initial_native_balance}" == "${new_native_balance}" ]] && { msg "Native withdraw failed, pre: ${initial_native_balance}, post: ${new_native_balance}"; exit 1; }

  echo "withdraw native successful"
}

test_aliases() {
  local node_api="${1}"
  local peer_id="${2}"

  aliases=$(api_get_aliases ${node_api} "\"me\"")
  api_set_alias ${node_api} ${peer_id} "Alice"
  aliases=$(api_get_aliases ${node_api} "\"Alice\"")
  api_get_alias ${node_api} "Alice" "${peer_id}"
  api_remove_alias ${node_api} "Alice"
  aliases=$(api_get_aliases ${node_api} "\"me\"")
  [[ "${aliases}" == *"Alice"* ]] && { msg "Alias removal failed: ${aliases}"; exit 1; }
  echo ${aliases}
}

test_aliases ${api1} ${addr2}

for node in ${addr2} ${addr3} ${addr4} ${addr5}; do
  log "Node 1 ping other node ${node}"
  result=$(api_ping "${api1}" ${node} "\"latency\":[0-9]+,\"reportedVersion\":")
  log "-- ${result}"
done

log "Node 2 ping node 3"
result=$(api_ping "${api2}" ${addr3} "\"latency\":[0-9]+,\"reportedVersion\":")
log "-- ${result}"

# FIXME: re-enable when network check works
# log "Node 1 should not be able to talk to Node 6 (different network id)"
# result=$(api_ping "${api6}" ${addr1} "TIMEOUT")
# log "-- ${result}"

# FIXME: re-enable when network check works
# log "Node 6 should not be able to talk to Node 1 (different network id)"
# result=$(api_ping "${api6}" ${addr1} "TIMEOUT")
# log "-- ${result}"

# log "Node 7 should not be able to talk to Node 1 (Node 7 is not in the register)"
# result=$(ping "${api7}" ${addr1} "TIMEOUT")
# log "-- ${result}"

# log "Node 1 should not be able to talk to Node 7 (Node 7 is not in the register)"
# result=$(ping "${api1}" ${addr7} "TIMEOUT")
# log "-- ${result}"

log "Node 2 has no unredeemed ticket value"
result=$(api_get_ticket_statistics "${api2}" "\"unredeemedValue\":\"0\"")
log "-- ${result}"

for i in `seq 1 10`; do
  log "Node 1 send 0 hop message to node 2"
  api_send_message "${api1}" "${msg_tag}" "${addr2}" 'hello, world from node 1 via 0-hop' "" & jobs+=( "$!" )

  log "Node 2 send 0 hop message to node 3"
  api_send_message "${api2}" "${msg_tag}" "${addr3}" 'hello, world from node 2 via 0-hop' "" & jobs+=( "$!" )

  log "Node 3 send 0 hop message to node 4"
  api_send_message "${api3}" "${msg_tag}" "${addr4}" 'hello, world from node 3 via 0-hop' "" & jobs+=( "$!" )

  log "Node 4 send 0 hop message to node 5"
  api_send_message "${api4}" "${msg_tag}" "${addr5}" 'hello, world from node 4 via 0-hop' "" & jobs+=( "$!" )
done

# opening channels in parallel
api_open_channel 1 2 "${api1}" "${node_addr2}" & jobs+=( "$!" )
api_open_channel 2 3 "${api2}" "${node_addr3}" & jobs+=( "$!" )
api_open_channel 3 4 "${api3}" "${node_addr4}" & jobs+=( "$!" )
api_open_channel 4 5 "${api4}" "${node_addr5}" & jobs+=( "$!" )
api_open_channel 5 1 "${api5}" "${node_addr1}" & jobs+=( "$!" )
# used for channel close test later
api_open_channel 1 5 "${api1}" "${node_addr5}" & jobs+=( "$!" )

# opening temporary channel just to test get all channels later on
api_open_channel 1 4 "${api1}" "${node_addr4}" & jobs+=( "$!" )

log "Waiting for nodes to finish open channel (long running)"
for j in ${jobs[@]}; do wait -n $j; done; jobs=()
log "Waiting DONE"

echo "Exit early after opening channels, because some 1-hop messages are not fully working yet"
exit 0

for i in `seq 1 10`; do
  log "Node 1 send 1 hop message to self via node 2"
  api_send_message "${api1}" "${msg_tag}" "${addr1}" 'hello, world from self via 2' "${addr2}" & jobs+=( "$!" )

  log "Node 2 send 1 hop message to self via node 3"
  api_send_message "${api2}" "${msg_tag}" "${addr2}" 'hello, world from self via 3' "${addr3}" & jobs+=( "$!" )

  log "Node 3 send 1 hop message to self via node 4"
  api_send_message "${api3}" "${msg_tag}" "${addr3}" 'hello, world from self via 4' "${addr4}" & jobs+=( "$!" )

  log "Node 4 send 1 hop message to self via node 5"
  api_send_message "${api4}" "${msg_tag}" "${addr4}" 'hello, world from self via 5' "${addr5}" & jobs+=( "$!" )
done

log "Waiting for nodes to finish sending 1 hop messages"
for j in ${jobs[@]}; do wait -n $j; done; jobs=()
log "Waiting DONE"

log "Node 2 should now have a ticket"
result=$(api_get_ticket_statistics "${api2}" "\"winProportion\":1")
log "-- ${result}"

log "Node 3 should now have a ticket"
result=$(api_get_ticket_statistics "${api3}" "\"winProportion\":1")
log "-- ${result}"

log "Node 4 should now have a ticket"
result=$(api_get_ticket_statistics "${api4}" "\"winProportion\":1")
log "-- ${result}"

log "Node 5 should now have a ticket"
result=$(api_get_ticket_statistics "${api5}" "\"winProportion\":1")
log "-- ${result}"

for i in `seq 1 10`; do
  log "Node 1 send 1 hop message to node 3 via node 2"
  api_send_message "${api1}" "${msg_tag}" "${addr3}" 'hello, world from 1 via 2' "${addr2}" & jobs+=( "$!" )

  log "Node 2 send 1 hop message to node 4 via node 3"
  api_send_message "${api2}" "${msg_tag}" "${addr4}" 'hello, world from 2 via 3' "${addr3}" & jobs+=( "$!" )

  log "Node 3 send 1 hop message to node 5 via node 4"
  api_send_message "${api3}" "${msg_tag}" "${addr5}" 'hello, world from 3 via 4' "${addr4}" & jobs+=( "$!" )

  log "Node 5 send 1 hop message to node 2 via node 1"
  api_send_message "${api5}" "${msg_tag}" "${addr2}" 'hello, world from 5 via 1' "${addr1}" & jobs+=( "$!" )
done
log "Waiting for nodes to finish sending 1-hop messages"
for j in ${jobs[@]}; do wait -n $j; done; jobs=()
log "Waiting DONE"

for i in `seq 1 10`; do
  log "Node 1 send 3 hop message to node 5 via node 2, node 3 and node 4"
  api_send_message "${api1}" "${msg_tag}" "${addr5}" "hello, world from 1 via 2,3,4" "${addr2} ${addr3} ${addr4}" & jobs+=( "$!" )
done
log "Waiting for nodes to finish sending 3-hop messages"
for j in ${jobs[@]}; do wait -n $j; done; jobs=()
log "Waiting DONE"

for i in `seq 1 10`; do
  log "Node 1 send message to node 5"
  api_send_message "${api1}" "${msg_tag}" "${addr5}" "hello, world from 1 via auto" "" & jobs+=( "$!" )
done
log "Waiting for node 1 to send messages to node 5"
for j in ${jobs[@]}; do wait -n $j; done; jobs=()
log "Waiting DONE"

test_redeem_in_specific_channel() {
  local node_id="${1}"
  local second_node_id="${2}"
  local node_api="${3}"
  local second_node_api="${4}"

  peer_id=$(get_hopr_address ${api_token}@${node_api})
  second_node_addr=$(get_native_address ${api_token}@${second_node_api})

  api_open_channel "${node_id}" "${second_node_id}" "${node_api}" "${second_node_addr}"

  for i in `seq 1 3`; do
    log "Node ${node_id} send 1 hop message to self via node ${second_node_id}"
    api_send_message "${node_api}" "${msg_tag}" "${peer_id}" "hello, world 1 self" "${second_peer_id}"
  done

  # seems like there's slight delay needed for tickets endpoint to return up to date tickets, probably because of blockchain sync delay
  sleep 2
  ticket_amount=$(api_get_tickets_in_channel ${second_node_api} ${peer_id} | jq '. | length')
  [[ "${ticket_amount}" != "3" ]] && { msg "Ticket amount is different than expected: ${ticket_amount} != 3"; exit 1; }

  api_redeem_tickets_in_channel ${second_node_api} ${peer_id}

  api_get_tickets_in_channel ${second_node_api} ${peer_id} "TICKETS_NOT_FOUND"

  api_close_channel "${node_id}" "${second_node_id}" "${node_api}" "${second_peer_id}" "outgoing"
  echo "all good"
}

# FIXME: re-enable when ticket redemption works
#
# test_redeem_in_specific_channel "1" "3" ${api1} ${api3} & jobs+=( "$!" )

# redeem_tickets "2" "${api2}" & jobs+=( "$!" )
# redeem_tickets "3" "${api2}" & jobs+=( "$!" )
# redeem_tickets "4" "${api2}" & jobs+=( "$!" )
# redeem_tickets "5" "${api2}" & jobs+=( "$!" )

#log "Waiting for nodes to finish ticket redemption (long running)"
#for j in ${jobs[@]}; do wait -n $j; done; jobs=()
#log "Waiting DONE"

# initiate channel closures, but don't wait because this will trigger ticket
# redemption as well
api_close_channel 1 4 "${api1}" "${node_addr4}" "outgoing" & jobs+=( "$!" )
api_close_channel 1 2 "${api1}" "${node_addr2}" "outgoing" & jobs+=( "$!" )
api_close_channel 2 3 "${api2}" "${node_addr3}" "outgoing" & jobs+=( "$!" )
api_close_channel 3 4 "${api3}" "${node_addr4}" "outgoing" & jobs+=( "$!" )
api_close_channel 4 5 "${api4}" "${node_addr5}" "outgoing" & jobs+=( "$!" )
api_close_channel 5 1 "${api5}" "${node_addr1}" "outgoing" & jobs+=( "$!" )

# initiate channel closures for channels without tickets so we can check
# completeness
api_close_channel 1 5 "${api1}" "${addr5}" "outgoing" "true" & jobs+=( "$!" )

log "Waiting for nodes to finish handling close channels calls"
for j in ${jobs[@]}; do wait -n $j; done; jobs=()
log "Waiting DONE"

test_get_all_channels() {
  local node_api=${1}

  channels=$(api_get_all_channels ${node_api} false)
  channels_count=$(echo ${channels} | jq '.incoming | length')

  channels_with_closed=$(api_get_all_channels ${node_api} true)
  channels_with_closed_count=$(echo ${channels_with_closed} | jq '.incoming | length')

  [[ "${channels_count}" -ge "${channels_with_closed_count}" ]] && { msg "There should be more channels returned with includeClosed flag: ${channels_count} !< ${channels_with_closed_count}"; exit 1; }
  [[ "${channels_with_closed}" != *"Closed"* ]] && { msg "Channels fetched with includeClosed flag should return channels with closed status: ${channels_with_closed}"; exit 1; }
  echo "Get all channels successful"
}

test_get_all_channels "${api1}"

# NOTE: strategy testing will require separate setup so commented out for now until moved
# test_strategy_setting() {
#   local node_api="${1}"

#   settings=$(get_settings ${node_api})
#   strategy=$(echo ${settings} | jq -r .strategy)
#   [[ "${strategy}" != "passive" ]] && { msg "Default strategy should be passive, got: ${strategy}"; exit 1; }

#   channels_count_pre=$(get_all_channels ${node_api} false | jq '.incoming | length')

#   set_setting ${node_api} "strategy" "promiscuous"

#   log "Waiting 100 seconds for the node to make connections to other nodes"
#   sleep 100

#   channels_count_post=$(get_all_channels ${node_api} false | jq '.incoming | length')
#   [[ "${channels_count_pre}" -ge "${channels_count_post}" ]] && { msg "Node didn't open any connections by itself even when strategy was set to promiscuous: ${channels_count_pre} !>= ${channels_count_post}"; exit 1; }
#   echo "Strategy setting successfull"
# }

# test_strategy_setting ${api4}


# checking statuses of the long running tests
check_native_withdraw_results ${native_balance}
