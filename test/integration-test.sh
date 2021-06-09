#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1;
}

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(dirname $(readlink -f $0))
declare HOPR_LOG_ID="e2e-test"
source "${mydir}/../scripts/utils.sh"

usage() {
  msg
  msg "Usage: $0 <node_api_1> <node_api_2> <node_api_3>"
  msg
  msg
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
test -z "${1:-}" && { msg "Missing first parameter"; usage; exit 1; }
test -z "${2:-}" && { msg "Missing second parameter"; usage; exit 1; }
test -z "${3:-}" && { msg "Missing third parameter"; usage; exit 1; }

declare api1="${1}"
declare api2="${2}"
declare api3="${3}"

# $1 = endpoint
# $2 = Hopr command
# $3 = negative assertion message
run_command(){
  local result
  local cmd="curl --silent --max-time 360 -X POST --data "$2" "$1/api/v1/command""

  result=$(${cmd})

  # if an error message was given and has been received, we fail
  if [[ -n "${3:-}" && "${result}" == *"${3:-}"* ]]; then
    log "${RED}run_command (${cmd}) FAILED, received: ${result}${NOFORMAT}"
    exit 1
  else
    echo "${result}"
  fi
}

get_eth_address(){
  curl --silent "$1/api/v1/address/eth"
}

get_hopr_address(){
  curl --silent "$1/api/v1/address/hopr"
}

validate_node_eth_address() {
  local ETH_ADDRESS
  local IS_VALID_ETH_ADDRESS

  ETH_ADDRESS="$(get_eth_address $1)"
  if [ -z "$ETH_ADDRESS" ]; then
    log "could not derive ETH_ADDRESS from first parameter $1"
    exit 1
  fi

  IS_VALID_ETH_ADDRESS="$(node -e "const ethers = require('ethers'); console.log(ethers.utils.isAddress('$ETH_ADDRESS'))")"
  if [ "$IS_VALID_ETH_ADDRESS" == "false" ]; then
    log "⛔️ Node returns an invalid address ETH_ADDRESS: $ETH_ADDRESS derived from $1"
    exit 1
  fi
  echo "$ETH_ADDRESS"
}

# TODO better validation
validate_node_balance_gt0() {
  local balance eth_balance hopr_balance

  balance="$(run_command $1 'balance')"
  eth_balance="$(echo -e "$balance" | grep -c " xDAI" || true)"
  hopr_balance="$(echo -e "$balance" | grep -c " HOPR" || true)"

  if [[ "$eth_balance" != "0" && "$hopr_balance" != "Hopr Balance: 0 HOPR" ]]; then
    log "$1 is funded"
  else
    log "⛔️ $1 Node has an invalid balance: $eth_balance, $hopr_balance"
    log "$balance"
    exit 1
  fi
}

log "Running full E2E test with ${api1}, ${api2}, ${api3}"

declare ETH_ADDRESS1
declare ETH_ADDRESS2
declare ETH_ADDRESS3

ETH_ADDRESS1="$(validate_node_eth_address "${api1}")"
ETH_ADDRESS2="$(validate_node_eth_address "${api2}")"
ETH_ADDRESS3="$(validate_node_eth_address "${api3}")"
log "ETH addresses exist"

validate_node_balance_gt0 "${api1}"
validate_node_balance_gt0 "${api2}"
validate_node_balance_gt0 "${api3}"
log "Nodes are funded"

declare result

log "Check peers"
result=$(run_command ${api1} 'peers' 'no connected peers')
log "-- ${result}"

declare HOPR_ADDRESS1
HOPR_ADDRESS1="$(get_hopr_address "${api1}")"
log "HOPR_ADDRESS1: $HOPR_ADDRESS1"

declare HOPR_ADDRESS2
HOPR_ADDRESS2="$(get_hopr_address "${api2}")"
log "HOPR_ADDRESS2: $HOPR_ADDRESS2"

log "Node 1 ping node 2"
result=$(run_command ${api1} "ping $HOPR_ADDRESS2" "Could not ping node. Timeout.")
log "-- ${result}"

log "Node 1 tickets: $(run_command ${api1} 'tickets')"

log "Node 1 send 0-hop message to node 2"
run_command "${api1}" "send ,$HOPR_ADDRESS2 'hello, world'"

log "Node 1 open channel to Node 2"
run_command "${api1}" "open $HOPR_ADDRESS2 0.1"

log "Node 1 send 10x 1 hop message to self via node 2"
run_command "${api1}" "send $HOPR_ADDRESS2,$HOPR_ADDRESS1 'hello, world 1'"
run_command "${api1}" "send $HOPR_ADDRESS2,$HOPR_ADDRESS1 'hello, world 2'"
run_command "${api1}" "send $HOPR_ADDRESS2,$HOPR_ADDRESS1 'hello, world 3'"
run_command "${api1}" "send $HOPR_ADDRESS2,$HOPR_ADDRESS1 'hello, world 4'"
run_command "${api1}" "send $HOPR_ADDRESS2,$HOPR_ADDRESS1 'hello, world 5'"
run_command "${api1}" "send $HOPR_ADDRESS2,$HOPR_ADDRESS1 'hello, world 6'"
run_command "${api1}" "send $HOPR_ADDRESS2,$HOPR_ADDRESS1 'hello, world 7'"
run_command "${api1}" "send $HOPR_ADDRESS2,$HOPR_ADDRESS1 'hello, world 8'"
run_command "${api1}" "send $HOPR_ADDRESS2,$HOPR_ADDRESS1 'hello, world 9'"
run_command "${api1}" "send $HOPR_ADDRESS2,$HOPR_ADDRESS1 'hello, world 10'"

log "Node 2 should now likely have a ticket"
run_command "${api2}" "tickets"
