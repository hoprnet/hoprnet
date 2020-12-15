#!/bin/bash
set -e #u
shopt -s expand_aliases

source scripts/cd/gcloud.sh
source scripts/cd/environments.sh

# ---- CONTINUOUS DEPLOYMENT: Start a bootstrap server -----

# ENV Variables:
# - RELEASE: release version, ie. `1.51.1-next.0`
# - GITHUB_REF: ie. `/refs/heads/mybranch`
# - RPC: provider address, ie `https://rpc-mainnet.matic.network`
# - FUNDING_PRIV_KEY: funding private key, raw
# - BS_PASSWORD: database password

MIN_FUNDS=0.01291
HOPRD_ARGS="--data='/app/db/ethereum/testnet/bootstrap' --password='$BS_PASSWORD'"
ZONE="--zone=europe-west6-a"


hoprd_image() {
  # For example ...hoprd:1.0.1-next-1234
  echo "gcr.io/hoprassociation/hoprd:$RELEASE"
}

gcloud_disk_name() {
  # NB: needs to be short
  echo "bs-$VERSION_MAJ_MIN-$RELEASE_NAME"
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
  local ADDR=$(curl $1:3001/api/v1/address/eth)
  echo $ADDR
}

# $1 = IP
get_hopr_address() {
  local ADDR=$(curl $1:3001/api/v1/address/hopr)
  echo $ADDR
}


# $1 = vm name
update_or_create_bootstrap_vm() {
  if [[ $(gcloud_find_vm_with_name $1) ]]; then
    echo "Container exists, updating"
    PREV=$(gcloud_get_image_running_on_vm $1)
    if [ "$PREV" == "$(hoprd_image)" ]; then 
      echo "Same version of image is currently running. Skipping update to $PREV"
      return 0
    fi
    echo "Previous GCloud VM Image: $PREV"
    gcloud_update_container_with_image $1 $(hoprd_image) $(gcloud_disk_name) "/app/db"
    sleep 60
  else
    echo "No container found, creating $1"
    local ip=$(gcloud_get_address $1)
    gcloud compute instances create-with-container $1 $ZONE \
      --machine-type=e2-medium \
      --network-interface=address=$ip,network-tier=PREMIUM,subnet=default \
      --metadata=google-logging-enabled=true --maintenance-policy=MIGRATE \
      --create-disk name=$(gcloud_disk_name),size=10GB,type=pd-ssd,mode=rw \
      --container-mount-disk mount-path="/app/db" \
      --tags=hopr-node,web-client,rest-client,portainer \
      --boot-disk-size=10GB --boot-disk-type=pd-standard \
      --container-env=^,@^DEBUG=hopr\*,@NODE_OPTIONS=--max-old-space-size=4096 \
      --container-image=$(hoprd_image) \
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

start_bootstrap() {
  get_environment
  local vm_name=$(gcloud_vm_name bootstrap)
  local ip=$(gcloud_get_address $vm_name)

  echo "Starting bootstrap server for r:$RELEASE_NAME at $ip"
  echo "- Release Version: $RELEASE"
  echo "- Release IP: $ip"
  echo "- Release Name: $RELEASE_NAME"
  echo "- GCloud VM name: $vm_name"

  update_or_create_bootstrap_vm $vm_name

  #GCLOUD_VM_DISK=/mnt/disks/gce-containers-mounts/gce-persistent-disks/$(gcloud_disk_name)
  BOOTSTRAP_ETH_ADDRESS=$(get_eth_address $ip)
  BOOTSTRAP_HOPR_ADDRESS=$(get_hopr_address $ip)

  echo "Bootstrap Server ETH Address: $BOOTSTRAP_ETH_ADDRESS"
  echo "Bootstrap Server HOPR Address: $BOOTSTRAP_HOPR_ADDRESS"

  fund_if_empty $BOOTSTRAP_ETH_ADDRESS
}

