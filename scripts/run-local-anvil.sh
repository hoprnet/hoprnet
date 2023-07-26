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
# shellcheck disable=SC1090
source "${mydir}/utils.sh"

usage() {
  msg
  msg "This script can be used to run a local Anvil instance at 0.0.0.0:8545"
  msg
  msg "Usage: $0 [-h|--help] [-f] [-l <log_file>] [-c <cfg_file>] [-s]"
  msg
  msg "Options:"
  msg
  msg "-h|--help: Display help and exit"
  msg "-s: Skip deployment of contracts"
  msg "-f: Start anvil in foreground, blocking the script execution"
  msg "-c <cfg_file>: Use particular anvil config file; default is ../.anvil.cfg"
  msg "-l <log_file>: Use particular anvil log file; default is /tmp/anvil.log"
  msg
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

declare tmp="$(find_tmp_dir)"
declare log_file="${tmp}/anvil.log"
declare cfg_file="${mydir}/../.anvil.cfg"
declare deployer_private_key=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
declare foreground="false"
declare skip_deploy="false"

while (( "$#" )); do
  case "$1" in
    -h|--help)
      # return early with help info when requested
      usage
      exit 0
      ;;
    -l)
      log_file="$2"
      shift
      shift
      ;;
    -c)
      cfg_file="$2"
      shift
      shift
      ;;
    -s)
      skip_deploy="true"
      shift
      ;;
    -f)
      foreground="true"
      shift
      ;;
    -*|--*=)
      usage
      exit 1
      ;;
    *)
      shift
      ;;
  esac
done

# derive location of the state file from the config file
declare state_file="${cfg_file%.cfg}.state.json"

function cleanup {
  local EXIT_CODE=$?

  # at this point we don't want to fail hard anymore
  trap - SIGINT SIGTERM ERR
  set +Eeuo pipefail

  log "Stop anvil chain"
  lsof -i ":8545" -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}

  log "Remove anvil configuration file ${cfg_file}"
  rm -f "${cfg_file}"

  wait

  exit $EXIT_CODE
}
trap cleanup SIGINT SIGTERM ERR

# mine a block every 2 seconds
declare flags="--host 0.0.0.0 --block-time 2 --config-out ${cfg_file}"

# prepare PATH if anvil is not present yet
if ! command -v anvil ; then
  PATH=${PATH}:${mydir}/../.foundry/bin
fi
if ! command -v anvil ; then
  echo "Error: cannot find anvil"
  exit 1
fi

if ! lsof -i ":8545" -s TCP:LISTEN; then
  log "Start local anvil chain"
  anvil ${flags} > "${log_file}" 2>&1 &
  wait_for_regex "${log_file}" "Listening on 0.0.0.0:8545"
  log "Anvil chain started (0.0.0.0:8545)"
else
  log "Anvil chain already running, skipping"
fi

if [ "${skip_deploy}" != "true" ]; then
  log "Deploying contracts"
  DEPLOYER_PRIVATE_KEY=${deployer_private_key} make -C "${mydir}"/../packages/ethereum/contracts/ anvil-deploy-all
  log "Deploying contracts finished"
fi

if [ "${foreground}" = "true" ]; then
  wait
fi
