#!/bin/bash
set -e #u
shopt -s expand_aliases


# ---- CONTINUOUS DEPLOYMENT: Start a bootstrap server -----

# ENV Variables:
# - RELEASE: release version, ie. 1.51.1-next.0
# - GITHUB_REF: ie. /refs/heads/mybranch
# - RPC: provider address, ie https://rpc-mainnet.matic.networuk
# - FUNDING_PRIV_KEY: funding private key, raw
# - BS_PASSWORD: database password${{ secrets.BS_PASSWORD }} \

# GCLOUD_VM_DISK=${{ env.GCLOUD_VM_DISK }} \

MIN_FUNDS=0.01291
HOPRD_IMAGE="gcr.io/hoprassociation/hoprd:$RELEASE_VERSION"
HOPRD_ARGS="--data='/app/db/ethereum/testnet/bootstrap' --password='$BS_PASSWORD'"
DOCKER_ARGS="-v $GCLOUD_VM_DISK:/app/db --entrypoint=node -it $HOPRD_IMAGE"

# ------ Aliases -------
alias wallet="ethers --rpc $RPC --account $FUNDING_PRIV_KEY"
alias gssh="gcloud compute ssh --ssh-flag='-t' --zone=europe-west6-a"

# $1=version string, semver
function get_version_maj_min() {
  # From https://github.com/cloudflare/semver_bash/blob/master/semver.sh
  local RE='[^0-9]*\([0-9]*\)[.]\([0-9]*\)[.]\([0-9]*\)\([0-9A-Za-z-]*\)'
  local MAJ=$(echo "$1" | sed -e "s#$RE#\1#")
  local MIN=$(echo "$1" | sed -e "s#$RE#\2#")
  echo "$MAJ.$MIN"
}

# ===== Load env variables for the current github ref =====
# Takes:
# - GITHUB_REF
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

    echo "Unknown version: $VERSION_MAJ_MIN"
  esac

  echo "Unknown release / environment: '$BRANCH'"
  #RELEASE_NAME=debug
  #RELEASE_IP="34.65.56.229"
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
  gssh $GCLOUD_VM_NAME -- docker run $DOCKER_ARGS index.js $HOPRD_ARGS --runAsBootstrap run 'myAddress native'
}

get_hopr_address() {
  gssh $GCLOUD_VM_NAME -- docker run $DOCKER_ARGS index.js $HOPRD_ARGS --runAsBootstrap --run 'myAddress hopr'
}

start_bootstrap() {
  get_environment
  echo "Starting bootstrap server for r:$RELEASE_NAME at $RELEASE_IP"

  GCLOUD_VM_NAME="$RELEASE_NAME-bootstrap"
  echo "VM: $GCLOUD_VM_NAME"
  exit 0
  BOOTSTRAP_ETH_ADDRESS=$(get_eth_address)
  BOOTSTRAP_HOPR_ADDRESS=$(get_hopr_address)

  echo "Bootstrap Server ETH Address: $BOOTSTRAP_ETH_ADDRESS"
  echo "Bootstrap Server HOPR Address: $BOOTSTRAP_HOPR_ADDRESS"

  fund_if_empty $BOOTSTRAP_ADDRESS

  # Restart bootstrap server virtual machine to restart main container
  gcloud compute instances reset --zone=europe-west6-a ${GCLOUD_VM_NAME }
  echo "Bootstrap multiaddr: /${ RELEASE_IP }/tcp/9091/p2p/$BOOTSTRAP_HOPR_ADDRESS"
}

start_bootstrap
