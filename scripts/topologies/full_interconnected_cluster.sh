#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
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
  msg "Usage: $0 [<node_api_endpoint>...]"
  msg
  msg "Any number of endpoints can be provided as paramters. All endpoints will establish HOPR channels to all other endpoints."
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
test -z "${HOPRD_API_TOKEN:-}" && { msg "Missing HOPRD_API_TOKEN"; usage; exit 1; }

declare endpoints="$@"
declare api_token=${HOPRD_API_TOKEN}

# $1 = endpoint
# $2 = Hopr command
# $3 = OPTIONAL: positive assertion message
# $4 = OPTIONAL: maximum wait time in seconds during which we busy try
# afterwards we fail, defaults to 0
# $4 = OPTIONAL: step time between retries in seconds, defaults to 40 seconds (xDAI 5s/block)
# $5 = OPTIONAL: end time for busy wait in nanoseconds since epoch, has higher
# priority than wait time, defaults to 0
run_command() {
  local result now
  local endpoint="${1}"
  local hopr_cmd="${2}"
  local assertion="${3:-}"
  local wait_time=${4:-0}
  local step_time=${5:-40}
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

# TODO better validation
# $1 = endpoint
validate_node_balance_gt0() {
  local balance native_balance hopr_balance

  balance="$(run_command ${1} "balance")"
  native_balance="$(echo -e "$balance" | grep -c " xDAI" || true)"
  hopr_balance="$(echo -e "$balance" | grep -c " txHOPR" || true)"

  if [[ "$native_balance" = "0" || "$hopr_balance" = "Hopr Balance: 0 txHOPR" ]]; then
    log "-- $1 Node has an invalid balance: $native_balance, $hopr_balance"
    log "-- $balance"
    exit 1
  fi
}

log "Using endpoints: ${endpoints}"

for endpoint in ${endpoints}; do
  log "Validate native address for ${endpoint}"
  declare address="$(validate_native_address "${endpoint}" "${api_token}")"
  log "Validate native address for ${endpoint} - OK ${address}"
done

for endpoint in ${endpoints}; do
  log "Validate funds for ${endpoint}"
  validate_node_balance_gt0 "${endpoint}"
  log "Validate funds for ${endpoint} - OK"
done

declare -A peers
for endpoint in ${endpoints}; do
  log "Get peer id for ${endpoint}"
  declare peer="$(get_hopr_address "${api_token}@${endpoint}")"
  peers["${endpoint}"]="${peer}"
  log "Get peer id for ${endpoint} - OK ${peer}"
done

declare endpoints_arr=( ${endpoints} )
log "Check peers announcements"
result=$(run_command ${endpoints_arr[1]} "peers" 'peers have announced themselves' 600)
log "-- ${result}"

for endpoint in ${endpoints}; do
  for other_endpoint in ${endpoints}; do
    # only perform operation if endpoints differ
    if [ "${endpoint}" != "${other_endpoint}" ]; then
      log "${endpoint} ping other node at ${other_endpoint}"
      result=$(run_command ${endpoint} "ping ${peers["${other_endpoint}"]}" "Pong received in:" 600)
      log "-- ${result}"
    fi
  done
done

log "Opening channels in background to parallelize operations"

# use for loop to read all values and indexes
for (( i=0; i<${#endpoints_arr[@]}; i++ )); do
  endpoint=${endpoints_arr[$i]}
  other_endpoint=${endpoints_arr[$i+1]:-""}

  if [ -n "${other_endpoint}" ]; then
    log "${endpoint} opening channel to other node at ${other_endpoint}"
    run_command ${endpoint} "open ${peers["${other_endpoint}"]} 0.1" "Successfully opened channel" 600 &
  fi
done


log "Wait for all channel operations to finish"
wait
