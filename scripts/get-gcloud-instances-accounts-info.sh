#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare -x HOPR_LOG_ID="get-gcloud-instances-accounts-info"
# shellcheck disable=SC1090
source "${mydir}/utils.sh"
# shellcheck disable=SC1090
source "${mydir}/gcloud.sh"
# shellcheck disable=SC1090
source "${mydir}/testnet.sh"

usage() {
  msg
  msg "This script can be used to get the accounts info (native and ERC20) for"
  msg "each node in the given gcloud instance group."
  msg
  msg "Usage: $0 <instance_group>"
  msg
  msg "where <instance_group>\t\tthe name of the GCP instance group"
  msg
  msg "Required environment variables"
  msg "------------------------------"
  msg
  msg "HOPRD_API_TOKEN\t\t\tused as api token for all nodes"
  msg
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

# verify and set parameters
: "${HOPRD_API_TOKEN?"Missing environment variable HOPRD_API_TOKEN"}"

declare instance_group="${1?"missing parameter <instance_group>"}"
declare api_token="${HOPRD_API_TOKEN}"
declare info_file="$(mktemp -q)"

function cleanup {
  local EXIT_CODE=$?

  trap - SIGINT SIGTERM ERR EXIT

  rm -f "${info_file}"

  exit $EXIT_CODE
}
trap cleanup SIGINT SIGTERM ERR EXIT

# get IPs of newly started VMs which run hoprd
declare node_ips
node_ips=$(gcloud_get_managed_instance_group_instances_ips "${instance_group}")
declare node_ips_arr=( ${node_ips} )

log "Gcloud machine information for cluster: ${instance_group}"
echo "HOPR ADDRESS,NATIVE ADDRESS,IP" > "${info_file}"

declare native_address hopr_address
for ip in ${node_ips}; do
  native_address="$(get_native_address "${api_token}@${ip}:3001")"
  hopr_address="$(get_hopr_address "${api_token}@${ip}:3001")"

  echo "${hopr_address},${native_address},${ip}"
done >> "${info_file}"

column -t -s',' "${info_file}"
