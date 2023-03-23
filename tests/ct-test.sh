#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# need to pass an alias into sub-shells
shopt -s expand_aliases

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="e2e-ct-test"
source "${mydir}/../scripts/utils.sh"

declare ct_node1_log="${1}"
declare healthcheck_host="${2}"
declare healthcheck_port="${3}"

log "Running Cover Traffic test"

log "Check CT daemon health"
curl "http://${healthcheck_host}:${healthcheck_port}/healthcheck/v1/version" | grep "CT node: "

log "Waiting for CT strategy to be applied"
wait_for_regex "${ct_node1_log}" "strategy tick RUNNING covertraffic"

log "Waiting for the first strategy tick"
wait_for_regex "${ct_node1_log}" "strategy tick:"

log "Waiting for the channel open"
wait_for_regex "${ct_node1_log}" "hopr:cover-traffic opening"

log "Waiting for the first CT send OR failed due to lacking number of nodes"
wait_for_regex "${ct_node1_log}" "(cover-traffic SEND)|(aborting send messages - less channels in network than hops required)"

log "Waiting for phase complete marker"
wait_for_regex "${ct_node1_log}" "message send phase complete"

log "Cover Traffic test finished successfully"
