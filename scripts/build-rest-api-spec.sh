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

declare spec_file_path="${mydir}/../packages/hoprd/rest-api-v2-full-spec.json"
declare api_port=9876
declare tmp="$(find_tmp_dir)"
declare node_log_file="${tmp}/node.logs"
declare hardhat_rpc_log="${tmp}/hopr-apidocgen-hardhat-rpc.log"

function cleanup {
  local EXIT_CODE=$?

  # at this point we don't want to fail hard anymore
  trap - SIGINT SIGTERM ERR EXIT
  set +Eeuo pipefail

  log "Stop hoprd node"
  lsof -i ":${api_port}" -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}

  log "Stop hardhat"
  lsof -i ":8545" -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}

  log "Remove logs"
  rm -f "${node_log_file}" "${hardhat_rpc_log}"

  wait

  exit $EXIT_CODE
}
trap cleanup SIGINT SIGTERM ERR EXIT

log "Clean previously generated spec (if exists)"
rm -f "${spec_file_path}"

log "Start local hardhat network"
HOPR_ENVIRONMENT_ID="hardhat-localhost" \
TS_NODE_PROJECT=${mydir}/../packages/ethereum/tsconfig.hardhat.json \
  yarn workspace @hoprnet/hopr-ethereum hardhat node \
    --network hardhat \
    --show-stack-traces > \
    "${hardhat_rpc_log}" 2>&1 &
wait_for_regex ${hardhat_rpc_log} "Started HTTP and WebSocket JSON-RPC server"
log "Hardhat node started (127.0.0.1:8545)"

# need to mirror contract data because of hardhat-deploy node only writing to localhost {{{
cp -R \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost/localhost" \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost/hardhat"
cp -R \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost" \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost2"
# }}}

log "Start hoprd node"
cd "${mydir}/.."
DEBUG="hopr*" CI="true" \
  yarn run run:hoprd --environment=hardhat-localhost \
    --admin false --api true --apiPort ${api_port} > "${node_log_file}" \
    2>&1 &

log "Wait 15 seconds for node startup to complete"
sleep 15

log "Verify spec has been generated at ${spec_file_path}"
test -f "${spec_file_path}" || {
  log "Spec file missing, printing node logs"
  cat "${node_log_file}"
}
