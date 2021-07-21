#!/usr/bin/env bash

# prevent execution of this script, only allow sourcing
$(return >/dev/null 2>&1)
test "$?" -eq "0" || { echo "This script should only be sourced." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="testnet"
source "${mydir}/utils.sh"
source "${mydir}/gcloud.sh"
source "${mydir}/dns.sh"

declare min_funds=0.01291
declare min_funds_hopr=0.5

# $1 = role (ie. node-4)
# $2 = network name
vm_name() {
  echo "$2-$1"
}

# $1 = vm name
disk_name() {
  echo "$1-dsk"
}

# $1 = account (hex)
# $2 = chain provider
wallet_balance() {
  local address=${1}
  local rpc=${2}

  yarn run --silent ethers eval "new ethers.providers.JsonRpcProvider('${rpc}').getBalance('$1').then(b => formatEther(b))"
}

# $1 = chain provider
funding_wallet_address() {
  local rpc="${1}"

  # the value of FUNDING_PRIV_KEY must be prefixed with 0x
  yarn run --silent ethers --rpc "${rpc}" --account ${FUNDING_PRIV_KEY} eval 'accounts[0].getAddress().then(a => a)'
}

# $1 = account (hex)
# $2 = chain provider
# $3 = optional: hopr token contract
fund_if_empty() {
  local address="${1}"
  local rpc="${2}"
  local token_contract="${3:-}"

  local funding_address funding_balance balance

  funding_address=$(funding_wallet_address "${rpc}")

  log "Checking balance of funding wallet ${funding_address} using RPC ${rpc}"
  funding_balance="$(wallet_balance "${funding_address}" "${rpc}")"

  if [ "${funding_balance}" = '0.0' ]; then
    log "Wallet ${funding_address} has zero balance and cannot fund node ${address}"
  else
    log "Funding wallet ${funding_address} has enough funds: ${funding_balance}"
    log "Checking balance of the wallet ${address} to be funded"
    balance="$(wallet_balance "${address}" "${rpc}")"

    log "Balance of ${address} is ${balance}"
    if [ "${balance}" = '0.0' ]; then
      # need to wait to make retries work
      local ethers_opts="--rpc ${rpc} --account ${FUNDING_PRIV_KEY} --yes --wait"

      log "Funding account with native token -> ${address} ${min_funds}"
      try_cmd "yarn run --silent ethers send ${address} ${min_funds} ${ethers_opts}" 12 10

      if [ -n "${token_contract}" ]; then
        # at this point we assume that the account needs HOPR as well
        log "Funding account with HOPR token -> ${address} ${min_funds_hopr}"
        try_cmd "yarn run --silent ethers send-token ${token_contract} ${address} ${min_funds_hopr} ${ethers_opts}" 12 10
      fi
    fi
  fi
}

# $1 = IP
# $2 = optional: port, defaults to 3001
get_eth_address(){
  local ip=${1}
  local port=${2:-3001}
  local cmd="curl ${ip}:${port}/api/v1/address/eth"

  # try every 5 seconds for 5 minutes
  try_cmd "${cmd}" 30 5
}

# $1 = IP
# $2 = optional: port, defaults to 3001
get_hopr_address() {
  local ip=${1}
  local port=${2:-3001}
  local cmd="curl ${ip}:${port}/api/v1/address/hopr"

  # try every 5 seconds for 5 minutes
  try_cmd "${cmd}" 30 5
}

# $1 = IP
# $2 = Hopr command
# $3 = optional: port
run_command(){
  curl --silent -X POST --data "${2}" "${1}:${3:-3001}/api/v1/command"
}

# $1 = vm name
# $2 = docker image
# $3 = chain provider
update_if_existing() {
  if [[ $(gcloud_find_vm_with_name $1) ]]; then
    log "Container exists, updating" 1>&2
    PREV=$(gcloud_get_image_running_on_vm $1)
    if [ "$PREV" == "$2" ]; then
      log "Same version of image is currently running. Skipping update to $PREV" 1>&2
      return 0
    fi
    log "Previous GCloud VM Image: $PREV"
    gcloud_update_container_with_image $1 $2 "$(disk_name $1)" "/app/db" "${3}"

    # prevent docker images overloading the disk space
    gcloud_cleanup_docker_images "$1"
  else
    echo "no container"
  fi

}

# $1 = vm name
# $2 = docker image
# $3 = chain provider
# NB: --run needs to be at the end or it will ignore the other arguments.
start_testnode_vm() {
  local rpc=${3}
  local api_token="${HOPRD_API_TOKEN}"
  local password="${BS_PASSWORD}"

  if [ "$(update_if_existing $1 $2 ${rpc})" = "no container" ]; then
    gcloud compute instances create-with-container $1 $GCLOUD_DEFAULTS \
      --create-disk name=$(disk_name $1),size=10GB,type=pd-standard,mode=rw \
      --container-mount-disk mount-path="/app/db" \
      --container-env=^,@^DEBUG=hopr\*,@NODE_OPTIONS=--max-old-space-size=4096,@GCLOUD=1 \
      --container-image=$2 \
      --container-arg="--admin" \
      --container-arg="--adminHost" --container-arg="0.0.0.0" \
      --container-arg="--announce" \
      --container-arg="--apiToken" --container-arg="${api_token}" \
      --container-arg="--healthCheck" \
      --container-arg="--healthCheckHost" --container-arg="0.0.0.0" \
      --container-arg="--identity" --container-arg="/app/db/.hopr-identity" \
      --container-arg="--init" \
      --container-arg="--password" --container-arg="${password}" \
      --container-arg="--provider" --container-arg="${rpc}" \
      --container-arg="--rest" \
      --container-arg="--restHost" --container-arg="0.0.0.0" \
      --container-arg="--run" --container-arg="\"cover-traffic start;daemonize\"" \
      --container-restart-policy=always
  fi
}

# $1 = vm name
# Run a VM with a hardhat instance
start_chain_provider(){
  gcloud compute instances create-with-container $1-provider $GCLOUD_DEFAULTS \
      --create-disk name=$(disk_name $1),size=10GB,type=pd-standard,mode=rw \
      --container-image='hopr-provider'

  #hardhat node --config packages/ethereum/hardhat.config.ts
}

# $1 = network name
# $2 = docker image
# $3 = node number
# $4 = chain provider
start_testnode() {
  local vm ip eth_address

  # start or update vm
  vm=$(vm_name "node-$3" $1)
  log "- Starting test node $vm with $2 ${4}"
  start_testnode_vm $vm $2 ${4}

  # ensure node has funds, even after just updating a release
  ip=$(gcloud_get_ip "${vm}")
  wait_until_node_is_ready $ip
  eth_address=$(get_eth_address "${ip}")
  fund_if_empty "${eth_address}" "${4}"
}

# $1 authorized keys file
add_keys() {
  if test -f "$1"; then
    log "Reading keys from $1"
    cat $1 | xargs -I {} gcloud compute os-login ssh-keys add --key="{}"
  else
    echo "Authorized keys file not found"
  fi
}

# ----- Start Testnet -------
#
# Using a standard naming scheme, based on a name, we
# either update or start VM's to create a network of
# N nodes

# $1 = network name
# $2 = number of nodes
# $3 = docker image
# $4 = chain provider
start_testnet() {
  for i in $(seq 1 $2);
  do
    log "Start node $i"
    start_testnode $1 $3 $i ${4}
  done
  # @jose can you fix this pls.
  # add_keys scripts/keys/authorized_keys
}
