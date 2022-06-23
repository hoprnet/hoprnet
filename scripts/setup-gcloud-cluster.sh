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
  msg "Usage: $0 <environment> [<init_script> [<cluster_id> [<docker_image> [<cluster_size> [<instance_template_name> [<announce_on_chain>]]]]]"
  msg
  msg "where <environment>\t\tthe environment from which the smart contract addresses are derived"
  msg "      <init_script>\t\tpath to a script which is called with all node API endpoints as parameters"
  msg "      <cluster_id>\t\tuses a random value as default"
  msg "      <docker_image>\t\tuses 'gcr.io/hoprassociation/hoprd:<environment>' as default"
  msg "      <cluster_size>\t\tnumber of nodes in the deployed cluster, default is 6."
  msg "      <instance_template_name>\t\tname of the gcloud instance template to use, default is <cluster_id>"
  msg "      <announce_on_chain>\t\tset to 'true' so started nodes should announce themselves, default is ''"
  msg
  msg "Required environment variables"
  msg "------------------------------"
  msg
  msg "FAUCET_SECRET_API_KEY\t\tsets the api key used to authenticate with the funding faucet"
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
: ${FAUCET_SECRET_API_KEY?"Missing environment variable FAUCET_SECRET_API_KEY"}

declare environment="${1?"missing parameter <environment>"}"
declare init_script=${2:-}
declare cluster_id="${3:-${environment}-topology-${RANDOM}-${RANDOM}}"
declare docker_image=${4:-gcr.io/hoprassociation/hoprd:${environment}}
declare cluster_size=${5:-6}
declare instance_template_name=${6:-${cluster_id}}
declare announce_on_chain=${7:-}

declare api_token="${HOPRD_API_TOKEN:-Token${RANDOM}^${RANDOM}^${RANDOM}Token}"
declare password="${HOPRD_PASSWORD:-pw${RANDOM}${RANDOM}${RANDOM}pw}"
declare perform_cleanup="${HOPRD_PERFORM_CLEANUP:-false}"
declare show_prestartinfo="${HOPRD_SHOW_PRESTART_INFO:-false}"

# Append environment as Docker image version, if not specified
[[ "${docker_image}" != *:* ]] && docker_image="${docker_image}:${environment}"

function cleanup {
  local EXIT_CODE=$?

  trap - SIGINT SIGTERM ERR EXIT
  set +Eeuo pipefail

  if [ ${EXIT_CODE} -ne 0 ] || [ "${perform_cleanup}" = "true" ] || [ "${perform_cleanup}" = "1" ]; then
    # Cleaning up everything upon failure
    gcloud_delete_managed_instance_group "${cluster_id}"
    gcloud_delete_instance_template "${instance_template_name}"
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
  log "\tinstance_template_name: ${instance_template_name}"
  log "\tannounce_on_chain: ${announce_on_chain}"
  log "\tinit_script: ${init_script}"
  log "\tenvironment: ${environment}"
  log "\tapi_token: ${api_token}"
  log "\tpassword: ${password}"
  log "\tperform_cleanup: ${perform_cleanup}"
  log "\tshow_prestartinfo: ${show_prestartinfo}"
fi
# }}}

# create instance template
# announce on-chain with routable address
gcloud_create_instance_template_if_not_exists \
  "${instance_template_name}" \
  "${docker_image}" \
  "${environment}" \
  "${api_token}" \
  "${password}" \
  "${announce_on_chain}"

# start nodes
gcloud_create_or_update_managed_instance_group  \
  "${cluster_id}" \
  ${cluster_size} \
  "${instance_template_name}"

# get IPs of newly started VMs which run hoprd
declare node_ips
node_ips=$(gcloud_get_managed_instance_group_instances_ips "${cluster_id}")
declare node_ips_arr=( ${node_ips} )

declare instance_names
instance_names=$(gcloud_get_managed_instance_group_instances_names "${cluster_id}")
declare instance_names_arr=( ${instance_names} )

# TODO: Feed in the staking account
declare staking_addrs=()

declare eth_addrs_arr=()
declare hopr_addrs_arr=()

#  --- Retrieve node information & fund nodes --- {{{
declare eth_addr
declare hopr_addr
for ip in ${node_ips}; do
  wait_until_node_is_ready "${ip}"

  eth_addr="$(get_native_address "${api_token}@${ip}:3001")"
  eth_addrs_arr+=( "${eth_addr}" )

  hopr_addr="$(get_hopr_address "${api_token}@${ip}:3001")"
  hopr_addrs_arr+=( "${hopr_addr}" )

  fund_if_empty "${eth_addr}" "${environment}"
done

# $1 = instance name
get_node_info_tag() {
  local instance_name="${1}"

  local current_tag_set=$(gcloud_get_instance_tags "${instance_name}")
  local -a current_tag_set_arr=( "${current_tag_set}" )

  # Find the tag containing the "info:" prefix
  local info_tag=""
  for tag in "${current_tag_set_arr[@]}"
  do
    echo "-- tag in $instance_name: $tag"
    if [[ ${tag} == info:* ]]; then
         info_tag="${info_tag#info:}"
         break
    fi
  done

  echo "${info_tag}"
}

# $1 = instance names array
# $2 = peer ids array
# $3 = wallet address array
# $4 = staking accounts to use
assign_staking_accounts() {
  local -n instance_names_arr="${1}"
  local -n peer_ids_arr="${2}"
  local -n native_addrs_arr="${3}"
  local -n staking_accs_arr="${4}"

  local current_staking_index=0
  local count_staking_accs=${#staking_accs_arr[@]}

  # Assign staking accounts to instances round-robin fashion
  local -a assigned_staking_accs_arr=()
  for i in "${!instance_names_arr[@]}"
  do
    local instance_name="${instance_names_arr[i]}"
    local instance_peer_id="${peer_ids_arr[i]}"
    local instance_native_addr="${native_addrs_arr[i]}"

    local existing_info_tag=$(get_node_info_tag ${instance_name})

    if [[ -z "${existing_info_tag}" ]]; then
      # Assign the new staking account
      local new_staking_addr="${staking_accs_arr[current_staking_index]}"
      local new_info_tag="info:peer_id=${instance_peer_id};native_addr=${instance_native_addr};nr_staking_addr=${new_staking_addr}"
      assigned_staking_accs_arr+=( "$new_staking_addr" )
      gcloud_add_instance_tags "${instance_name}" "${new_info_tag}"
    else
      # Use the existing staking account
      existing_staking_addr=$(echo "${existing_info_tag}" | sed -E 's/.*nr_staking_addr=([Xxa-f0-9A-F]+).*/\1/g')
      assigned_staking_accs_arr+=( "${existing_staking_addr}" )
    fi

    current_staking_index=$(( (current_staking_index + 1) % count_staking_accs ))
  done

  # Return the assigned staking accounts array
  echo "${assigned_staking_accs_arr[@]}"
}

# To test Network registry, the cluster_size is greater or equal to 3 and staker_addresses are provided as parameters

# TODO: call stake API so that the first staker_addresses[0] stake in the current program
# TODO: call register API and register staker_addresses with node peer ids


if [[ "${docker_image}" != *-nat:* ]]; then
  # -- Public nodes --

  # TODO: The first public node will be left unstaked

  for vm in ${instance_names_arr}; do

  done


  # Finally wait for the public nodes to come up
  for ip in ${node_ips}; do
    wait_for_port "9091" "${ip}"
  done

else
  # -- NAT nodes --


 # We cannot wait for NAT nodes to come up, because their port 9091 is not exposed
fi
# }}}

# --- Call init script--- {{{
if [ -n "${init_script}" ] && [ -x "${init_script}" ]; then
  HOPRD_API_TOKEN="${api_token}" \
    "${init_script}" \
    ${node_ips_arr[@]/%/:3001}
fi
# }}}

log "finished"
