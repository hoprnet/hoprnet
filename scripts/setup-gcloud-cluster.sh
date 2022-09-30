#!/usr/bin/env bash

# TODO: this script currently uses goerli as the RPC provider. However, it
# should be extended to use its own instance of hardhat too.

# prevent sourcing of this script, only allow execution
# shellcheck disable=SC2091
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare -x HOPR_LOG_ID="setup-gcloud-cluster"
# shellcheck disable=SC1091
source "${mydir}/utils.sh"
# shellcheck disable=SC1091
source "${mydir}/gcloud.sh"
# shellcheck disable=SC1091
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
  msg "HOPRD_RESET_METADATA\t\tset to 'true' to trigger metadata reset on instances"
  msg "HOPRD_SKIP_UNSTAKED\t\tset to 'true' to stake all nodes and not keep the first unstaked"
  msg
}

# return early with help info when requested
{ [[ "${1:-}" = "-h" ]] || [[ "${1:-}" = "--help" ]]; } && { usage; exit 0; }

# verify and set parameters
: "${FAUCET_SECRET_API_KEY?"Missing environment variable FAUCET_SECRET_API_KEY"}"
: "${STAKING_ACCOUNT_BA28?"Missing environment variable STAKING_ACCOUNT_BA28"}"
: "${STAKING_ACCOUNT_F84B?"Missing environment variable STAKING_ACCOUNT_F84B"}"
: "${STAKING_ACCOUNT_0FD4?"Missing environment variable STAKING_ACCOUNT_0FD4"}"
: "${STAKING_ACCOUNT_6C15?"Missing environment variable STAKING_ACCOUNT_6C15"}"

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
declare reset_metadata="${HOPRD_RESET_METADATA:-false}"
declare skip_unstaked="${HOPRD_SKIP_UNSTAKED:-false}"

# Append environment as Docker image version, if not specified
[[ "${docker_image}" != *:* ]] && docker_image="${docker_image}:${environment}"

function cleanup {
  local EXIT_CODE=$?

  trap - SIGINT SIGTERM ERR EXIT
  set +Eeuo pipefail

  if [[ ${EXIT_CODE} -ne 0 ]] || [[ "${perform_cleanup}" = "true" ]] || [[ "${perform_cleanup}" = "1" ]]; then
    # Cleaning up everything upon failure
    gcloud_delete_managed_instance_group "${cluster_id}"
    gcloud_delete_instance_template "${instance_template_name}"
  fi

  exit $EXIT_CODE
}

if [[ "${perform_cleanup}" = "1" ]] || [[ "${perform_cleanup}" = "true" ]]; then
  cleanup

  # exit right away
  exit
fi

# --- Log test info {{{
if [[ "${show_prestartinfo}" = "1" ]] || [[ "${show_prestartinfo}" = "true" ]]; then
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
gcloud_create_instance_template \
  "${instance_template_name}" \
  "${docker_image}" \
  "${environment}" \
  "${api_token}" \
  "${password}" \
  "${announce_on_chain}"

# start nodes
gcloud_create_or_update_managed_instance_group  \
  "${cluster_id}" \
  "${cluster_size}" \
  "${instance_template_name}"

# This maps "staking account address" => "private key"
declare -A staking_addrs_dict

# NOTE: the addresses are sorted alphabetically here, to see the actual order the keys will
# have after sorting. As usual, dictionaries do not keep the insertion order of the keys.

# May be supplied differently in future to accommodate with bigger GCP cluster sizes.
if [[  "${docker_image}" != *-nat:* ]]; then
  # Staking addresses for public nodes
  staking_addrs_dict=(
    [0xBA28EE6743d008ed6794D023B10D212bc4Eb7e75]="${STAKING_ACCOUNT_BA28}"
    [0xf84Ba32dd2f2EC2F355fB63F3fC3e048900aE3b2]="${STAKING_ACCOUNT_F84B}"
  )
else
  # Staking addresses for NAT nodes
  staking_addrs_dict=(
    [0x0Fd4C32CC8C6237132284c1600ed94D06AC478C6]="${STAKING_ACCOUNT_0FD4}"
    [0x6c150A63941c6d58a2f2687a23d5a8E0DbdE181C]="${STAKING_ACCOUNT_6C15}"
  )
fi

declare network_id
network_id="$(get_network "${environment}")"

# Deployer CI wallet should ideally be "eligible". To be eligible:
# 1. The wallet should have obtained a "Network_registry" NFT of `developer` rank (wallet should already have this)
# 2. The wallet should have sent one above-mentioned NFT to the staking contract
# FIXME: Correctly format the condition (in line with *meta* environment), so that the following lines are skipped for most of the time, and only be executed when:
# - the CI nodes wants to perform `selfRegister`
# This can be called always, because the "stake" task is idempotent given the same arguments
for staking_addr in "${!staking_addrs_dict[@]}" ; do
  fund_if_empty "${staking_addr}" "${environment}"
  # we only stake NFT for valencia release
  PRIVATE_KEY="${staking_addrs_dict[${staking_addr}]}" make -C "${mydir}/.." stake-nrnft environment="${environment}" nftrank=developer network="${network_id}"
done

# Get names of all instances in this cluster
# TODO: now `native-addresses` (a.k.a. `hopr_addrs`) doesn't need to contain unique values. The array can contain repetitive addresses
declare instance_names
instance_names="$(gcloud_get_managed_instance_group_instances_names "${cluster_id}")"
declare -a instance_names_arr
IFS="," read -r -a instance_names_arr <<< "$(echo "${instance_names}" | jq -r '@csv' | tr -d '\"')"

# Prepare sorted staking account addresses so we ensure a stable order of assignment
declare staking_addresses_arr=( "${!staking_addrs_dict[@]}" )
readarray -t staking_addresses_arr < <(for addr in "${!staking_addrs_dict[@]}"; do echo "$addr"; done | sort)

# These arrays will hold IP addresses, peer IDs and staking addresses
# for instance VMs in the encounter order of the `instance_names` array
declare -a ip_addrs
declare -a hopr_addrs
declare -a used_staking_addrs

# Iterate through all VM instances
# The loop should be parallelized in future to accommodate better with larger clusters
for instance_idx in "${!instance_names_arr[@]}" ; do
  # Firstly, retrieve the IP address of this VM instance
  instance_name="${instance_names_arr[instance_idx]}"
  node_ip=$(gcloud_get_ip "${instance_name}")

  wait_until_node_is_ready "${node_ip}"

  if [[ "${reset_metadata}" = "true" ]]; then
    gcloud_remove_instance_metadata "${instance_name}" "HOPRD_PEER_ID,HOPRD_WALLET_ADDR,HOPRD_STAKING_ADDR"
  fi

  # All VM instances in the deployed cluster will get a special metadata entries
  # which contain all information about the HOPR instance running in the VM.
  # These currently include:
  # - node wallet address
  # - node peer ID
  # - associated staking account
  # This information is constant during the lifetime of the VM and
  # does not change during re-deployment once set.
  declare instance_metadata
  instance_metadata="$(gcloud_get_node_info_metadata "${instance_name}")"

  # known metadata keys
  declare wallet_addr peer_id staking_addr
  wallet_addr="$(echo "${instance_metadata}" | jq -r '."HOPRD_WALLET_ADDR" // empty')"
  peer_id="$(echo "${instance_metadata}" | jq -r '."HOPRD_PEER_ID" // empty')"
  staking_addr="$(echo "${instance_metadata}" | jq -r '."HOPRD_STAKING_ADDR" // empty')"

  # data from the node's API for verification or initialization
  declare api_wallet_addr api_peer_id
  api_wallet_addr="$(get_native_address "${api_token}@${node_ip}:3001")"
  api_peer_id="$(get_hopr_address "${api_token}@${node_ip}:3001")"

  if [[ -z "${staking_addr}" ]]; then
    # If the instance does not have metadata yet, we set it once

    # NOTE: We leave only the first public node unstaked
    if [[ ${instance_idx} -eq 0 && "${instance_template_name}" != *-nat* && "${skip_unstaked}" != "true" ]]; then
      staking_addr="unstaked"
    else
      # Staking accounts are assigned round-robin
      staking_addr_idx=$(( (instance_idx ) % ${#staking_addresses_arr[@]} ))
      staking_addr="${staking_addresses_arr[staking_addr_idx]}"
    fi

    # Save the metadata
    declare new_metadata="HOPRD_WALLET_ADDR=${api_wallet_addr},HOPRD_PEER_ID=${api_peer_id},HOPRD_STAKING_ADDR=${staking_addr}"
    gcloud_add_instance_metadata "${instance_name}" "${new_metadata}"
    gcloud_execute_command_instance "${instance_name}" 'sudo /opt/hoprd/startup-script.sh >> /tmp/startup-script-`date +%Y%m%d-%H%M%S`.log'
  else
    # cross-check data, and log discrepancies, we keep going though and leave
    # the reconciliation for another process to do
    if [[ "${api_wallet_addr}" != "${wallet_addr}" ]]; then
      log "ERROR: instance ${instance_name} has changed wallet addr from original ${wallet_addr} to ${api_wallet_addr}"
    fi
    if [[ "${api_peer_id}" != "${peer_id}" ]]; then
      log "ERROR: instance ${instance_name} has changed peer id from original ${peer_id} to ${api_peer_id}"
    fi
  fi

  ip_addrs+=( "${node_ip}" )

  # Do not include the unstaked nodes (= skipped during registration for NR)
  if [[ "${staking_addr}" != "unstaked" ]]; then
    hopr_addrs+=( "${api_peer_id}" )
    used_staking_addrs+=( "${staking_addr}" )
  fi

  # Fund the node as well
  fund_if_empty "${api_wallet_addr}" "${environment}"
done

# Register all nodes in cluster
IFS=','
# If same order of parameters is given, the "register" task is idempotent
make -C "${mydir}/.." register-nodes \
  environment="${environment}" \
  native_addresses="${used_staking_addrs[*]}" \
  peer_ids="${hopr_addrs[*]}" \
  network="${network_id}"

make -C "${mydir}/.." sync-eligibility \
  environment="${environment}" \
  peer_ids="${hopr_addrs[*]}" \
  network="${network_id}"
unset IFS

# Finally wait for the public nodes to come up, for NAT nodes this isn't possible
# because the P2P port is not exposed.
if [[ "${instance_template_name}" != *-nat* ]]; then
  for ip in "${ip_addrs[@]}"; do
    wait_for_port "9091" "${ip}"
  done
fi
# }}}

# --- Call init script--- {{{
if [[ -n "${init_script}" ]] && [[ -x "${init_script}" ]]; then
  # shellcheck disable=SC2068
  HOPRD_API_TOKEN="${api_token}" \
    "${init_script}" \
    ${ip_addrs[@]/%/:3001}
fi
# }}}

log "finished"
