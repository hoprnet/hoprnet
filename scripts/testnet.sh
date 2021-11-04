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

# $1=role (ie. node-4)
# $2=network name
vm_name() {
  local role="${1}"
  local network_name="${2}"

  echo "${network_name}-${role}"
}

# $1=vm name
disk_name() {
  local vm_name="${1}"
  echo "${vm_name}-dsk"
}

# $1=environment id
get_rpc() {
  local environment_id="${1}"
  local network_id=$(cat "${mydir}/../packages/core/protocol-config.json" | jq -r ".environments.\"${environment_id}\".network_id")
  local unresolved_rpc=$(cat "${mydir}/../packages/core/protocol-config.json" | jq -r ".networks.\"${network_id}\".default_provider")
  echo "${unresolved_rpc}" | envsubst
}

# $1=account (hex)
# $2=rpc
wallet_balance() {
  local address=${1}
  local rpc=${2} # @TODO: Replace RPC for chain to avoid calling only xdai

  curl https://api.hoprnet.org/api/balance/xdai/$1/native/get?text=true
}

# $1=rpc
funding_wallet_address() {
  local rpc="${1}" # @TODO: Remove RPC

  # the value of FUNDING_PRIV_KEY must be prefixed with 0x
  curl --silent https://api.hoprnet.org/api/faucet/address/get?text=true
}

# $1 = account (hex)
# $2 = token address (hex)
faucet_address_token() {
  local address="${1}"
  local token="${2}"
  local secret="${FAUCET_SECRET_API_KEY}"

  curl --silent --request POST \
  "https://api.hoprnet.org/api/faucet/xdai/$address/$token/default" \
  --header 'Content-Type: application/json' \
  --data-raw "{\"secret\": \"$secret\"}"
}

# $1 = account (hex)
faucet_address_ether() {
  local address="${1}"
  local secret="${FAUCET_SECRET_API_KEY}"

  curl --silent --request POST \
  "https://api.hoprnet.org/api/faucet/xdai/$address/native/default" \
  --header 'Content-Type: application/json' \
  --data-raw "{\"secret\": \"$secret\"}"
}

# $1 = IP
store_git_commit_per_ip() {
  local ip=${1}
  local commit=$(git rev-parse --short HEAD | cut -c1-7)
  local secret="${FAUCET_SECRET_API_KEY}"

  curl --silent --request POST \
  "https://api.hoprnet.org/api/ceramic/ip/$commit/post" \
  --header 'Content-Type: application/json' \
  --data-raw "{\"secret\": \"$secret\", \"ip\": \"$ip\"}"
}

# $1=account (hex)
# $2=environment_id
# $3=optional: hopr token contract
fund_if_empty() {
  local address="${1}"
  local environment_id="${2}"
  local token_contract="${3:-}"

  local funding_address funding_balance balance

  local rpc=$(get_rpc "${environment_id}")

  funding_address=$(funding_wallet_address "${rpc}")

  log "Checking balance of funding wallet ${funding_address} using RPC ${rpc}"
  funding_balance="$(wallet_balance "${funding_address}" "${rpc}")"

  if [ "${funding_balance}" == '0.0' ]; then
    log "Wallet ${funding_address} has zero balance and cannot fund node ${address}"
  else
    log "Funding wallet ${funding_address} has enough funds: ${funding_balance}"
    log "Checking balance of the wallet ${address} to be funded"
    balance="$(wallet_balance "${address}" "${rpc}")"

    log "Balance of ${address} is ${balance}"
    if [ "${balance}" == '0.0' ]; then
      # need to wait to make retries work
      local cmd_ether="faucet_address_ether ${address}"

      log "Funding account with native token -> ${address} ${min_funds}"
      try_cmd "${cmd_ether}" 12 10

      if [ -n "${token_contract}" ]; then
        # at this point we assume that the account needs HOPR as well
        local cmd_token="faucet_address_token ${address} ${token_contract}"
        log "Funding account with HOPR token -> ${address} ${min_funds_hopr}"
        try_cmd "${cmd_token}" 12 10
      fi
    fi
  fi
}

# $1=IP
# $2=optional: port, defaults to 3001
get_eth_address(){
  local ip=${1}
  local port=${2:-3001}
  local cmd="curl ${ip}:${port}/api/v1/address/eth"

  # try every 5 seconds for 5 minutes
  try_cmd "${cmd}" 30 5
}

# $1=IP
# $2=optional: port, defaults to 3001
get_hopr_address() {
  local ip=${1}
  local port=${2:-3001}
  local cmd="curl ${ip}:${port}/api/v1/address/hopr"

  # try every 5 seconds for 5 minutes
  try_cmd "${cmd}" 30 5
}

# $1=IP
# $2=Hopr command
# $3=optional: port
run_command(){
  curl --silent -X POST --data "${2}" "${1}:${3:-3001}/api/v1/command"
}

# $1=vm name
# $2=docker image
# $3=environment id
update_if_existing() {
  local vm_name=${1}
  local docker_image=${2}
  local environment_id=${3}

  if [[ $(gcloud_find_vm_with_name $1) ]]; then
    log "Container exists, updating" 1>&2
    PREV=$(gcloud_get_image_running_on_vm $1)
    if [ "$PREV" == "$2" ]; then
      log "Same version of image is currently running. Skipping update to $PREV" 1>&2
      return 0
    fi
    log "Previous GCloud VM Image: $PREV"
    gcloud_update_container_with_image "${vm_name}" "${docker_image}" "$(disk_name ${vm_name})" "/app/db" "${environment_id}"

    # prevent docker images overloading the disk space
    gcloud_cleanup_docker_images "${vm_name}"
  else
    echo "no container"
  fi

}

# $1=vm name
# $2=docker image
# $3=environment id
# $4=preemptible
# NB: --run needs to be at the end or it will ignore the other arguments.
start_testnode_vm() {
  local vm_name=${1}
  local docker_image=${2}
  local environment_id=${3}
  local api_token="${HOPRD_API_TOKEN}"
  local password="${BS_PASSWORD}"
  local preemptible_args=""
  local preemptible="${4:-}"

  if [ -n "${preemptible}" ]; then
    preemptible_args="--preemptible --metadata-from-file shutdown-script=scripts/gcloud-shutdown.sh"
  fi

  local rpc=$(get_rpc "${environment_id}")

  if [ "$(update_if_existing ${vm_name} ${docker_image} ${environment_id})"="no container" ]; then
    gcloud compute instances create-with-container ${vm_name} $GCLOUD_DEFAULTS \
      --create-disk name=$(disk_name ${vm_name}),size=10GB,type=pd-standard,mode=rw \
      --container-mount-disk mount-path="/app/db" \
      --container-env=^,@^DEBUG=hopr\*,@NODE_OPTIONS=--max-old-space-size=4096,@GCLOUD=1 \
      --container-image=${docker_image} \
      --container-arg="--admin" \
      --container-arg="--adminHost" --container-arg="0.0.0.0" \
      --container-arg="--announce" \
      --container-arg="--apiToken" --container-arg="${api_token}" \
      --container-arg="--healthCheck" \
      --container-arg="--healthCheckHost" --container-arg="0.0.0.0" \
      --container-arg="--identity" --container-arg="/app/db/.hopr-identity" \
      --container-arg="--init" \
      --container-arg="--password" --container-arg="${password}" \
      --container-arg="--environment" --container-arg="${environment_id}" \
      --container-arg="--rest" \
      --container-arg="--restHost" --container-arg="0.0.0.0" \
      --container-restart-policy=always \
      ${preemptible_args}
  fi
}

# $1=testnet name
# $2=docker image
# $3=node number
# $4=environment id
# $5=preemptible
start_testnode() {
  local vm ip eth_address
  local testnet_name=${1}
  local docker_image=${2}
  local node_number=${3}
  local environment_id=${4}
  local preemptible=${5}

  # start or update vm
  vm=$(vm_name "node-${node_number}" ${testnet_name})
  log "- Starting test node ${vm} with ${docker_image} ${environment_id}"
  start_testnode_vm ${vm} ${docker_image} ${environment_id} ${preemptible}

  # ensure node has funds, even after just updating a release
  ip=$(gcloud_get_ip "${vm}" "${preemptible}")
  wait_until_node_is_ready ${ip}
  store_git_commit_per_ip ${ip}
  eth_address=$(get_eth_address "${ip}")
  fund_if_empty "${eth_address}" "${environment_id}"
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

# $1=network name
# $2=number of nodes
# $3=docker image
# $4=chain provider
# $5=preemptible
start_testnet() {
  local preemptible="${5:-}"
  for i in $(seq 1 $2);
  do
    log "Start node $i"
    start_testnode $1 $3 $i ${4} $preemptible
  done
  # @jose can you fix this pls.
  # add_keys scripts/keys/authorized_keys
}
