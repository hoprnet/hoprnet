#!/bin/bash
set -e #u

source scripts/cd/gcloud.sh

MIN_FUNDS=0.01291
HOPRD_ARGS="--data='/app/db/ethereum/testnet/bootstrap' --password='$BS_PASSWORD'"
ZONE="--zone=europe-west6-a"

# $1 = role (ie. bootstrap)
# $2 = network name
vm_name() {
  echo "$1-$2"
}

# $1 = vm name
disk_name() {
  echo "$1-dsk"
}

# $1=account (hex)
balance() {
  ethers eval "new ethers.providers.JsonRpcProvider('$RPC').getBalance('$1').then(b => formatEther(b))"
}

# $1=account (hex)
fund_if_empty() {
  echo "Checking balance of $1"
  local BALANCE="$(balance $1)"
  echo "Balance is $BALANCE"
  if [ "$BALANCE" = '0.0' ]; then
    echo "Funding account ..."
    ethers send --rpc "$RPC" --account "$FUNDING_PRIV_KEY" "$1" $MIN_FUNDS --yes
    sleep 60
  fi
}

# $1 = IP
get_eth_address(){
  echo $(curl $1:3001/api/v1/address/eth)
}

# $1 = IP
get_hopr_address() {
  echo $(curl $1:3001/api/v1/address/hopr)
}

# $1 = vm name
# $2 = docker image
update_or_create_bootstrap_vm() {
  if [[ $(gcloud_find_vm_with_name $1) ]]; then
    echo "Container exists, updating"
    PREV=$(gcloud_get_image_running_on_vm $1)
    if [ "$PREV" == "$2" ]; then 
      echo "Same version of image is currently running. Skipping update to $PREV"
      return 0
    fi
    echo "Previous GCloud VM Image: $PREV"
    gcloud_update_container_with_image $1 $2 "$(disk_name $1)" "/app/db"
    sleep 60
  else
    echo "No container found, creating $1"
    local ip=$(gcloud_get_address $1)
    gcloud compute instances create-with-container $1 $ZONE \
      --machine-type=e2-medium \
      --network-interface=address=$ip,network-tier=PREMIUM,subnet=default \
      --metadata=google-logging-enabled=true --maintenance-policy=MIGRATE \
      --create-disk name=$(disk_name $1),size=10GB,type=pd-ssd,mode=rw \
      --container-mount-disk mount-path="/app/db" \
      --tags=hopr-node,web-client,rest-client,portainer \
      --boot-disk-size=10GB --boot-disk-type=pd-standard \
      --container-env=^,@^DEBUG=hopr\*,@NODE_OPTIONS=--max-old-space-size=4096 \
      --container-image=$2 \
      --container-arg="--password" --container-arg="$BS_PASSWORD" \
      --container-arg="--init" --container-arg="true" \
      --container-arg="--runAsBootstrap" --container-arg="true" \
      --container-arg="--rest" --container-arg="true" \
      --container-arg="--restHost" --container-arg="0.0.0.0" \
      --container-arg="--admin" \
      --container-restart-policy=always
    sleep 120
  fi
}

# $1 network name
# $2 docker image
start_bootstrap() {
  local vm=$(vm_name bootstrap $1)
  echo "- Starting bootstrap server for $1 at ($vm) with $2"
  local ip=$(gcloud_get_address $vm)
  echo "- public ip for bootstrap server: $ip"
  update_or_create_bootstrap_vm $vm $2
  BOOTSTRAP_ETH_ADDRESS=$(get_eth_address $ip)
  BOOTSTRAP_HOPR_ADDRESS=$(get_hopr_address $ip)
  echo "- Bootstrap Server ETH Address: $BOOTSTRAP_ETH_ADDRESS"
  echo "- Bootstrap Server HOPR Address: $BOOTSTRAP_HOPR_ADDRESS"
  fund_if_empty $BOOTSTRAP_ETH_ADDRESS
}

# ----- Start Testnet -------
#
# Using a standard naming scheme, based on a name, we
# either update or start VM's to create a network of
# N nodes, including a bootstrap node running on a public
# IP
#
# $1 network name
# $2 number of nodes
# $3 docker image
start_testnet() {
  # First node is always bootstrap
  start_bootstrap $1 $3

  for i in $(seq 2 $2);
  do
    echo "Start node $i"
  done
}

