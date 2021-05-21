#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

# set log id and use shared log function for readable logs
declare HOPR_LOG_ID="e2e-test"
source "$(dirname $(readlink -f $0))/../scripts/utils.sh"

# -- Integration test --
# We assume the existence of a test network with three nodes:
# api1, api2 api3.

declare api1="${1}"
declare api2="${2}"
declare api3="${3}"

# $1 = IP
# $2 = Hopr command
run_command(){
  curl --silent -X POST --data "$2" "$1/api/v1/command"
}

get_eth_address(){
  curl --silent "$1/api/v1/address/eth"
}

get_hopr_address(){
  curl --silent "$1/api/v1/address/hopr"
}

validate_ip() {
  if [ -z "$1" ]; then
    log "missing ip as first parameter"
    exit 1
  fi
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
  local BALANCE
  local ETH_BALANCE
  local HOPR_BALANCE

  BALANCE="$(run_command $1 'balance')"
  ETH_BALANCE="$(echo -e "$BALANCE" | grep -c " xDAI" || true)"
  HOPR_BALANCE="$(echo -e "$BALANCE" | grep -c " HOPR" || true)"

  if [[ "$ETH_BALANCE" != "0" && "$HOPR_BALANCE" != "Hopr Balance: 0 HOPR" ]]; then
    log "$1 is funded"
  else
    log "⛔️ $1 Node has an invalid balance: $ETH_BALANCE, $HOPR_BALANCE"
    log "$BALANCE"
    exit 1
  fi
}

log "Running full E2E test with $api1, $api2, $api3"

validate_ip "$api1"
validate_ip "$api2"
validate_ip "$api3"

declare ETH_ADDRESS1
declare ETH_ADDRESS2
declare ETH_ADDRESS3

ETH_ADDRESS1="$(validate_node_eth_address "$api1")"
ETH_ADDRESS2="$(validate_node_eth_address "$api2")"
ETH_ADDRESS3="$(validate_node_eth_address "$api3")"

validate_node_balance_gt0 "$api1"
validate_node_balance_gt0 "$api2"

log "Nodes are funded"

log "$(run_command $api1 'peers')"

declare HOPR_ADDRESS1
HOPR_ADDRESS1="$(get_hopr_address "$api1")"
log "HOPR_ADDRESS1: $HOPR_ADDRESS1"

declare HOPR_ADDRESS2
HOPR_ADDRESS2="$(get_hopr_address "$api2")"
log "HOPR_ADDRESS2: $HOPR_ADDRESS2"

log "Node 1 ping node 2: $(run_command $api1 "ping $HOPR_ADDRESS2")"

log "Node 1 tickets: $(run_command $api1 'tickets')"

log "Node 1 send 0-hop message to node 2"
run_command "$api1" "send ,$HOPR_ADDRESS2 'hello, world'"

log "Node 1 open channel to Node 2"
run_command "$api1" "open $HOPR_ADDRESS2 0.1"

log "Node 1 send 1 hop message to self via node 2"
run_command "$api1" "send $HOPR_ADDRESS2,$HOPR_ADDRESS1 'hello, world'"

log "Node 2 should now have a ticket"
run_command "$api2" "tickets"
