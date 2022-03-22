#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="e2e-test"
source "${mydir}/../scripts/utils.sh"

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

# $1 = endpoint
# $2 = recipient peer id
# $3 = message
# $4 = OPTIONAL: peers in the message path
# $5 = OPTIONAL: maximum wait time in seconds during which we busy try
# afterwards we fail, defaults to 0
# $6 = OPTIONAL: step time between retries in seconds, defaults to 25 seconds
# (8 blocks with 1-3 s/block in ganache)
# $7 = OPTIONAL: end time for busy wait in nanoseconds since epoch, has higher
# priority than wait time, defaults to 0
send_message(){
  local result now
  local endpoint="${1}"
  local recipient="${2}"
  local msg="${3}"
  local peers="${4}"
  local wait_time=${5:-0}
  local step_time=${6:-25}
  local end_time_ns=${7:-0}
  # no timeout set since the test execution environment should cancel the test if it takes too long
  local cmd="curl -m ${step_time} --connect-timeout ${step_time} -s -H X-Auth-Token:${api_token} -H Content-Type:application/json --url ${endpoint}/api/v2/messages -o /dev/null -w %{http_code} -d "

  # if no end time was given we need to calculate it once
  if [ ${end_time_ns} -eq 0 ]; then
    now=$(node -e "console.log(process.hrtime.bigint().toString());")
    # need to calculate in nanoseconds
    ((end_time_ns=now+wait_time*1000000000))
  fi

  local path=$(echo ${peers} | tr -d '\n' | jq -R -s 'split(" ")')
  local message='{"body":"'${msg}'","path":'${path}',"recipient":"'${recipient}'"}'
  result=$(${cmd} "${message}")

  # we fail if the HTTP status code is anything but 204
  if [ "${result}" = "204" ]; then
    echo "${result}"
  else
    now=$(node -e "console.log(process.hrtime.bigint().toString());")
    if [ ${end_time_ns} -lt ${now} ]; then
      log "${RED}send_message (${cmd} \"${message}\") FAILED, received: ${result}${NOFORMAT}"
      exit 1
    else
      log "${YELLOW}send_message (${cmd} \"${message}\") FAILED, retrying in ${step_time} seconds${NOFORMAT}"
      sleep ${step_time}
      send_message "${endpoint}" "${recipient}" "${msg}" "${peers}" "${wait_time}" "${step_time}" "${end_time_ns}"
    fi
  fi
}

# $1 = endpoint
# $2 = Hopr command
# $3 = OPTIONAL: positive assertion message
# $4 = OPTIONAL: maximum wait time in seconds during which we busy try
# afterwards we fail, defaults to 0
# $4 = OPTIONAL: step time between retries in seconds, defaults to 25 seconds
# (8 blocks with 1-3 s/block in ganache)
# $5 = OPTIONAL: end time for busy wait in nanoseconds since epoch, has higher
# priority than wait time, defaults to 0
run_command(){
  local result now
  local endpoint="${1}"
  local hopr_cmd="${2}"
  local assertion="${3:-}"
  local wait_time=${4:-0}
  local step_time=${5:-25}
  local end_time_ns=${6:-0}
  # no timeout set since the test execution environment should cancel the test if it takes too long
  local cmd="curl -m ${step_time} --connect-timeout ${step_time} -s -H X-Auth-Token:${api_token} --url ${endpoint}/api/v1/command -d "

  # if no end time was given we need to calculate it once
  if [ ${end_time_ns} -eq 0 ]; then
    now=$(node -e "console.log(process.hrtime.bigint().toString());")
    # need to calculate in nanoseconds
    ((end_time_ns=now+wait_time*1000000000))
  fi

  result=$(${cmd} "${hopr_cmd}")

  # if an assertion was given and has not been fulfilled, we fail
  if [ -z "${assertion}" ] || [[ -n "${assertion}" && "${result}" == *"${assertion}"* ]]; then
    echo "${result}"
  else
    now=$(node -e "console.log(process.hrtime.bigint().toString());")
    if [ ${end_time_ns} -lt ${now} ]; then
      log "${RED}run_command (${cmd} \"${hopr_cmd}\") FAILED, received: ${result}${NOFORMAT}"
      exit 1
    else
      log "${YELLOW}run_command (${cmd} \"${hopr_cmd}\") FAILED, retrying in ${step_time} seconds${NOFORMAT}"
      sleep ${step_time}
      run_command "${endpoint}" "${hopr_cmd}" "${assertion}" "${wait_time}" \
        "${step_time}" "${end_time_ns}"
    fi
  fi
}

# TODO better validation
validate_node_balance_gt0() {
  local balance eth_balance hopr_balance

  balance="$(run_command ${1} "balance" "Balance" 600)"
  eth_balance="$(echo -e "$balance" | grep -c " xDAI" || true)"
  hopr_balance="$(echo -e "$balance" | grep -c " txHOPR" || true)"

  if [[ "$eth_balance" != "0" && "$hopr_balance" != "Hopr Balance: 0 txHOPR" ]]; then
    log "$1 is funded"
  else
    log "⛔️ $1 Node has an invalid balance: $eth_balance, $hopr_balance"
    log "$balance"
    exit 1
  fi
}

# $1 = source node id
# $2 = destination node id
# $3 = channel source api endpoint
# $4 = channel destination peer id
# $5 = OPTIONAL: verify closure strictly
close_channel() {
  local source_id="${1}"
  local destination_id="${2}"
  local source_api="${3}"
  local destination_peer_id="${4}"
  local strict_check="${5:-false}"
  local result

  log "Node ${source_id} close channel to Node ${destination_id}"
  if [ "${strict_check}" = "true" ]; then
    result=$(run_command "${source_api}" "close ${destination_peer_id}" "Channel is already closed" 600)
  else
    result=$(run_command "${source_api}" "close ${destination_peer_id}" "" 20 20)
  fi
  log "Node ${source_id} close channel to Node ${destination_id} result -- ${result}"
}

# $1 = source node id
# $2 = destination node id
# $3 = channel source api endpoint
# $4 = channel destination peer id
open_channel() {
  local source_id="${1}"
  local destination_id="${2}"
  local source_api="${3}"
  local destination_peer_id="${4}"
  local result

  log "Node ${source_id} open channel to Node ${destination_id}"
  result=$(run_command "${source_api}" "open ${destination_peer_id} 2" "Successfully opened channel" 600 60)
  log "Node ${source_id} open channel to Node ${destination_id} result -- ${result}"
}


# $1 = node id
# $2 = node api endpoint
redeem_tickets() {
  local node_id="${1}"
  local node_api="${2}"
  local rejected redeemed prev_redeemed

  # First get the inital ticket statistics for reference
  result=$(run_command "${node_api}" "tickets" "" 600)
  log "Node ${node_id} ticket information (before redemption) -- ${result}"
  rejected=$(echo "${result}" | grep "Rejected:" | awk '{ print $3; }' | tr -d '\n')
  redeemed=$(echo "${result}" | grep "Redeemed:" | awk '{ print $3; }' | tr -d '\n')
  [[ ${rejected} -gt 0 ]] && { msg "rejected tickets count on node ${node_id} is ${rejected}"; exit 1; }
  last_redeemed="${redeemed}"

  # Trigger a redemption run, but cap it at 1 minute. We only want to measure
  # progress, not redeeem all tickets which takes too long.
  log "Node ${node_id} should redeem all tickets"
  result=$(run_command "${node_api}" "redeemTickets" "" 60 60)
  log "--${result}"

  # Get ticket statistics again and compare with previous state. Ensure we
  # redeemed tickets.
  result=$(run_command "${node_api}" "tickets" "" 600)
  log "Node ${node_id} ticket information (after redemption) -- ${result}"
  rejected=$(echo "${result}" | grep "Rejected:" | awk '{ print $3; }' | tr -d '\n')
  redeemed=$(echo "${result}" | grep "Redeemed:" | awk '{ print $3; }' | tr -d '\n')
  [[ ${rejected} -gt 0 ]] && { msg "rejected tickets count on node ${node_id} is ${rejected}"; exit 1; }
  [[ ${redeemed} -gt 0 && ${redeemed} -gt ${last_redeemed} ]] || { msg "redeemed tickets count on node ${node_id} is ${redeemed}, previously ${last_redeemed}"; exit 1; }
  last_redeemed="${redeemed}"

  # Trigger another redemption run, but cap it at 1 minute. We only want to measure
  # progress, not redeeem all tickets which takes too long.
  log "Node ${node_id} should redeem all tickets (again to ensure re-run of operation)"
  result=$(run_command "${node_api}" "redeemTickets" "" 60 60)
  log "--${result}"

  # Get final ticket statistics
  result=$(run_command "${node_api}" "tickets" "" 600)
  log "Node ${node_id} ticket information (after second redemption) -- ${result}"
  rejected=$(echo "${result}" | grep "Rejected:" | awk '{ print $3; }' | tr -d '\n')
  redeemed=$(echo "${result}" | grep "Redeemed:" | awk '{ print $3; }' | tr -d '\n')
  [[ ${rejected} -gt 0 ]] && { msg "rejected tickets count on node ${node_id} is ${rejected}"; exit 1; }
  [[ ${redeemed} -gt 0 && ${redeemed} -gt ${last_redeemed} ]] || { msg "redeemed tickets count on node ${node_id} is ${redeemed}, previously ${last_redeemed}"; exit 1; }
}


log "Running full E2E test with ${api1}, ${api2}, ${api3}, ${api4}, ${api5}, ${api6}, ${api7}"

validate_native_address "${api1}" "${api_token}"
validate_native_address "${api2}" "${api_token}"
validate_native_address "${api3}" "${api_token}"
validate_native_address "${api4}" "${api_token}"
validate_native_address "${api5}" "${api_token}"
# we don't need node6 because it's short-living
validate_native_address "${api7}" "${api_token}"
log "ETH addresses exist"

validate_node_balance_gt0 "${api1}"
validate_node_balance_gt0 "${api2}"
validate_node_balance_gt0 "${api3}"
validate_node_balance_gt0 "${api4}"
validate_node_balance_gt0 "${api5}"
# we don't need node6 because it's short-living
validate_node_balance_gt0 "${api7}"
log "Nodes are funded"

declare addr1 addr2 addr3 addr4 addr5 result
addr1="$(get_hopr_address "${api_token}@${api1}")"
addr2="$(get_hopr_address "${api_token}@${api2}")"
addr3="$(get_hopr_address "${api_token}@${api3}")"
addr4="$(get_hopr_address "${api_token}@${api4}")"
addr5="$(get_hopr_address "${api_token}@${api5}")"
# we don't need node6 because it's short-living
addr7="$(get_hopr_address "${api_token}@${api7}")"

log "hopr addr1: ${addr1}"
log "hopr addr2: ${addr2}"
log "hopr addr3: ${addr3}"
log "hopr addr4: ${addr4}"
log "hopr addr5: ${addr5}"
# we don't need node6 because it's short-living
log "hopr addr7: ${addr7}"

log "Check peers"
result=$(run_command ${api1} "peers" 'peers have announced themselves' 600)
log "-- ${result}"

for node in ${addr2} ${addr3} ${addr4} ${addr5}; do
  log "Node 1 ping other node ${node}"
  result=$(run_command ${api1} "ping ${node}" "Pong received in:" 600)
  log "-- ${result}"
done

log "Node 2 ping node 3"
result=$(run_command ${api2} "ping ${addr3}" "Pong received in:" 600)
log "-- ${result}"

log "Node 7 should not be able to talk to Node 1 (different environment id)"
result=$(run_command ${api7} "ping ${addr1}" "Could not ping node. Timeout." 600)
log "-- ${result}"

log "Node 1 should not be able to talk to Node 7 (different environment id)"
result=$(run_command ${api1} "ping ${addr7}" "Could not ping node. Timeout." 600)
log "-- ${result}"

log "Node 2 has no unredeemed ticket value"
result=$(run_command ${api2} "tickets" "Unredeemed Value: 0 txHOPR" 600)
log "-- ${result}"

log "Node 1 send 0-hop message to node 2"
run_command "${api1}" "send ,${addr2} 'hello, world'" "Message sent" 600

# opening channels in parallel
open_channel 1 2 "${api1}" "${addr2}" &
open_channel 2 3 "${api2}" "${addr3}" &
open_channel 3 4 "${api3}" "${addr4}" &
open_channel 4 5 "${api4}" "${addr5}" &
open_channel 5 1 "${api5}" "${addr1}" &

#used for channel close test later
open_channel 1 5 "${api1}" "${addr5}" &

log "Waiting for nodes to finish open channel (long running)"
wait

for i in `seq 1 10`; do
  log "Node 1 send 1 hop message to self via node 2"
  run_command "${api1}" "send ${addr2},${addr1} 'hello, world'" "Message sent" 600

  log "Node 2 send 1 hop message to self via node 3"
  run_command "${api2}" "send ${addr3},${addr2} 'hello, world'" "Message sent" 600

  log "Node 3 send 1 hop message to self via node 4"
  run_command "${api3}" "send ${addr4},${addr3} 'hello, world'" "Message sent" 600

  log "Node 4 send 1 hop message to self via node 5"
  run_command "${api4}" "send ${addr5},${addr4} 'hello, world'" "Message sent" 600
done

log "Node 2 should now have a ticket"
result=$(run_command ${api2} "tickets" "Win Proportion:   100%" 600)
log "-- ${result}"

log "Node 3 should now have a ticket"
result=$(run_command ${api3} "tickets" "Win Proportion:   100%" 600)
log "-- ${result}"

log "Node 4 should now have a ticket"
result=$(run_command ${api4} "tickets" "Win Proportion:   100%" 600)
log "-- ${result}"

log "Node 5 should now have a ticket"
result=$(run_command ${api5} "tickets" "Win Proportion:   100%" 600)
log "-- ${result}"

for i in `seq 1 10`; do
  log "Node 1 send 1 hop message to node 3 via node 2"
  run_command "${api1}" "send ${addr2},${addr3} 'hello, world'" "Message sent" 600

  log "Node 2 send 1 hop message to node 4 via node 3"
  run_command "${api2}" "send ${addr3},${addr4} 'hello, world'" "Message sent" 600

  log "Node 3 send 1 hop message to node 5 via node 4"
  run_command "${api3}" "send ${addr4},${addr5} 'hello, world'" "Message sent" 600

  log "Node 5 send 1 hop message to node 2 via node 1"
  run_command "${api5}" "send ${addr1},${addr2} 'hello, world'" "Message sent" 600
done

# for the last send tests we use Rest API v2 instead of the older command-based Rest API v1

for i in `seq 1 10`; do
  log "Node 1 send 3 hop message to node 5 via node 2, node 3 and node 4"
  send_message "${api1}" "${addr5}" "hello, world" "${addr2} ${addr3} ${addr4}" 600
done

for i in `seq 1 10`; do
  log "Node 1 send message to node 5"
  send_message "${api1}" "${addr5}" "hello, world" "" 600
done

redeem_tickets "2" "${api2}" &
redeem_tickets "3" "${api2}" &
redeem_tickets "4" "${api2}" &
redeem_tickets "5" "${api2}" &

log "Waiting for nodes to finish ticket redemption (long running)"
wait

# initiate channel closures, but don't wait because this will trigger ticket
# redemption as well
close_channel 1 2 "${api1}" "${addr2}" &
close_channel 2 3 "${api2}" "${addr3}" &
close_channel 3 4 "${api3}" "${addr4}" &
close_channel 4 5 "${api4}" "${addr5}" &
close_channel 5 1 "${api5}" "${addr1}" &

# initiate channel closures for channels without tickets so we can check
# completeness
close_channel 1 5 "${api1}" "${addr5}" &

log "Waiting for nodes to finish handling close channels calls"
wait

# Also add confirmation time
log "Waiting 70 seconds for cool-off period"
sleep 70

# verify channel has been closed
close_channel 1 5 "${api1}" "${addr5}" "true"
