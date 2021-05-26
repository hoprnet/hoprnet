#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

usage() {
  echo >&2
  echo "Usage: $0" >&2
  echo >&2
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
declare mydir

mydir=$(dirname $(readlink -f $0))

# set log id and use shared log function for readable logs
declare HOPR_LOG_ID="cleanup"
source "${mydir}/lib/utils.sh"

# source gcloud functions and environments info
source "${mydir}/lib/gcloud.sh"
source "${mydir}/lib/environments.sh"

cleanup_instances() {
  local INSTANCES="$(gcloud_list_instances)"

  for old in $OLD_RELEASES
  do
    echo "$INSTANCES" | grep $old | grep 'RUNNING' | while read -r instance; do
      local name=$(echo "$instance" | awk '{ print $1 }')
      log "stopping $name"
      gcloud compute instances stop $name $ZONE
    done

    echo "$INSTANCES" | grep $old | grep 'TERMINATED' | while read -r instance; do
      local name=$(echo "$instance" | awk '{ print $1 }')
      local zone=$(echo "$instance" | awk '{ print $2 }')
      log "deleting terminated instance $name"
      gcloud_delete_instance "${name}" "${zone}"
    done
  done
}

gcloud_delete_addresses "${OLD_RELEASES}"

gcloud_delete_instances "${OLD_RELEASES}"
