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


# TODO: Populate this from GH Secrets
declare -A staking_acc_dict=() # Maps "staking account address/name" => "private key"

# TODO: Call this always?
# yarn hardhat stake --network hardhat --amount 1000000000000000000000 --privatekey "${staking_priv_key[staking_acc_name]}"

declare -A ip_addrs
declare -A hopr_addrs
declare -A native_addrs

declare instance_names
instance_names=$(gcloud_get_managed_instance_group_instances_names "${cluster_id}")
declare instance_names_arr=( ${instance_names} )

# Iterate through all instances in this cluster
declare staking_acc_names_arr=( "${!staking_acc_dict[@]}" ) # staking accounts addresses/names only
declare count_staking_accs=${#staking_acc_names_arr[@]}
declare current_staking_index=0
for instance_name in "${instance_names_arr[@]}" ; do

  declare info_tag=$(gcloud_get_node_info_tag "${instance_name}")
  declare node_ip=$(gcloud_get_ip "${instance_name}")

  declare wallet_addr
  declare peer_id
  declare staking_acc
  if [[ -z "${info_tag}" ]]; then
    # If the instance does not have the INFO tag yet, we need to retrieve all info
    wallet_addr=$(get_native_address "${api_token}@${node_ip}:3001")
    peer_id=$(get_hopr_address "${api_token}@${node_ip}:3001")
    staking_acc="${staking_acc_names_arr[current_staking_index]}"

    # Save the info tag
    info_tag="info:native_addr=${wallet_addr};peer_id=${peer_id};nr_staking_account=${staking_acc}"
    gcloud_add_instance_tags "${instance_name}" "${info_tag}"
  else
    # Retrieve all information from the INFO tag
    wallet_addr=$(echo "${info_tag}" | sed -E 's/.*native_addr=([Xxa-f0-9A-F]+).*/\1/g')
    peer_id=$(echo "${info_tag}" | sed -E 's/.*peer_id=([a-zA-Z0-9]+).*/\1/g')
    staking_acc=$(echo "${info_tag}" | sed -E 's/.*nr_staking_account=([a-zA-Z0-9]+).*/\1/g')
  fi

  # Also use the staking account address here?
  ip_addrs+=( [$instance_name]="${node_ip}" )
  native_addrs+=( [$instance_name]="${wallet_addr}" )
  hopr_addrs+=( [$instance_name]="${peer_id}" )

  # Staking accounts are assigned round-robin
  current_staking_index=$(( (current_staking_index + 1) % count_staking_accs ))

  # Fund the node as well
  wait_until_node_is_ready "${node_ip}"
  fund_if_empty "${wallet_addr}" "${environment}"

done

# Register all nodes in cluster
declare cifs="$IFS"
IFS=','
yarn workspace @hoprnet/hopr-ethereum hardhat register \
   --network hardhat \
   --task add \
   --native-addresses "${native_addrs[*]}" \
   --peer-ids "${hopr_addrs[*]}"
IFS="${cifs}"


if [[ "${docker_image}" != *-nat:* ]]; then
  # Finally wait for the public nodes to come up
  for ip in "${ip_addrs[@]}"; do
    wait_for_port "9091" "${ip}"
  done
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
