#!/usr/bin/env bash

# prevent execution of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" || { echo "This script should only be sourced." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="api"
# shellcheck disable=SC1090
source "${mydir}/../scripts/utils.sh"

# Call API endpoint and assert response or status code
# $1 = source api url (http://localhost:13001)
# $2 = api_endpoint (/channels)
# $3 = rest method for cURL (GET,POST...)
# $4 = request body as json string
# $5 = OPTIONAL: positive assertion message
# $6 = OPTIONAL: maximum wait time in seconds during which we busy try afterwards we fail, defaults to 0
# $7 = OPTIONAL: step time between retries in seconds, defaults to 25 seconds (8 blocks with 1-3 s/block in ganache)
# $8 = OPTIONAL: end time for busy wait in nanoseconds since epoch, has higher priority than wait time, defaults to 0
# $9 = OPTIONAL: should assert status code
api_call(){
  local result
  local source_api="${1}"
  local api_endpoint="${2}"
  local rest_method="${3}"
  local request_body="${4}"
  local assertion="${5:-}"
  local wait_time=${6:-0}
  local step_time=${7:-25}
  local end_time_ns=${8:-0}
  local should_assert_status_code=${9:-false}

  local response_type

  # no timeout set since the test execution environment should cancel the test if it takes too long
  if [[ "$should_assert_status_code" == true ]]; then
    response_type="-o /dev/null -w %{http_code} -d"
  else
    response_type="-d"
  fi

  local cmd="curl -X ${rest_method} -m ${step_time} --connect-timeout ${step_time} -s -H X-Auth-Token:${api_token} -H Content-Type:application/json --url ${source_api}/api/v3${api_endpoint} ${response_type}"
  # if no end time was given we need to calculate it once

  local now=$(node -e "console.log(process.hrtime.bigint().toString());")

  if [[ ${end_time_ns} -eq 0 ]]; then
    # need to calculate in nanoseconds
    end_time_ns=$((now+wait_time*1000000000))
  fi

  local done=false
  local attempt=0

  while [[ "${done}" == false ]]; do
    result=$(${cmd} "${request_body}")

    # if an assertion was given and has not been fulfilled, we fail
    if [[ -z "${assertion}" ]] || [[ -n $(echo "${result}" | sed -nE "/${assertion}/p") ]]; then
      done=true
    else
      if [[ ${end_time_ns} -lt ${now} ]]; then
        log "${RED}attempt: ${attempt} - api_call (${cmd} \"${request_body}\") FAILED, received: ${result} but expected ${assertion}${NOFORMAT}"
        exit 1
      else
        log "${YELLOW}attempt: ${attempt} - api_call (${cmd} \"${request_body}\") FAILED, received: ${result} but expected ${assertion}, retrying in ${step_time} seconds${NOFORMAT}"
      fi

      sleep "${step_time}"

      now=$(node -e "console.log(process.hrtime.bigint().toString());")
      (( ++attempt ))
    fi
  done

  echo "${result}"
}

# $1 = node api endpoint
# $2 = currency to withdraw
# $3 = amount to withdraw
# $4 = where to send the funds to
api_withdraw() {
  local node_api="${1}"
  local currency="${2}"
  local amount="${3}"
  local ethereum_address="${4}"

  api_call "${node_api}" "/account/withdraw" "POST" "{\"currency\": \"${currency}\", \"amount\": \"${amount}\", \"ethereumAddress\": \"${ethereum_address}\"}" "receipt" 600
}

# $1 = node api endpoint
api_get_balances() {
  local origin=${1}

  api_call "${origin}" "/account/balances" "GET" "" "native" 600
}

# get addresses is in the utils file

# $1 = node api endpoint
# $2 = peerId to alias
# $3 = alias name
api_set_alias() {
  local node_api="${1}"
  local peer_id="${2}"
  local alias="${3}"

  api_call "${node_api}" "/aliases" "POST" "{\"peerId\": \"${peer_id}\", \"alias\": \"${alias}\"}" "" 600
}

# $1 = node api endpoint
# $2 = assertion
api_get_aliases() {
  local node_api="${1}"
  local assertion="${2}"

  api_call "${node_api}" "/aliases" "GET" "" "${assertion}" 600
}

# $1 = node api endpoint
# $2 = assertion
api_get_alias() {
  local node_api="${1}"
  local alias="${2}"
  local assertion="${3}"

  api_call "${node_api}" "/aliases/${alias}" "GET" "" "${assertion}" 600
}

# $1 = node api endpoint
# $2 = alias name to remove
api_remove_alias() {
  local node_api="${1}"
  local alias="${2}"

  api_call "${node_api}" "/aliases/${alias}" "DELETE" "" "" 600
}

# $1 = node api endpoint
# $2 = include closing (true/false)
api_get_all_channels() {
  local node_api="${1}"
  local including_closed=${2}

  api_call "${node_api}" "/channels?includingClosed=${including_closed}" "GET" "" "incoming" 600
}

# $1 = node api endpoint
api_get_settings() {
  local node_api="${1}"

  api_call "${node_api}" "/settings" "GET" "" "includeRecipient" 600
}

# $1 = node api endpoint
# $2 = key of the setting
# $3 = value of the setting
api_set_setting() {
  local node_api="${1}"
  local key="${2}"
  local value="${3}"

  api_call "${node_api}" "/settings/${key}" "PUT" "{\"settingValue\": \"${value}\"}" "" 600
}

# $1 = node api endpoint
# $2 = channel id
# $3 = OPTIONAL: call timeout
api_redeem_tickets_in_channel() {
  local node_api="${1}"
  local channel_id="${2}"
  local timeout="${3:-600}"

  log "redeeming tickets in specific channel, this can take up to 5 minutes depending on the amount of unredeemed tickets in that channel"
  api_call "${node_api}" "/channels/${channel_id}/tickets/redeem" "POST" "" "" "${timeout}" "${timeout}"
}

# $1 = node api endpoint
# $2 = OPTIONAL: call timeout
api_redeem_tickets() {
  local node_api="${1}"
  local timeout="${2:-600}"

  log "redeeming all tickets, this can take up to 5 minutes depending on the amount of unredeemed tickets"
  api_call "${node_api}" "/tickets/redeem" "POST" "" "" "${timeout}" "${timeout}"
}

# $1 = node api endpoint
# $2 = channel id
# $3 = assertion
api_get_tickets_in_channel() {
  local node_api="${1}"
  local channel id="${2}"
  local assertion="${3:-"counterparty"}"

  api_call "${node_api}" "/channels/${channel_id}/tickets" "GET" "" "${assertion}" 600
}

# $1 = node api endpoint
# $2 = counterparty peer id
# $3 = assertion
api_ping() {
  local origin=${1:-localhost:3001}
  local peer_id="${2}"
  local assertion="${3}"

  api_call "${origin}" "/peers/${peer_id}/ping" "POST" "{}" "${assertion}" 600
}

# $1 = node api endpoint
api_peers() {
  local origin=${1:-localhost:3001}

  api_call "${origin}" "/node/peers" "GET" "" "" 600
}

# $1 = node api endpoint
# $2 = assertion
api_get_ticket_statistics() {
  local origin=${1:-localhost:3001}
  local assertion="${2}"

  api_call "${origin}" "/tickets/statistics" "GET" "" "${assertion}" 600
}

# $1 = node api endpoint
# $2 = assertion
api_get_node_info() {
  local origin=${1:-localhost:3001}

  api_call "${origin}" "/node/info" "GET" "" "" 600
}


# $1 = source api url
# $2 = message app tag
# $3 = peer_address peer id
# $4 = message
# $5 = OPTIONAL: peers in the message path
api_send_message(){
  local source_api="${1}"
  local tag="${2}"
  local peer_address="${3}"
  local msg="${4}"
  local peers="${5}"

  local path=$(echo "${peers}" | tr -d '\n' | jq -R -s 'split(" ")')
  local payload='{"body":"'${msg}'","path":'${path}',"peerId":"'${peer_address}'","tag":'${tag}'}'
  # Node might need some time once commitment is set on-chain
  api_call "${source_api}" "/messages" "POST" "${payload}" "202" 90 15 "" true
}

# $1 = source node id
# $2 = destination node id
# $3 = channel source api endpoint
# $4 = destination address
# $5 = direction
# $6 = OPTIONAL: verify closure strictly
api_close_channel() {
  local source_id="${1}"
  local destination_id="${2}"
  local source_api="${3}"
  local destination_address="${4}"
  local direction="${5}"
  local close_check="${6:-false}"
  local result channel_id channels_info source_addr

  # fetch channel id from API
  channels_info="$(api_get_all_channels "${source_api}" false)"
  channel_id="$(echo "${channel_info}" | jq  -r ".${direction}| map(select(.peerAddress | contains("${destination_address}")))[0].id")"

  log "Node ${source_id} close channel ${channel_id} to Node ${destination_id}"

  if [ "${close_check}" = "true" ]; then
    result="$(api_call "${source_api}" "/channels/${channel_id}" "DELETE" "" 'Closed|Channel is already closed' 600)"
  else
    result="$(api_call "${source_api}" "/channels/${channel_id}" "DELETE" "" 'PendingToClose|Closed' 60 20)"
  fi

  log "Node ${source_id} close channel ${channel_id} to Node ${destination_id} result -- ${result}"
}

# $1 = source node id
# $2 = destination node id
# $3 = channel source api endpoint
# $4 = channel destination native address
# $5 = OPTIONAL: amount of tokens to stake (full denomination), default is 100
api_open_channel() {
  local source_id="${1}"
  local destination_id="${2}"
  local source_api="${3}"
  local destination_address="${4}"
  local amount="${5:-100000000000000000000}"
  local result

  log "Node ${source_id} open channel to Node ${destination_id}"
  result=$(api_call "${source_api}" "/channels" "POST" "{ \"peerAddress\": \"${destination_address}\", \"amount\": \"${amount}\" }" 'channelId|CHANNEL_ALREADY_OPEN' 600 30)
  log "Node ${source_id} open channel to Node ${destination_id} result -- ${result}"
}

# $1 = node api address (origin)
# validate that node is funded
api_validate_node_balance_gt0() {
  local balance eth_balance hopr_balance
  local endpoint=${1:-localhost:3001}

  balance=$(api_get_balances "${endpoint}")
  eth_balance=$(echo "${balance}" | jq -r ".native")
  hopr_balance=$(echo "${balance}" | jq -r ".hopr")

  if [[ "$eth_balance" != "0" && "$hopr_balance" != "0" ]]; then
    log "$1 is funded"
  else
    log "⛔️ $1 Node has an invalid balance: $eth_balance, $hopr_balance"
    log "$balance"
    exit 1
  fi
}
