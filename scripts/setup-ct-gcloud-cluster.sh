#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare -x HOPR_LOG_ID="setup-ct-gcloud-cluster"
source "${mydir}/utils.sh"
source "${mydir}/gcloud.sh"
source "${mydir}/testnet.sh"

usage() {
  msg
  msg "This script can be used to setup a cluster of CT nodes on gcloud."
  msg "Once usage has completed the script can be used to cleanup the"
  msg "cluster as well."
  msg
  msg "Usage: $0 [<cluster_id> [<docker_image_tag> [<number_of_nodes>]]]"
  msg
  msg "where <cluster_id>\t\tuses a random value as default"
  msg "      <docker_image_tag>\tuses 'latest' as default"
  msg "      <number_of_nodes>\tuses '1' as default"
  msg
  msg "The docker image 'gcr.io/hoprassociation/hopr-cover-traffic-daemon' is used."
  msg
  msg "Required environment variables"
  msg "------------------------------"
  msg
  msg "CT_PRIV_KEY\t\t\tsets the account which is used to run the nodes"
  msg
  msg "Optional environment variables"
  msg "------------------------------"
  msg
  msg "HOPRD_SHOW_PRESTART_INFO\tset to 'true' to print used parameter values before starting"
  msg "HOPRD_PERFORM_CLEANUP\t\tset to 'true' to perform the cleanup process for the given cluster id"
  msg
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

# verify and set parameters
declare cluster_id="${1:-custom-ct-cluster-${RANDOM}-${RANDOM}}"
declare docker_image_tag=${2:-latest}
declare number_of_nodes=${3:-1}

declare docker_image="gcr.io/hoprassociation/hopr-cover-traffic-daemon:${docker_image_tag}"
declare perform_cleanup="${HOPRD_PERFORM_CLEANUP:-false}"
declare show_prestartinfo="${HOPRD_SHOW_PRESTART_INFO:-1}"

test -z "${CT_PRIV_KEY:-}" && { msg "Missing environment variable CT_PRIV_KEY"; usage; exit 1; }

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
fi

# --- Log test info {{{
if [ "${show_prestartinfo}" = "1" ] || [ "${show_prestartinfo}" = "true" ]; then
  log "Pre-Start Info"
  log "\tdocker_image: ${docker_image}"
  log "\tcluster_id: ${cluster_id}"
  log "\tperform_cleanup: ${perform_cleanup}"
  log "\tshow_prestartinfo: ${show_prestartinfo}"
fi
# }}}

# create test specific instance template
# the empty values are placeholders for optional parameters which are not used
gcloud_create_or_update_instance_template \
  "${cluster_id}" \
  "${docker_image}" \
  "" \
  "" \
  "" \
  "${CT_PRIV_KEY}" \
  "true"

# start nodes
gcloud_create_or_update_managed_instance_group \
  "${cluster_id}" \
  "${number_of_nodes}" \
  "${cluster_id}"

log "finished"
