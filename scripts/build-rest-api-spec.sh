#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare -x HOPR_LOG_ID="build-rest-api-spec"
source "${mydir}/utils.sh"

usage() {
  msg
  msg "This script can be used to generate the OpenAPI spec file for hoprd's Rest API v2."
  msg
  msg "Usage: $0"
  msg
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

declare spec_file_path="${mydir}/../packages/hoprd/rest-api-v2-full-spec.yaml"
declare api_port=9876
declare node_log_file="node.logs"

log "Clean previously generated spec (if exists)"
rm -f "${spec_file_path}"

log "Start hoprd node"
cd "${mydir}/.."
DEBUG="hopr*" CI="true" yarn run run:hoprd --environment=master-goerli --admin false --api true --apiPort ${api_port} > "${node_log_file}" 2>&1 &

log "Wait 15 seconds for node startup to complete"
sleep 15

log "Verify spec has been generated at ${spec_file_path}"
test -f "${spec_file_path}" || {
  log "Spec file missing, printing node logs"
  cat "${node_log_file}"
}

log "Stop hoprd node"
lsof -i ":${api_port}" -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}

log "Remove hoprd node logs"
rm -f "${node_log_file}"

wait
