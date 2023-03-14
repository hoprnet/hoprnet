#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare -x HOPR_LOG_ID="register-node"
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 <account> <node_api>"
  msg
  msg "Required environment variables"
  msg "------------------------------"
  msg
  msg "HOPRD_API_TOKEN\t\t\tused as api token for all nodes"
  msg "ACCOUNT_PRIVKEY\t\t\tused the private key of the staking account"
  msg "DEV_BANK_PRIVKEY\t\t\tused as the private key of the dev bank"
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
test -z "${1:-}" && { msg "Missing account address"; usage; exit 1; }
test -z "${HOPRD_API_TOKEN:-}" && { msg "Missing HOPRD_API_TOKEN"; usage; exit 1; }
test -z "${ACCOUNT_PRIVKEY:-}" && { msg "Missing ACCOUNT_PRIVKEY"; usage; exit 1; }
test -z "${DEV_BANK_PRIVKEY:-}" && { msg "Missing DEV_BANK_PRIVKEY"; usage; exit 1; }

declare account="${1}"
# node_api defaults to 127.0.0.1:3001
declare node_api="${2:-127.0.0.1:3001}"

declare api_token=${HOPRD_API_TOKEN}
declare account_privkey="${ACCOUNT_PRIVKEY:-}"
declare dev_bank_privkey="${DEV_BANK_PRIVKEY:-}"
# Get node's peer
declare peer_id="$(get_hopr_address "${api_token}@${node_api}")"

# Request a Network_registry (developer) NFT from DevBank
PRIVATE_KEY="${dev_bank_privkey}" make request-nrnft \
    environment=master-staging \
    environment_type=staging \
    nftrank=developer \
    recipient="${account}"

# Stake NFT
PRIVATE_KEY="${account_privkey}" make stake-nrnft \
    environment=master-staging \ 
    environment_type=staging \
    nftrank=developer

# Register "Peer ID" on Network Registry
PRIVATE_KEY="${account_privkey}" make self-register-node \
    environment=master-staging \
    environment_type=staging \
    peer_ids=${peer_id}"
