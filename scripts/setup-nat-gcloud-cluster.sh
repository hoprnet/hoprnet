#!/usr/bin/env bash

# TODO: this script currently uses goerli as the RPC provider. However, it
# should be extended to use its own instance of hardhat too.

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare -x HOPR_LOG_ID="setup-nat-gcloud-cluster"
source "${mydir}/utils.sh"
source "${mydir}/gcloud.sh"
source "${mydir}/testnet.sh"

usage() {
  msg
  msg "This script can be used to setup a cluster of nodes behind NAT on gcloud and run"
  msg "an initial setup script against these nodes. Once testing has"
  msg "completed the script can be used to cleanup the cluster as well."
  msg
  msg "Usage: $0 <environment> [<init_script> [<cluster_id> [<docker_image>]]]"
  msg
  msg "where <environment>\t\tthe environment from which the smart contract addresses are derived"
  msg "      <init_script>\t\tpath to a script which is called with all node API endpoints as parameters"
  msg "      <cluster_id>\t\tuses a random value as default"
  msg "      <docker_image>\t\tuses 'gcr.io/hoprassociation/hoprd-nat:<environment>' as default"
  msg
  msg "Required environment variables"
  msg "------------------------------"
  msg
  msg "FUNDING_PRIV_KEY\t\tsets the account which is used to fund nodes"
  msg
  msg "Optional environment variables"
  msg "------------------------------"
  msg
  msg "HOPRD_API_TOKEN\t\t\tused as api token for all nodes, defaults to a random value"
  msg "HOPRD_PASSWORD\t\t\tused as password for all nodes, defaults to a random value"
  msg "HOPRD_SHOW_PRESTART_INFO\tset to 'true' to print used parameter values before starting"
  msg "HOPRD_PERFORM_CLEANUP\t\tset to 'true' to perform the cleanup process for the given cluster id"
  msg
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

# verify and set parameters
: ${FUNDING_PRIV_KEY?"Missing environment variable FUNDING_PRIV_KEY"}

declare environment="${1?"missing parameter <environment>"}"
declare init_script=${2:-}
declare cluster_id="${3:-${environment}-nat-${RANDOM}-${RANDOM}}"
declare docker_image=${4:-gcr.io/hoprassociation/hoprd-nat:${environment}}

declare api_token="${HOPRD_API_TOKEN:-Token${RANDOM}%${RANDOM}%${RANDOM}Token}"
declare password="${HOPRD_PASSWORD:-pw${RANDOM}${RANDOM}${RANDOM}pw}"
declare perform_cleanup="${HOPRD_PERFORM_CLEANUP:-false}"
declare show_prestartinfo="${HOPRD_SHOW_PRESTART_INFO:-false}"

function cleanup {
  local EXIT_CODE=$?

  trap - SIGINT SIGTERM ERR EXIT
  set +Eeuo pipefail

  if [ ${EXIT_CODE} -ne 0 ] || [ "${perform_cleanup}" = "true" ] || [ "${perform_cleanup}" = "1" ]; then
    # Cleaning up everything upon failure
    gcloud_delete_managed_instance_group "${cluster_id}"
    gcloud_delete_instance_template "${cluster_id}"
  fi

  exit $EXIT_CODE
}

if [ "${perform_cleanup}" = "1" ] || [ "${perform_cleanup}" = "true" ]; then
  cleanup

  # exit right away
  exit
fi

# --- Log test info {{{
if [ "${show_prestartinfo}" = "1" ] || [ "${show_prestartinfo}" = "true" ]; then
  log "Pre-Start Info"
  log "\tdocker_image: ${docker_image}"
  log "\tcluster_id: ${cluster_id}"
  log "\tinit_script: ${init_script}"
  log "\tenvironment: ${environment}"
  log "\tapi_token: ${api_token}"
  log "\tpassword: ${password}"
  log "\tperform_cleanup: ${perform_cleanup}"
  log "\tshow_prestartinfo: ${show_prestartinfo}"
fi
# }}}

# create test specific instance template
gcloud_create_or_update_instance_template \
  "${cluster_id}" \
  "${docker_image}" \
  "${environment}" \
  "${api_token}" \
  "${password}"

# start nodes
gcloud_create_or_update_managed_instance_group  \
  "${cluster_id}" \
  1 \
  "${cluster_id}"

# get IPs of newly started VMs which run hoprd
declare node_ips
node_ips=$(gcloud_get_managed_instance_group_instances_ips "${cluster_id}")
declare node_ips_arr=( ${node_ips} )

#  --- Fund nodes --- {{{
declare eth_address
for ip in ${node_ips}; do
  wait_until_node_is_ready "${ip}"
  eth_address=$(get_native_address "${ip}:3001")
  fund_if_empty "${eth_address}" "${environment}"
done

for ip in ${node_ips}; do
  wait_for_port "3001" "${ip}"
done
# }}}

# --- Call init script--- {{{
if [ -n "${init_script}" ] && [ -x "${init_script}" ]; then
  HOPRD_API_TOKEN="${api_token}" \
    "${init_script}" \
    ${node_ips_arr[@]/%/:3001}
fi
# }}}

log "finished"
