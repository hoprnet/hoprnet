#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="full-interconnected_cluster"
source "${mydir}/../utils.sh"

usage() {
  msg
  msg "Usage: $0 <node_api_1> <node_api_2> <node_api_3> <node_api_4> <node_api_5>"
  msg
  msg "Required environment variables"
  msg "------------------------------"
  msg
  msg "HOPRD_API_TOKEN\t\t\tused as api token for all nodes"
  msg
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
test -z "${1:-}" && { msg "Missing <node_api_1>"; usage; exit 1; }
test -z "${2:-}" && { msg "Missing <node_api_2>"; usage; exit 1; }
test -z "${3:-}" && { msg "Missing <node_api_3>"; usage; exit 1; }
test -z "${4:-}" && { msg "Missing <node_api_4>"; usage; exit 1; }
test -z "${5:-}" && { msg "Missing <node_api_5>"; usage; exit 1; }
test -z "${HOPRD_API_TOKEN:-}" && { msg "Missing HOPRD_API_TOKEN"; usage; exit 1; }

declare api1="${1}"
declare api2="${2}"
declare api3="${3}"
declare api4="${4}"
declare api5="${5}"
declare api_token=${HOPRD_API_TOKEN}

# $1 = endpoint
# $2 = Hopr command
# $3 = OPTIONAL: positive assertion message
# $4 = OPTIONAL: maximum wait time in seconds during which we busy try
# afterwards we fail, defaults to 0
# $4 = OPTIONAL: step time between retries in seconds, defaults to 5 seconds
# $5 = OPTIONAL: end time for busy wait in nanoseconds since epoch, has higher
# priority than wait time, defaults to 0
run_command(){
  local result now
  local endpoint="${1}"
  local hopr_cmd="${2}"
  local assertion="${3:-}"
  local wait_time=${4:-0}
  local step_time=${5:-5}
  local end_time_ns=${6:-0}
  # no timeout set since the test execution environment should cancel the test if it takes too long
  local cmd="curl --silent -X POST --header X-Auth-Token:${api_token} --url ${endpoint}/api/v1/command --data "

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

get_eth_address(){
  curl --silent "$1/api/v1/address/eth"
}

get_hopr_address(){
  curl --silent "$1/api/v1/address/hopr"
}

validate_node_eth_address() {
  local ETH_ADDRESS IS_VALID_ETH_ADDRESS

  ETH_ADDRESS="$(get_eth_address $1)"
  if [ -z "$ETH_ADDRESS" ]; then
    log "could not derive ETH_ADDRESS from first parameter $1"
    exit 1
  fi

  IS_VALID_ETH_ADDRESS="$(curl --silent https://api.hoprnet.org/api/validate/$ETH_ADDRESS/get?text=true)"
  if [ "$IS_VALID_ETH_ADDRESS" == "false" ]; then
    log "⛔️ Node returns an invalid address ETH_ADDRESS: $ETH_ADDRESS derived from $1"
    exit 1
  fi
  echo "$ETH_ADDRESS"
}

# TODO better validation
validate_node_balance_gt0() {
  local balance eth_balance hopr_balance

  balance="$(run_command ${1} "balance")"
  eth_balance="$(echo -e "$balance" | grep -c " xDAI" || true)"
  hopr_balance="$(echo -e "$balance" | grep -c " HOPR" || true)"

  if [[ "$eth_balance" != "0" && "$hopr_balance" != "Hopr Balance: 0 HOPR" ]]; then
    log "$1 is funded"
  else
    log "⛔️ $1 Node has an invalid balance: $eth_balance, $hopr_balance"
    log "$balance"
    exit 1
  fi
}

log "Running full E2E test with ${api1}, ${api2}, ${api3}, ${api4}, ${api5}"

validate_node_eth_address "${api1}"
validate_node_eth_address "${api2}"
validate_node_eth_address "${api3}"
validate_node_eth_address "${api4}"
validate_node_eth_address "${api5}"
log "ETH addresses exist"

validate_node_balance_gt0 "${api1}"
validate_node_balance_gt0 "${api2}"
validate_node_balance_gt0 "${api3}"
validate_node_balance_gt0 "${api4}"
validate_node_balance_gt0 "${api5}"
log "Nodes are funded"

declare addr1 addr2 addr3 addr4 addr5 result
addr1="$(get_hopr_address "${api1}")"
addr2="$(get_hopr_address "${api2}")"
addr3="$(get_hopr_address "${api3}")"
addr4="$(get_hopr_address "${api4}")"
addr5="$(get_hopr_address "${api5}")"
log "hopr addr1: ${addr1}"
log "hopr addr2: ${addr2}"
log "hopr addr3: ${addr3}"
log "hopr addr4: ${addr4}"
log "hopr addr5: ${addr5}"

log "Check peers"
result=$(run_command ${api1} "peers" 'peers have announced themselves' 600)
log "-- ${result}"

for node in ${addr2} ${addr3} ${addr4} ${addr5}; do
  log "Node 1 ping other node ${node}"
  result=$(run_command ${api1} "ping ${node}" "Pong received in:" 600)
  log "-- ${result}"
done

log "Opening channels in background to parallelize operations"

for addr in ${addr2} ${addr3} ${addr4} ${addr5}; do
  log "Node ${addr1} open channel to node ${addr}"
  run_command "${api1}" "open ${addr} 0.1" "Successfully opened channel" 600 &
done

for addr in ${addr1} ${addr3} ${addr4} ${addr5}; do
  log "Node ${addr2} open channel to node ${addr}"
  run_command "${api2}" "open ${addr} 0.1" "Successfully opened channel" 600 &
done

for addr in ${addr1} ${addr2} ${addr4} ${addr5}; do
  log "Node ${addr3} open channel to node ${addr}"
  run_command "${api3}" "open ${addr} 0.1" "Successfully opened channel" 600 &
done

for addr in ${addr1} ${addr2} ${addr3} ${addr5}; do
  log "Node ${addr4} open channel to node ${addr}"
  run_command "${api4}" "open ${addr} 0.1" "Successfully opened channel" 600 &
done

for addr in ${addr1} ${addr2} ${addr3} ${addr4}; do
  log "Node ${addr5} open channel to node ${addr}"
  run_command "${api5}" "open ${addr} 0.1" "Successfully opened channel" 600 &
done

log "Wait for all operations to finish"
wait

log "finished"
