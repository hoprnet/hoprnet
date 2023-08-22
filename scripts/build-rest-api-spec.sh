#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare -x HOPR_LOG_ID="build-rest-api-spec"
# shellcheck disable=SC1090
source "${mydir}/utils.sh"

usage() {
  msg
  msg "This script can be used to generate the OpenAPI spec file for hoprd's Rest API v3."
  msg
  msg "Usage: $0"
  msg
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

declare spec_file_path="${mydir}/../packages/hoprd/rest-api-v3-full-spec.json"
declare api_port=9876
declare tmp="$(find_tmp_dir)"
declare node_log_file="${tmp}/node.logs"
declare anvil_rpc_log="${tmp}/hopr-apidocgen-anvil-rpc.log"

function cleanup {
  local EXIT_CODE=$?

  # at this point we don't want to fail hard anymore
  trap - SIGINT SIGTERM ERR EXIT
  set +Eeuo pipefail

  log "Stop hoprd node"
  lsof -i ":${api_port}" -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}

  log "Stop anvil"
  make -C "${mydir}/../" kill-anvil

  log "Remove logs"
  rm -f "${node_log_file}" "${anvil_rpc_log}"

  wait

  exit $EXIT_CODE
}
trap cleanup SIGINT SIGTERM ERR EXIT

log "Clean previously generated spec (if exists)"
rm -f "${spec_file_path}"

make -C "${mydir}/../" run-anvil

# need to mirror contract data because of anvil-deploy node only writing to localhost {{{
declare protocol_config="${mydir}/../packages/core/protocol-config.json"
declare deployments_summary="${mydir}/../packages/ethereum/contracts/contracts-addresses.json"
update_protocol_config_addresses "${protocol_config}" "${deployments_summary}" "anvil-localhost" "anvil-localhost"
update_protocol_config_addresses "${protocol_config}" "${deployments_summary}" "anvil-localhost" "anvil-localhost2"

log "Start hoprd node"
env DEBUG="hopr*" CI="true" HOPRD_API_PORT="${api_port}" \
  make -C "${mydir}/../" run-local-with-safe > "${node_log_file}" 2>&1 &

log "Wait 15 seconds for node startup to complete"
sleep 15

log "Verify spec has been generated at ${spec_file_path}"
test -f "${spec_file_path}" || {
  log "Spec file missing, printing node logs"
  cat "${node_log_file}"
}
