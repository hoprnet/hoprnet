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
source "${mydir}/../api.sh"

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

log "Using endpoints: ${endpoints}"

for endpoint in ${endpoints}; do
  log "Validate native address for ${endpoint}"
  declare address="$(validate_native_address "${endpoint}" "${api_token}")"
  log "Validate native address for ${endpoint} - OK ${address}"
done

for endpoint in ${endpoints}; do
  log "Validate funds for ${endpoint}"
  api_validate_node_balance_gt0 "${endpoint}"
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
result=$(api_peers "${endpoints_arr[1]}")
log "-- ${result}"

for endpoint in ${endpoints}; do
  for other_endpoint in ${endpoints}; do
    # only perform operation if endpoints differ
    if [ "${endpoint}" != "${other_endpoint}" ]; then
      log "${endpoint} ping other node at ${other_endpoint}"
      result=$(api_ping "${endpoint}" "${peers["${other_endpoint}"]}" "\"latency\":")
      log "-- ${result}"
    fi
  done
done

log "Opening channels in background to parallelize operations"

# put 0.5 HOPR token into each channel
declare amount_per_channel="500000000000000000"

for endpoint in ${endpoints}; do
  for other_endpoint in ${endpoints}; do
    # only perform operation if endpoints differ
    if [ "${endpoint}" != "${other_endpoint}" ]; then
      log "${endpoint} opening channel to other node at ${other_endpoint}"
      declare src="${peers["${endpoint}"]}"
      declare dst="${peers["${other_endpoint}"]}"
      api_open_channel "${src}" "${dst}" "${endpoint}" "${dst}" "${amount_per_channel}" &
    fi
  done
done

log "Wait for all channel operations to finish"
wait
