#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare -x HOPR_LOG_ID="run-local-anvil"
source "${mydir}/utils.sh"

usage() {
  msg
  msg "This script can be used to run a local Anvil instance at 127.0.0.1:8545"
  msg
  msg "Usage: $0 [<log_file>] [<cfg_file>]"
  msg
  msg "Parameters:"
  msg
  msg "log_file: (optional) log file name to be used by Anvil"
  msg "cfg_file: (optional) cfg file name to be used by Anvil"
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

declare tmp="$(find_tmp_dir)"
declare log_file="${1:-${tmp}/anvil.log}"
declare cfg_file="${2:-${mydir}/../.anvil.cfg}"
declare deployer_private_key=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

function cleanup {
  local EXIT_CODE=$?

  # at this point we don't want to fail hard anymore
  trap - SIGINT SIGTERM ERR
  set +Eeuo pipefail

  log "Stop anvil network"
  lsof -i ":8545" -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}

  log "Remove anvil configuration file ${cfg_file}"
  rm -f "${cfg_file}"

  wait

  exit $EXIT_CODE
}
trap cleanup SIGINT SIGTERM ERR

# mine a block every 2 seconds
declare flags="--block-time 2 --config-out ${cfg_file}"

if ! lsof -i ":8545" -s TCP:LISTEN; then
  log "Start local anvil network"
  ${mydir}/../.foundry/bin/anvil ${flags} > "${log_file}" 2>&1 &
  wait_for_regex ${log_file} "Listening on 127.0.0.1:8545"
  log "Anvil network started (127.0.0.1:8545)"
else
  log "Anvil network already running, skipping"
fi

log "Deploying contracts"
DEPLOYER_PRIVATE_KEY=${deployer_private_key} make -C ${mydir}/../packages/ethereum/contracts/ anvil-deploy-all
log "Deploying contracts finished"
