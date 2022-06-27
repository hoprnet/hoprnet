#!/usr/bin/env bash

# prevent execution of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" || { echo "This script should only be sourced." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
source "${mydir}/gcloud.sh"

# $1 = instance name
get_node_info_tag() {
  local instance_name="${1}"

  local current_tag_set=$(gcloud_get_instance_tags "${instance_name}")
  local current_tag_set_arr=( "${current_tag_set}" )

  # Find the tag containing the "info:" prefix
  local info_tag=""
  for tag in ${current_tag_set_arr}; do
    if [[ ${tag} =~ info:.+ ]]; then
         info_tag="${tag#info:}"
         break
    fi
  done

  echo "${info_tag}"
}

# $1 = instance name
# $2 = api token
create_node_info_tag() {
  local instance_name="${1}"
  local api_token="${2}"

  local node_ip=$(gcloud_get_ip "${instance_name}")
  local wallet_addr=$(get_native_address "${api_token}@${node_ip}:3001")
  local peer_id=$(get_hopr_address "${api_token}@${node_ip}:3001")

  echo "info:peer_id=${peer_id};native_addr=${wallet_addr};"
}

# $1 = dictionary: instance name => "" (will become staking account address)
# $2 = array of staking accounts to use
assign_staking_accounts() {
  local -n instance_names_dict="${1}"
  local -n staking_accs_arr="${2}"

  local current_staking_index=0
  local count_staking_accs=${#staking_accs_arr[@]}

  for instance_name in "${!instance_names_dict[@]}"; do

    local existing_info_tag=$(get_node_info_tag "${instance_name}")
    if [[ -z "${existing_info_tag}" ]]; then

      # Create a new INFO tag for the given VM instance
      local new_info_tag=$(create_node_info_tag "${instance_name}")

      # Also assign staking account (this is done in a round-robin fashion)
      local new_staking_addr="${staking_accs_arr[current_staking_index]}"

      # Append it to the info tag
      new_info_tag="${new_info_tag};nr_staking_addr=${new_staking_addr}"

      # Save the completed info tag to the VM instance
      gcloud_add_instance_tags "${instance_name}" "${new_info_tag}"

      instance_names_dict+=([${instance_name}]="${new_staking_addr}")
    else
      # Just retrieve the existing staking account
      local existing_staking_addr=$(echo "${existing_info_tag}" | sed -E 's/.*nr_staking_addr=([Xxa-f0-9A-F]+).*/\1/g')


      if [[ -z "${existing_staking_addr}" ]]; then
        existing_staking_addr="${staking_accs_arr[current_staking_index]}"

        # If the "nr_staking_addr" part of the info tag is missing, add it
        gcloud_remove_instance_tags "${instance_name}" "${existing_info_tag}"
        existing_info_tag="${existing_info_tag};nr_staking_addr=${existing_staking_addr}"
        gcloud_add_instance_tags "${instance_name}" "${existing_info_tag}"
      fi

      instance_names_dict+=([${instance_name}]="${existing_staking_addr}")
    fi

    current_staking_index=$(( (current_staking_index + 1) % count_staking_accs ))
  done
}