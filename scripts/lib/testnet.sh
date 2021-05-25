#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

# prevent execution of this script, only allow sourcing
$(return >/dev/null 2>&1)
test "$?" -eq "0" || (echo "This script should only be sourced."; exit 1)

# don't source this file twice
test -z "${TESTNET_SOURCED:-}" && TESTNET_SOURCED=1 || exit 0

# source functions to work with gcloud and dns
source "$(dirname $(readlink -f $0))/gcloud.sh"
source "$(dirname $(readlink -f $0))/dns.sh"

# set log id and use shared log function for readable logs
declare HOPR_LOG_ID="testnet"
source "$(dirname $(readlink -f $0))/utils.sh"

# $1 = role (ie. node-4)
# $2 = network name
testnet_vm_name() {
  echo "$2-$1"
}

# $1 = vm name
testnet_disk_name() {
  echo "$1-dsk"
}

# $1=account (hex)
testnet_balance() {
  ethers eval "new ethers.providers.JsonRpcProvider('$RPC').getBalance('$1').then(b => formatEther(b))"
}

# $1=account (hex)
testnet_fund_if_empty() {
  local MIN_FUNDS=0.01291
  local BALANCE="$(balance $1)"

  log "Checking balance of $1"
  log "Balance is $BALANCE"

  if [ "$BALANCE" = '0.0' ]; then
    log "Funding account ... $RPC -> $1 $MIN_FUNDS"
    ethers send --rpc "$RPC" --account "$FUNDING_PRIV_KEY" "$1" $MIN_FUNDS --yes
    sleep 60
  fi
}

# $1 = IP
testnet_get_eth_address(){
  curl "$1:3001/api/v1/address/eth"
}

# $1 = IP
testnet_get_hopr_address() {
  curl "$1:3001/api/v1/address/hopr"
}

# $1 = IP
# $2 = Hopr command
testnet_run_command(){
  curl -gsilent -X POST --data "$2" "$1:3001/api/v1/command"
}

# $1 = vm name
# $2 = docker image
testnet_update_node_if_existing() {
  if [[ $(gcloud_find_vm_with_name $1) ]]; then
    log "Container exists, updating" 1>&2
    PREV=$(gcloud_get_image_running_on_vm $1)
    if [ "$PREV" == "$2" ]; then
      log "Same version of image is currently running. Skipping update to $PREV" 1>&2
      return 0
    fi
    log "Previous GCloud VM Image: $PREV"
    gcloud_update_container_with_image $1 $2 "$(disk_name $1)" "/app/db"
    sleep 60
  else
    echo "no container"
  fi
}

# Run a VM with a hardhat instance
# $1 - network name
# $2 - docker image
testnet_start_chain_provider() {
  local name="${1}-chainprovider"

  # start vm
  gcloud_create_container_instance "${name}" "${2}" ""
}

# $1 - network name
# $2 - docker image
# $3 - node number
# $4 - database password
# $5 - OPTIONAL chain provider
testnet_start_node() {
  local name=$(testnet_vm_name "node-$3" $1)
  local gcloud_args="\
    --container-mount-disk mount-path=\"/app/db\" \
    --container-env=^,@^DEBUG=hopr\*,@NODE_OPTIONS=--max-old-space-size=4096,@GCLOUD=1 \
    --container-arg=\"--password\" --container-arg=\"$4\" \
    --container-arg=\"--init\" --container-arg=\"true\" \
    --container-arg=\"--announce\" --container-arg=\"true\" \
    --container-arg=\"--rest\" --container-arg=\"true\" \
    --container-arg=\"--restHost\" --container-arg=\"0.0.0.0\" \
    --container-arg=\"--healthCheck\" --container-arg=\"true\" \
    --container-arg=\"--healthCheckHost\" --container-arg=\"0.0.0.0\" \
    --container-arg=\"--admin\" --container-arg=\"true\" \
    --container-arg=\"--adminHost\" --container-arg=\"0.0.0.0\" \
    --container-arg=\"--run\" --container-arg=\"\"cover-traffic start;daemonize\" \
  "

  if [ -n "$5" ]; then
    log "using chain provider $5"
    gcloud_args="${gcloud_args} --container-arg=\"--provider\" --container-arg=\"$5\""
  fi

  log "Starting test node $vm with $2 $5"
  if [ "$(testnet_update_node_if_existing $1 $2)" = "no container" ]; then
    gcloud_create_container_instance "${name}" "${2}" "${gcloud_args}"
  fi
}

# $1 network name
# $2 node number
testnet_destroy_node() {
  local name=$(testnet_vm_name "node-$2" $1)

  # delete disks as well since these are test nodes
  gcloud_delete_instance "${name}" "" "true"
}

# $1 authorized keys file
testnet_add_keys() {
  if test -f "$1"; then
    log "Reading keys from $1"
    cat $1 | xargs -I {} gcloud compute os-login ssh-keys add --key="{}"
  else
    log "Authorized keys file not found"
  fi
}

# ----- Start Testnet -------
#
# Using a standard naming scheme, based on a name, we
# either update or start VM's to create a network of
# N nodes

# $1 network name
# $2 number of nodes
# $3 docker image
# $4 = OPTIONAL chain provider
testnet_start() {
  for i in $(seq 1 $2);
  do
    log "Start node $i"
    testnet_start_node $1 $3 $i $4
  done
  # @jose can you fix this pls.
  # add_keys scripts/keys/authorized_keys
}

# $1 network name
# $2 number of nodes
testnet_destroy() {
  for i in $(seq 1 $2);
  do
    log "Destroy node $i"
    testnet_destroy_node $1 $i
  done
}
