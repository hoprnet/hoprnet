#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

if [ -z "${GCLOUD_INCLUDED:-}" ]; then
  source scripts/gcloud.sh
  source scripts/dns.sh
fi

source scripts/utils.sh

MIN_FUNDS=0.01291
ZONE="--zone=europe-west6-a"

# $1 = role (ie. node-4)
# $2 = network name
vm_name() {
  echo "$2-$1"
}

# $1 = vm name
disk_name() {
  echo "$1-dsk"
}

# $1=account (hex)
balance() {
  yarn run --silent ethers eval "new ethers.providers.JsonRpcProvider('$RPC').getBalance('$1').then(b => formatEther(b))"
}

funding_wallet_balance() {
  yarn run --silent ethers --rpc "$RPC" --account "$FUNDING_PRIV_KEY" eval 'accounts[0].getBalance().then(b => formatEther(b))'
}

funding_wallet_address() {
  yarn run --silent ethers --rpc "$RPC" --account "$FUNDING_PRIV_KEY" eval 'accounts[0].getAddress().then(a => a)'
}

# $1=account (hex)
fund_if_empty() {
  echo "Starting funding wallet process"
  local FUNDING_WALLET_ADDRESS=$(funding_wallet_address)
  echo "Checking balance of funding wallet $FUNDING_WALLET_ADDRESS using RPC $RPC"
  local FUNDING_WALLET_BALANCE="$(funding_wallet_balance)"
  if [ "$FUNDING_WALLET_BALANCE" = '0.0' ]; then
    echo "Wallet $FUNDING_WALLET_ADDRESS has zero balance and cannot fund node $1"
  else
    echo "Funding wallet $FUNDING_WALLET_ADDRESS has enough: $FUNDING_WALLET_BALANCE"
    echo "Checking balance of the wallet $1 to be funded"
    local BALANCE="$(balance $1)"
    echo "Balance of $1 is $BALANCE"
    if [ "$BALANCE" = '0.0' ]; then
      echo "Funding account ... $RPC -> $1 $MIN_FUNDS"
      yarn ethers send --rpc "$RPC" --account "$FUNDING_PRIV_KEY" "$1" $MIN_FUNDS --yes
      sleep 60
    fi
  fi
}

# $1 = IP
get_eth_address(){
  curl "$1:3001/api/v1/address/eth"
}

# $1 = IP
get_hopr_address() {
  curl "$1:3001/api/v1/address/hopr"
}

# $1 = IP
# $2 = Hopr command
run_command(){
  curl --silent -X POST --data "$2" "$1:3001/api/v1/command"
}


# $1 = vm name
# $2 = docker image
update_if_existing() {
  if [[ $(gcloud_find_vm_with_name $1) ]]; then
    echo "Container exists, updating" 1>&2
    PREV=$(gcloud_get_image_running_on_vm $1)
    if [ "$PREV" == "$2" ]; then
      echo "Same version of image is currently running. Skipping update to $PREV" 1>&2
      return 0
    fi
    echo "Previous GCloud VM Image: $PREV"
    gcloud_update_container_with_image $1 $2 "$(disk_name $1)" "/app/db"
    sleep 60

    # prevent docker images overloading the disk space
    gcloud_cleanup_docker_images "$1"
  else
    echo "no container"
  fi

}

# $1 = vm name
# $2 = docker image
# $3 = OPTIONAL chain provider
# NB: --run needs to be at the end or it will ignore the other arguments.
start_testnode_vm() {
  local additional_flags=""
  if [ -n "${3:-}" ]; then
    additional_flags="--container-arg=--provider --container-arg=$RPC"
  fi
  if [ "$(update_if_existing $1 $2)" = "no container" ]; then
    gcloud compute instances create-with-container $1 $GCLOUD_DEFAULTS \
      --create-disk name=$(disk_name $1),size=10GB,type=pd-standard,mode=rw \
      --container-mount-disk mount-path="/app/db" \
      --container-env=^,@^DEBUG=hopr\*,@NODE_OPTIONS=--max-old-space-size=4096,@GCLOUD=1 \
      --container-image=$2 \
      --container-arg="--identity" --container-arg="/app/db/.hopr-identity" \
      --container-arg="--password" --container-arg="$BS_PASSWORD" \
      --container-arg="--init" --container-arg="true" \
      --container-arg="--announce" --container-arg="true" \
      --container-arg="--rest" --container-arg="true" \
      --container-arg="--restHost" --container-arg="0.0.0.0" \
      --container-arg="--healthCheck" --container-arg="true" \
      --container-arg="--healthCheckHost" --container-arg="0.0.0.0" \
      --container-arg="--admin" --container-arg="true" \
      --container-arg="--adminHost" --container-arg="0.0.0.0" \
      --container-arg="--run" --container-arg="\"cover-traffic start;daemonize\"" \
      --container-restart-policy=always \
      ${additional_flags}
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

# $1 network name
# $2 docker image
# $3 node number
# $4 = OPTIONAL chain provider
start_testnode() {
  local vm ip eth_address

  # start or update vm
  vm=$(vm_name "node-$3" $1)
  echo "- Starting test node $vm with $2 ${4:-}"
  start_testnode_vm $vm $2 ${4:-}

  # ensure node has funds, even after just updating a release
  ip=$(gcloud_get_ip "${vm}")
  wait_until_node_is_ready $ip
  eth_address=$(get_eth_address "${ip}")
  fund_if_empty "${eth_address}"
}

# $1 authorized keys file
add_keys() {
  if test -f "$1"; then
    echo "Reading keys from $1"
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

# $1 network name
# $2 number of nodes
# $3 docker image
# $4 = OPTIONAL chain provider
start_testnet() {
  for i in $(seq 1 $2);
  do
    echo "Start node $i"
    start_testnode $1 $3 $i ${4:-}
  done
  # @jose can you fix this pls.
  # add_keys scripts/keys/authorized_keys
}

