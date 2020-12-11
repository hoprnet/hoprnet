#!/bin/bash
set -e #u
shopt -s expand_aliases

source scripts/cd/gcloud.sh

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

# ------ Aliases -------
alias wallet="ethers --rpc $RPC --account $FUNDING_PRIV_KEY"
alias gssh="gcloud compute ssh --ssh-flag='-t' $ZONE"

# $1=version string, semver
function get_version_maj_min() {
  # From https://github.com/cloudflare/semver_bash/blob/master/semver.sh
  local RE='[^0-9]*\([0-9]*\)[.]\([0-9]*\)[.]\([0-9]*\)\([0-9A-Za-z-]*\)'
  local MAJ=$(echo "$1" | sed -e "s#$RE#\1#")
  local MIN=$(echo "$1" | sed -e "s#$RE#\2#")
  echo "$MAJ.$MIN"
}

hoprd_image() {
  # For example ...hoprd:1.0.1-next-1234
  echo "gcr.io/hoprassociation/hoprd:$RELEASE"
}

gcloud_vm_name() {
  # For example, larnaca-bootstrap, master-bootstrap
  echo "$RELEASE_NAME-bootstrap"
}

gcloud_disk_name() {
  echo "bs-$RELEASE_NAME"
}

# ===== Load env variables for the current github ref =====
# Takes:
# - GITHUB_REF
# - RELEASE
# Sets: 
# - RELEASE_NAME
# - RELEASE_IP
get_environment() {
  BRANCH=$(echo "$GITHUB_REF" | sed -e "s#refs/heads/##g") # Removing `refs/heads`

  if [ "$BRANCH" == 'master' ]; then
    RELEASE_NAME='master'
    RELEASE_IP='34.65.102.152'
    return
  fi

  case "$BRANCH" in release/*)
    VERSION_MAJ_MIN=$(get_version_maj_min $RELEASE) 
    if [ "$VERSION_MAJ_MIN" == '1.58' ]; then
      RELEASE_NAME='queretaro'
      RELEASE_IP='34.65.207.39'
      return
    fi

    if [ "$VERSION_MAJ_MIN" == '1.57' ]; then
      RELEASE_NAME='larnaca'
      RELEASE_IP='unknown'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.56' ]; then
      RELEASE_NAME='luzern'
      RELEASE_IP='34.65.5.42'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.55' ]; then
      RELEASE_NAME='zug'
      RELEASE_IP='34.65.158.118'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.54' ]; then
      RELEASE_NAME='zurich'
      RELEASE_IP='unknown'
      return
    fi

    if [ "$VERSION_MAJ_MIN" == '1.0' ]; then
      RELEASE_NAME='debug'
      RELEASE_IP='34.65.56.229'
      return
    fi

    echo "Unknown version: $VERSION_MAJ_MIN"
  esac

  echo "Unknown release / environment: '$BRANCH'"
  exit 1
}


# $1=account (hex)
balance() {
  wallet eval "new ethers.providers.JsonRpcProvider('$RPC').getBalance('$1').then(b => formatEther(b))"
}

# $1=account (hex)
fund_if_empty() {
  echo "Checking balance of $1"
  local BALANCE="$(balance $1)"
  echo "Balance is $BALANCE"
  if [$BALANCE = '0.0']; then
    echo "Funding account ..."
    wallet send $1 $MIN_FUNDS --yes
    sleep 1m
  fi
}

get_eth_address(){
  DOCKER_ARGS="-v $GCLOUD_VM_DISK:/app/db --entrypoint=node -it $(hoprd_image)"
  gssh $(gcloud_vm_name) -- docker run $DOCKER_ARGS index.js $HOPRD_ARGS --runAsBootstrap run 'myAddress native'
}

get_hopr_address() {
  DOCKER_ARGS="-v $GCLOUD_VM_DISK:/app/db --entrypoint=node -it $(hoprd_image)"
  gssh $(gcloud_vm_name) -- docker run $DOCKER_ARGS index.js $HOPRD_ARGS --runAsBootstrap --run 'myAddress hopr'
}


gcloud_update() {
  if [[ $(gcloud compute instances list | grep $(gcloud_vm_name)) ]]; then
    echo "Container exists, updating"
    local GCLOUD_VM_IMAGE=$(gcloud compute instances describe $(gcloud_vm_name) $ZONE --format='value[](metadata.items.gce-container-declaration)' | grep image | tr -s ' ' | cut -f3 -d' ')
    echo "Previous GCloud VM Image: $GCLOUD_VM_IMAGE"
    update_container_with_image $(hoprd_image) $(gcloud_disk_name)
  else
    echo "No container found, creating"
    create_instance_with_image $(hoprd_image)
  fi
}

# $1 = container-image
# $2 = disk name
update_container_with_image() {
  echo "Updating container $1 $2"
  gcloud compute instances update-container $(gcloud_vm_name) $ZONE \
    --container-image=$1 --container-mount-disk name=$2,mount-path="/app/db"
  sleep 30s
}


# $1 = container-image 
create_instance_with_image() {
  gcloud compute instances create-with-container $(gcloud_vm_name) }} $ZONE \
    --machine-type=e2-medium \
    --network-interface=address=$RELEASE_IP,network-tier=PREMIUM,subnet=default \
    --metadata=google-logging-enabled=true --maintenance-policy=MIGRATE \
    --create-disk name=$(gcloud_disk_name),size=10GB,type=pd-ssd,mode=rw \
    --container-mount-disk mount-path="/app/db" \
    --tags=hopr-node,web-client,portainer \
    --boot-disk-size=10GB --boot-disk-type=pd-standard \
    --container-env=^,@^DEBUG=hopr\*,@NODE_OPTIONS=--max-old-space-size=4096 \
    --container-image=$(hoprd_image) \
    --container-arg="--password" --container-arg="$BS_PASSWORD" \
    --container-arg="--init" --container-arg="true" \
    --container-arg="--runAsBootstrap" --container-arg="true" \
    --container-arg="--admin" \
    --container-restart-policy=always
  sleep 2m
}


start_bootstrap() {
  get_environment

  echo "Starting bootstrap server for r:$RELEASE_NAME at $RELEASE_IP"
  echo "Release Version: $RELEASE"
  echo "Release IP: $RELEASE_IP"
  echo "Release Name: $RELEASE_NAME"
  echo "GCloud VM name: $(gcloud_vm_name)"

  gcloud_update
  GCLOUD_VM_DISK=/mnt/disks/gce-containers-mounts/gce-persistent-disks/$(gcloud_disk_name)

  #Stop bootstrap node to get the address from the database
  gcloud_stop $(gcloud_vm_name) $(hoprd_image)

  BOOTSTRAP_ETH_ADDRESS=$(get_eth_address)
  BOOTSTRAP_HOPR_ADDRESS=$(get_hopr_address)

  echo "Bootstrap Server ETH Address: $BOOTSTRAP_ETH_ADDRESS"
  echo "Bootstrap Server HOPR Address: $BOOTSTRAP_HOPR_ADDRESS"

  fund_if_empty $BOOTSTRAP_ADDRESS

  # Restart bootstrap server virtual machine to restart main container
  gcloud compute instances reset $ZONE ${(gcloud_vm_name) }
  echo "Bootstrap multiaddr: /${ RELEASE_IP }/tcp/9091/p2p/$BOOTSTRAP_HOPR_ADDRESS"
}

start_bootstrap
