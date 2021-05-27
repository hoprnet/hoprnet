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
  yarn ethers eval "new ethers.providers.JsonRpcProvider('$RPC').getBalance('$1').then(b => formatEther(b))"
}

# $1=account (hex)
fund_if_empty() {
  echo "Checking balance of $1"
  local BALANCE="$(balance $1)"
  echo "Balance is $BALANCE"
  if [ "$BALANCE" = '0.0' ]; then
    echo "Funding account ... $RPC -> $1 $MIN_FUNDS"
    yarn ethers send --rpc "$RPC" --account "$FUNDING_PRIV_KEY" "$1" $MIN_FUNDS --yes
    sleep 60
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
  else
    echo "no container"
  fi

}

# $1 = vm name
# $2 = docker image
# NB: --run needs to be at the end or it will ignore the other arguments.
start_testnode_vm() {
  if [ "$(update_if_existing $1 $2)" = "no container" ]; then
    gcloud compute instances create-with-container $1 $GCLOUD_DEFAULTS \
      --create-disk name=$(disk_name $1),size=10GB,type=pd-standard,mode=rw \
      --container-mount-disk mount-path="/app/db" \
      --container-env=^,@^DEBUG=hopr\*,@NODE_OPTIONS=--max-old-space-size=4096,@GCLOUD=1 \
      --container-image=$2 \
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

# $1 network name
# $2 docker image
# $3 node number
# $4 = OPTIONAL chain provider
start_testnode() {
  local vm=$(vm_name "node-$3" $1)
  echo "- Starting test node $vm with $2 ${4:-}"
  start_testnode_vm $vm $2 ${4:-}
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

