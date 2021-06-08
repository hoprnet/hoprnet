#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1;
}

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

usage() {
  echo >&2
  echo "Usage: $0 <node_api_1> <node_api_2> <node_api_3>" >&2
  echo
  echo >&2
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
test -z "${1:-}" && { echo "Missing first parameter" >&2; usage; exit 1; }
test -z "${2:-}" && { echo "Missing second parameter" >&2; usage; exit 1; }
test -z "${3:-}" && { echo "Missing third parameter" >&2; usage; exit 1; }

declare api1="${1}"
declare api2="${2}"
declare api3="${3}"

# $1 = IP
# $2 = Hopr command
run_command(){
  curl --silent --max-time 360 -X POST --data "$2" "$1/api/v1/command"
}

get_eth_address(){
  curl --silent "$1/api/v1/address/eth"
}

get_hopr_address(){
  curl --silent "$1/api/v1/address/hopr"
}

validate_ip() {
  if [ -z "$1" ]; then
    echo "missing ip as first parameter"
    exit 1
  fi
}

validate_node_eth_address() {
  local ETH_ADDRESS
  local IS_VALID_ETH_ADDRESS

  ETH_ADDRESS="$(get_eth_address $1)"
  if [ -z "$ETH_ADDRESS" ]; then
    echo "could not derive ETH_ADDRESS from first parameter $1"
    exit 1
  fi

  IS_VALID_ETH_ADDRESS="$(node -e "const ethers = require('ethers'); console.log(ethers.utils.isAddress('$ETH_ADDRESS'))")"
  if [ "$IS_VALID_ETH_ADDRESS" == "false" ]; then
    echo "⛔️ Node returns an invalid address ETH_ADDRESS: $ETH_ADDRESS derived from $1"
    exit 1
  fi
  echo "$ETH_ADDRESS"
}

# TODO better validation
validate_node_balance_gt0() {
  local BALANCE
  local ETH_BALANCE
  local HOPR_BALANCE

  BALANCE="$(run_command $1 'balance')"
  ETH_BALANCE="$(echo -e "$BALANCE" | grep -c " xDAI" || true)"
  HOPR_BALANCE="$(echo -e "$BALANCE" | grep -c " HOPR" || true)"

  if [[ "$ETH_BALANCE" != "0" && "$HOPR_BALANCE" != "Hopr Balance: 0 HOPR" ]]; then
    echo "- $1 is funded"
  else
    echo "⛔️ $1 Node has an invalid balance: $ETH_BALANCE, $HOPR_BALANCE"
    echo -e "$BALANCE"
    exit 1
  fi
}

echo "- Running full E2E test with ${api1}, ${api2}, ${api3}"

validate_ip "${api1}"
validate_ip "${api2}"
validate_ip "${api3}"
echo "- IP's exist"

declare ETH_ADDRESS1
declare ETH_ADDRESS2
declare ETH_ADDRESS3

ETH_ADDRESS1="$(validate_node_eth_address "${api1}")"
ETH_ADDRESS2="$(validate_node_eth_address "${api2}")"
ETH_ADDRESS3="$(validate_node_eth_address "${api3}")"
echo "- ETH addresses exist"

validate_node_balance_gt0 "${api1}"
validate_node_balance_gt0 "${api2}"
validate_node_balance_gt0 "${api3}"
echo "- Nodes are funded"

echo "$(run_command ${api1} 'peers')"

declare HOPR_ADDRESS1
HOPR_ADDRESS1="$(get_hopr_address "${api1}")"
echo "HOPR_ADDRESS1: $HOPR_ADDRESS1"

declare HOPR_ADDRESS2
HOPR_ADDRESS2="$(get_hopr_address "${api2}")"
echo "HOPR_ADDRESS2: $HOPR_ADDRESS2"

echo "- Node 1 ping node 2: $(run_command ${api1} "ping $HOPR_ADDRESS2")"

echo "- Node 1 tickets: $(run_command ${api1} 'tickets')"

echo "- Node 1 send 0-hop message to node 2"
run_command "${api1}" "send ,$HOPR_ADDRESS2 'hello, world'"

echo "- Node 1 open channel to Node 2"
run_command "${api1}" "open $HOPR_ADDRESS2 0.1"

echo "- Node 1 send 10x 1 hop message to self via node 2"
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

echo "- Node 2 should now likely have a ticket"
run_command "${api2}" "tickets"
