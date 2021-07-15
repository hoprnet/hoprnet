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
declare alice_pipe="${tmp}/hopr-connect-alice-pipe.log"
declare alice_port=11090

declare bob_log="${tmp}/hopr-connect-bob.log"
declare bob_pipe="${tmp}/hopr-connect-bob-pipe.log"
declare bob_port=11091

declare charly_log="${tmp}/hopr-connect-charly.log"
declare charly_port=11092

function free_ports {
    for port in ${alice_port} ${bob_port} ${charly_port}; do
        if lsof -i ":${port}" -s TCP:LISTEN > /dev/null; then
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
    declare script=${3}
    declare rest_args=${@:4}

    DEBUG=hopr-connect*,simple-peer \
    yarn dlx \
    ts-node \
        "${filename}" \
        > "${log_file}" \
        ${rest_args} \
        --script "${script}" \
        2>&1 &
    declare pid=$!
    log "node started with PID ${pid}"
    log "args: ${rest_args}"
    log "script: "
    log "${script}"
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
for file in "${alice_log}" "${bob_log}" "${charly_log}" "${alice_pipe}" "${bob_pipe}"; do 
  rm -Rf ${file}
done

log "alice logs -> ${alice_log}"
log "alice msgs -> ${alice_pipe}"
log "bob logs -> ${bob_log}"
log "bob msgs -> ${bob_pipe}"
log "charly logs -> ${charly_log}"

# run alice (client)
start_node tests/node.ts \
    "${alice_log}" \
    "[ {
        'cmd': 'wait',
        'delay': 8
      },
      {
        'cmd': 'dial',
        'targetIdentityName': 'charly',
        'targetPort': ${charly_port}
      },
      {
        'cmd': 'msg',
        'relayIdentityName': 'charly',
        'targetIdentityName': 'bob',
        'msg': 'test'
      }
    ]" \
    --port ${alice_port} \
    --pipeFile "${alice_pipe}" \
    --identityName 'alice' \
    --bootstrapPort ${charly_port} \
    --bootstrapIdentityName 'charly' \
    --noDirectConnections true \
    --noWebRTCUpgrade false \
    
# run bob (client)
start_node tests/node.ts "${bob_log}"  \
  "[ {
        'cmd': 'wait',
        'delay': 8
      },
      {
        'cmd': 'dial',
        'targetIdentityName': 'charly',
        'targetPort': ${charly_port}
      }      
    ]" \
  --port ${bob_port} \
  --pipeFile "${bob_pipe}" \
  --identityName 'bob' \
  --bootstrapPort ${charly_port} \
  --bootstrapIdentityName 'charly' \
  --noDirectConnections true \
  --noWebRTCUpgrade false \  
  
# run charly (bootstrap, relay)
start_node tests/node.ts "${charly_log}" \
  "[]" \
  --port ${charly_port} \
  --identityName 'charly' \
  --noDirectConnections true \
  --noWebRTCUpgrade false


# wait till nodes finish communicating
wait_for_regex_in_file "${alice_log}" "all tasks executed"
wait_for_regex_in_file "${bob_log}" "all tasks executed"
wait_for_regex_in_file "${charly_log}" "all tasks executed"


expect_file_content "${alice_pipe}" \
">bob: test
<bob: echo: test"

expect_file_content "${bob_pipe}" \
"<alice: test
>alice: echo: test"

log "Test succesful"