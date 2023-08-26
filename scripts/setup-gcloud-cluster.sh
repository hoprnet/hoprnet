#!/usr/bin/env bash

# TODO: this script currently uses goerli as the RPC provider. However, it
# should be extended to use its own instance of anvil too.

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
# shellcheck disable=SC1090
source "${mydir}/utils.sh"
# shellcheck disable=SC1090
source "${mydir}/gcloud.sh"
# shellcheck disable=SC1090
source "${mydir}/testnet.sh"

usage() {
  msg
  msg "This script can be used to setup a cluster of nodes on gcloud and run"
  msg "an initial setup script against these nodes. Once testing has"
  msg "completed the script can be used to cleanup the cluster as well."
  msg
  msg "Usage: $0 <network> [<cluster_template_name> [<cluster_size>]]"
  msg
  msg "where <network>\t\tthe network id from which the smart contract addresses are derived"
  msg "      <cluster_id>\t\tuses a random value as default"
  msg "      <cluster_size>\t\tnumber of nodes in the deployed cluster, default is 6."
  msg
  msg "Required environment variables"
  msg "------------------------------"
  msg
  msg "Optional environment variables"
  msg "------------------------------"
  msg
  msg "HOPRD_SHOW_PRESTART_INFO\tset to 'true' to print used parameter values before starting"
  msg "HOPRD_PERFORM_CLEANUP\t\tset to 'true' to perform the cleanup process for the given cluster id"
  msg
}

# return early with help info when requested
{ [[ "${1:-}" = "-h" ]] || [[ "${1:-}" = "--help" ]]; } && { usage; exit 0; }


declare network="${1?"missing parameter <network>"}"
declare cluster_template_name="${2:-${network}-topology-${RANDOM}-${RANDOM}}"
declare cluster_size=${3-6}


declare perform_cleanup="${HOPRD_PERFORM_CLEANUP:-false}"

function cleanup {
  local EXIT_CODE=$?

  trap - SIGINT SIGTERM ERR EXIT
  set +Eeuo pipefail

  if [[ ${EXIT_CODE} -ne 0 ]] || [[ "${perform_cleanup}" = "true" ]] || [[ "${perform_cleanup}" = "1" ]]; then
    # Cleaning up everything upon failure
    gcloud_delete_managed_instance_group "${cluster_id}"
    gcloud_delete_instance_template "${cluster_template_name}"
  fi

  exit $EXIT_CODE
}

if [[ "${perform_cleanup}" = "1" ]] || [[ "${perform_cleanup}" = "true" ]]; then
  cleanup

  # exit right away
  exit
fi



# create instance template
# announce on-chain with routable address
gcloud_create_instance_template "${cluster_template_name}" 

# start nodes
gcloud_create_or_update_managed_instance_group  \
  "${cluster_template_name}" \
  "${cluster_size}" \
  "${cluster_template_name}"


