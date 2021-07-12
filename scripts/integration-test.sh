#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }
          
# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="hopr-connect-test"
source "${mydir}/utils.sh"

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

usage() {
  msg
  msg "Usage: $0"
  msg
}

# setup paths
# find usable tmp dir
declare tmp="/tmp"
[[ -d "${tmp}" && -h "${tmp}" ]] && tmp="/var/tmp"
[[ -d "${tmp}" && -h "${tmp}" ]] && { msg "Neither /tmp or /var/tmp can be used for writing logs"; exit 1; }

declare alice_log="${tmp}/hopr-connect-alice.log"
declare alice_port=11090

declare bob_log="${tmp}/hopr-connect-bob.log"
declare bob_port=11091

declare charly_log="${tmp}/hopr-connect-charly.log"
declare charly_port=11092

function free_ports {
    for port in ${alice_port} ${bob_port} ${charly_port}; do
        if lsof -i ":${port}" -s TCP:LISTEN; then
        lsof -i ":${port}" -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}
        fi
    done
}

function cleanup {
  local EXIT_CODE=$?

  trap - SIGINT SIGTERM ERR EXIT

  free_ports

  exit $EXIT_CODE
}
trap cleanup SIGINT SIGTERM ERR EXIT

function start_node() {
    declare filename=${1}
    declare log_file=${2}
    declare rest_args=${@:3}

    DEBUG=hopr-connect*,simple-peer \
    yarn dlx \
    ts-node \
        "${filename}" \
        > "${log_file}" \
        ${rest_args} \
        2>&1 &
    declare pid=$!
    log "${filename} ${rest_args} started with PID ${pid}"
    echo ${pid}
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

declare exit_code=0

# check prerequisites
which yarn > /dev/null || exit_code=$?

if [[ "${exit_code}" != 0 ]]; then
    log "⛔️ yarn is not installed"
    exit 1
fi

declare yarn_version=$(yarn -v)
declare yarn_version_parsed=( ${yarn_version//./ } )
if [[ "${yarn_version_parsed[0]}" != "2" ]]; then
    log "⛔️ yarn v2.x.x required, ${yarn_version} found"
    exit 1
fi

free_ports

# check ports are free

for port in ${alice_port} ${bob_port} ${charly_port}; do
  ensure_port_is_free ${port}
done

log "Test started"

# remove logs
rm -Rf "${charly_log}"
rm -Rf "${bob_log}"
rm -Rf "${alice_log}"

log "alice -> ${alice_log}"
log "bob -> ${bob_log}"
log "charly -> ${charly_log}"

# run nodes
start_node examples/server.ts "${charly_log}" \
  --serverPort ${charly_port} \
  --serverIdentityName 'charly'

start_node examples/client.ts ${bob_log}  \
  --clientPort ${bob_port} \
  --clientIdentityName 'bob' \
  --relayPort ${charly_port} \
  --relayIdentityName 'charly'

start_node examples/client.ts ${alice_log} \
  --clientPort ${alice_port} \
  --clientIdentityName 'alice' \
  --relayPort ${charly_port} \
  --relayIdentityName 'charly' \
  --counterPartyIdentityName 'bob'

wait_for_regex_in_file ${bob_log} "Received message <test>"
wait_for_regex_in_file ${alice_log} "Received <Echoing <test>>"

log "Test succesful"