#!/bin/bash
set -e #u

if [ -z "$GCLOUD_INCLUDED" ]; then
  source scripts/gcloud.sh 
  source scripts/dns.sh
fi

MIN_FUNDS=0.01291
HOPRD_ARGS="--data='/app/db/ethereum/testnet/bootstrap' --password='$BS_PASSWORD'"
ZONE="--zone=europe-west6-a"

# $1 = role (ie. bootstrap, node-4)
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
  ethers eval "new ethers.providers.JsonRpcProvider('$RPC').getBalance('$1').then(b => formatEther(b))"
}

# $1=account (hex)
fund_if_empty() {
  echo "Checking balance of $1"
  local BALANCE="$(balance $1)"
  echo "Balance is $BALANCE"
  if [ "$BALANCE" = '0.0' ]; then
    echo "Funding account ... $RPC -> $1 $MIN_FUNDS"
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
update_or_create_bootstrap_vm() {
  if [ "$(update_if_existing $1 $2)" = "no container" ]; then
    echo "No container found, creating $1"
    local ip=$(gcloud_get_address $1)
    gcloud compute instances create-with-container $1 $GCLOUD_DEFAULTS \
      --network-interface=address=$ip,network-tier=PREMIUM,subnet=default \
      --create-disk name=$(disk_name $1),size=10GB,type=pd-balanced,mode=rw \
      --container-mount-disk mount-path="/app/db" \
      --container-env=^,@^DEBUG=hopr\*,@NODE_OPTIONS=--max-old-space-size=4096,GCLOUD=1 \
      --container-image=$2 \
      --container-arg="--password" --container-arg="$BS_PASSWORD" \
      --container-arg="--init" --container-arg="true" \
      --container-arg="--runAsBootstrap" --container-arg="true" \
      --container-arg="--rest" --container-arg="true" \
      --container-arg="--restHost" --container-arg="0.0.0.0" \
      --container-arg="--healthCheck" --container-arg="true" \
      --container-arg="--healthCheckHost" --container-arg="0.0.0.0" \
      --container-arg="--admin" --container-arg="true" \
      --container-arg="--adminHost" --container-arg="0.0.0.0" \
      --container-arg="--run" --container-arg="\"settings strategy passive;daemonize\""
      --container-restart-policy=always
    sleep 120
  fi
}

# $1 = vm name
# $2 = docker image
# $3 = BS multiaddr
# NB: --run needs to be at the end or it will ignore the other arguments.
start_testnode_vm() {
  if [ "$(update_if_existing $1 $2)" = "no container" ]; then
    gcloud compute instances create-with-container $1 $GCLOUD_DEFAULTS \
      --create-disk name=$(disk_name $1),size=10GB,type=pd-standard,mode=rw \
      --container-mount-disk mount-path="/app/db" \
      --container-env=^,@^DEBUG=hopr\*,@NODE_OPTIONS=--max-old-space-size=4096,GCLOUD=1 \
      --container-image=$2 \
      --container-arg="--password" --container-arg="$BS_PASSWORD" \
      --container-arg="--init" --container-arg="true" \
      --container-arg="--rest" --container-arg="true" \
      --container-arg="--restHost" --container-arg="0.0.0.0" \
      --container-arg="--healthCheck" --container-arg="true" \
      --container-arg="--healthCheckHost" --container-arg="0.0.0.0" \
      --container-arg="--bootstrapServers" --container-arg="$3" \
      --container-arg="--admin" --container-arg="true" \
      --container-arg="--adminHost" --container-arg="0.0.0.0" \
      --container-arg="--run" --container-arg="\"cover-traffic start;daemonize\"" \
      --container-restart-policy=always
  fi
}

# $1 network name
# $2 docker image
start_bootstrap() {
  local vm=$(vm_name bootstrap $1)
  echo "- Starting bootstrap server for $1 at ($vm) with $2" 1>&2
  local ip=$(gcloud_get_address $vm)
  echo "- public ip for bootstrap server: $ip" 1>&2
  update_or_create_bootstrap_vm $vm $2 1>&2
  BOOTSTRAP_ETH_ADDRESS=$(get_eth_address $ip)
  BOOTSTRAP_HOPR_ADDRESS=$(get_hopr_address $ip)
  echo "- Bootstrap Server ETH Address: $BOOTSTRAP_ETH_ADDRESS" 1>&2
  echo "- Bootstrap Server HOPR Address: $BOOTSTRAP_HOPR_ADDRESS" 1>&2
  fund_if_empty $BOOTSTRAP_ETH_ADDRESS 1>&2
  local multiaddr="/ip4/$ip/tcp/9091/p2p/$BOOTSTRAP_HOPR_ADDRESS"
  local release=$(echo $2 | cut -f2 -d:)
  echo "- Bootstrap Release: $release"
  echo "- Bootstrap Multiaddr value: $multiaddr"
  local txt_record=$(gcloud_txt_record $release bootstrap $multiaddr)
  echo "- DNS entry: $(gcloud_dns_entry $release bootstrap)"
  echo "- TXT record: $txt_record"
}

# $1 network name
# $2 docker image
# $3 node number
# $4 bootstrap multiaddr
start_testnode() {
  local vm=$(vm_name "node-$3" $1)
  echo "- Starting test node $vm with $2, bs: $4"
  start_testnode_vm $vm $2 $4
}

# Usage 
# $1 authorized keys file
add_keys() {
  if test -f "$1"; then
    echo "Reading keys from $1"
    xargs -a $1 -I {} gcloud compute os-login ssh-keys add --key="{}"
  else
    echo "Authorized keys file not found"
    exit 1
  fi
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
  bs_addr=$(start_bootstrap $1 $3)
  echo "- bootstrap addr: $bs_addr"

  for i in $(seq 2 $2);
  do
    echo "Start node $i"
    start_testnode $1 $3 $i $bs_addr
  done
  add_keys scripts/keys/authorized_keys
}

