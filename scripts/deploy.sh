#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

usage() {
  echo >&2
  echo "Usage: $0 <package-name> <package-version> [<wait-interval>]" >&2
  echo >&2
  echo -e "\twhere <wait-interval> is in seconds, default is 5" >&2
  echo >&2
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }


# verify and set parameters

declare mydir

mydir=$(dirname $(readlink -f $0))

# set log id and use shared log function for readable logs
declare HOPR_LOG_ID="deploy"
source "${mydir}/lib/utils.sh"

# source gcloud functions and environments info
source "${mydir}/lib/gcloud.sh"
source "${mydir}/lib/environments.sh"

# ---- On Deployment -----
#
# This is run on pushes to master, or release/**
#
# ENV Variables:
# - RPC: provider address, ie `https://rpc-mainnet.matic.network`
# - FUNDING_PRIV_KEY: funding private key, raw
# - BS_PASSWORD: database password

if [ -z "${RPC:-}" ]; then
  RPC=https://eth-goerli.gateway.pokt.network/v1/6021a2b6928ff9002e6c7f2f
fi

# Get version from package.json
RELEASE=$(node -p -e "require('./packages/hoprd/package.json').version")

# Get RELEASE_NAME, from environment
get_environment

TESTNET_NAME="$RELEASE_NAME-$(echo "$VERSION_MAJ_MIN" | sed 's/\./-/g')"
TESTNET_SIZE=3

log "Cleaning up before deploy"
./cleanup.sh

log "Starting testnet '$TESTNET_NAME' with $TESTNET_SIZE nodes and image hoprd:$RELEASE"
start_testnet $TESTNET_NAME $TESTNET_SIZE "gcr.io/hoprassociation/hoprd:$RELEASE"
