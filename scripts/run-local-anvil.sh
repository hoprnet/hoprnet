#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && {
  echo "This script should only be executed." >&2
  exit 1
}

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
  msg "This script can be used to run a local Anvil instance at 0.0.0.0:PORT"
  msg
  msg "Usage: $0 [-h|--help] [-f] [-l <log_file>] [-c <cfg_file>] [-p <port>] [-ls <state_file>] [-ds <state_file>] [-s] [-sp]"
  msg
  msg "Options:"
  msg
  msg "-h|--help: Display help and exit"
  msg "-s: Skip deployment of contracts"
  msg "-f: Start anvil in foreground, blocking the script execution"
  msg "-c <cfg_file>: Use particular anvil config file; default is ../.anvil.cfg"
  msg "-l <log_file>: Use particular anvil log file; default is /tmp/anvil.log"
  msg "-p <port>: Use particular port; default is 8545"
  msg "-ds <state_file>: Use particular state file to dump state on exit"
  msg "-ls <state_file>: Use particular state file to load state on startup"
  msg "-sp: Use staking proxy. If not supplied, the script will use the dummy proxy."
  msg
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && {
  usage
  exit 0
}

declare tmp="$(find_tmp_dir)"
declare log_file="${tmp}/anvil.log"
declare cfg_file="${mydir}/../.anvil.cfg"
declare port="8545"
declare deployer_private_key=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
declare foreground="false"
declare skip_deploy="false"
declare dump_state_file=""
declare load_state_file=""
declare use_staking_proxy="false"

while (("$#")); do
  case "$1" in
  -h | --help)
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
  -p)
    port="$2"
    shift
    shift
    ;;
  -ds)
    dump_state_file="$2"
    shift
    shift
    ;;
  -ls)
    load_state_file="$2"
    shift
    shift
    ;;
  -sp)
    # use staking proxy
    use_staking_proxy="true"
    shift
    ;;
  -* | --*=)
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
  lsof -i ":${port}" -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}

  log "Remove anvil configuration file ${cfg_file}"
  rm -f "${cfg_file}"

  wait

  exit $EXIT_CODE
}
trap cleanup SIGINT SIGTERM ERR

# mine a block every 2 seconds
declare flags="--host 0.0.0.0 --port ${port} --block-time 2 --config-out ${cfg_file}"
if [ -n "${dump_state_file}" ]; then
  flags="${flags} --dump-state ${dump_state_file}"
fi
if [ -n "${load_state_file}" ]; then
  if [ -f "${load_state_file}" ]; then
    flags="${flags} --load-state ${load_state_file}"
  else
    log "State file ${load_state_file} does not exist!"
    exit 101
  fi
fi

# prepare PATH if anvil is not present yet
if ! command -v anvil; then
  PATH=${PATH}:${mydir}/../.foundry/bin
fi
if ! command -v anvil; then
  echo "Error: cannot find anvil"
  exit 1
fi

if ! lsof -i ":${port}" -s TCP:LISTEN; then
  log "Start local anvil chain"
  if [ "${foreground}" = "true" ]; then
    anvil ${flags} >"${log_file}" 2>&1 &
  else
    # ignore hangup signals so the script can complete without anvil taking
    # notice
    nohup nice anvil ${flags} >"${log_file}" 2>&1 &
  fi
  get_eth_block_number "http://localhost:${port}"
  log "Anvil chain started (0.0.0.0:${port})"
else
  log "Anvil chain already running, skipping"
fi

if [ "${skip_deploy}" != "true" ]; then
  log "Deploying contracts"
  env \
    FOUNDRY_ETH_RPC_URL="http://127.0.0.1:${port}" \
    ETH_RPC_URL="http://127.0.0.1:${port}" \
    DEPLOYER_PRIVATE_KEY=${deployer_private_key} \
    USE_STAKING_PROXY=${use_staking_proxy} \
    make -C "${mydir}"/../ethereum/contracts/ anvil-deploy-all
  log "Deploying contracts finished"
fi

if [ "${foreground}" = "true" ]; then
  wait
fi
