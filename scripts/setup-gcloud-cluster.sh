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
declare -x HOPR_LOG_ID="setup-gcloud-cluster"
source "${mydir}/utils.sh"
source "${mydir}/gcloud.sh"
source "${mydir}/testnet.sh"

usage() {
  msg
  msg "This script can be used to setup a cluster of nodes on gcloud and run"
  msg "an initial setup script against these nodes. Once testing has"
  msg "completed the script can be used to cleanup the cluster as well."
  msg
  msg "Usage: $0 [<cluster_id> [<docker_image> [<init_script>]]]"
  msg
  msg "where <cluster_id>:\t\tuses a random value as default"
  msg "      <docker_image>:\t\tuses 'gcr.io/hoprassociation/hoprd:latest' as default"
  msg "      <init_script>:\t\tpath to a script which is called with all node API endpoints as parameters"
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
  msg "HOPRD_PROVIDER\t\t\tused as provider for all nodes, defaults to infura/goerli"
  msg "              \t\t\twhich requires the additional env var HOPRD_INFURA_KEY to be set"
  msg "HOPRD_SHOW_PRESTART_INFO\tset to 'true' to print used parameter values before starting"
  msg "HOPRD_PERFORM_CLEANUP\t\tset to 'true' to perform the cleanup process for the given cluster id"
  msg "HOPRD_TOKEN_CONTRACT\t\tused to fund HOPR token, defaults to 0x566a5c774bb8ABE1A88B4f187e24d4cD55C207A5"
  msg
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

# verify and set parameters
declare cluster_id="${1:-custom-cluster-${RANDOM}-${RANDOM}}"
declare docker_image=${2:-gcr.io/hoprassociation/hoprd:latest}
declare init_script=${3:-}

declare api_token="${HOPRD_API_TOKEN:-Token${RANDOM}%${RANDOM}%${RANDOM}Token}"
declare password="${HOPRD_PASSWORD:-pw${RANDOM}${RANDOM}${RANDOM}pw}"
declare provider="${HOPRD_PROVIDER:-https://goerli.infura.io/v3/${HOPRD_INFURA_KEY}}"
declare hopr_token_contract="${HOPRD_TOKEN_CONTRACT:-0x566a5c774bb8ABE1A88B4f187e24d4cD55C207A5}"
declare perform_cleanup="${HOPRD_PERFORM_CLEANUP:-false}"
declare show_prestartinfo="${HOPRD_SHOW_PRESTART_INFO:-false}"

test -z "${FUNDING_PRIV_KEY:-}" && { msg "Missing FUNDING_PRIV_KEY"; usage; exit 1; }

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

function fund_ip() {
  local ip="${1}"
  local eth_address

  wait_until_node_is_ready "${ip}"
  eth_address=$(get_eth_address "${ip}")
  fund_if_empty "${eth_address}" "${provider}" "${hopr_token_contract}"
  wait_for_port "9091" "${ip}"
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
  log "\tapi_token: ${api_token}"
  log "\tpassword: ${password}"
  log "\tprovider: ${provider}"
  log "\thopr_token_contract: ${hopr_token_contract}"
  log "\tperform_cleanup: ${perform_cleanup}"
  log "\tshow_prestartinfo: ${show_prestartinfo}"
fi
# }}}

# create test specific instance template
gcloud_create_or_update_instance_template "${cluster_id}" \
  "${docker_image}" \
  "${provider}" \
  "${api_token}" \
  "${password}"
#
# start nodes
gcloud_create_or_update_managed_instance_group "${cluster_id}" \
  6 \
  "${cluster_id}"

# get IPs of newly started VMs which run hoprd
declare node_ips
node_ips=$(gcloud_get_managed_instance_group_instances_ips "${cluster_id}")
declare node_ips_arr=( ${node_ips} )

#  --- Fund nodes --- {{{
declare eth_address
for ip in ${node_ips}; do
  wait_until_node_is_ready "${ip}"
  eth_address=$(get_eth_address "${ip}")
  fund_if_empty "${eth_address}" "${provider}" "${hopr_token_contract}"
done

for ip in ${node_ips}; do
  wait_for_port "9091" "${ip}"
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
