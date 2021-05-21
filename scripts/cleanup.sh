#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && (echo "This script should only be executed."; exit 1)

# Cleanup old gcloud resources.

# don't source this file twice
test -z "${CLEANUP_SOURCED:-}" && CLEANUP_SOURCED=1 || exit 0

# set log id and use shared log function for readable logs
declare HOPR_LOG_ID="cleanup"
source "$(dirname $(readlink -f $0))/utils.sh"

# source gcloud functions and environments info
source "$(dirname $(readlink -f $0))/gcloud.sh"
source "$(dirname $(readlink -f $0))/environments.sh"

cleanup_instances() {
  local INSTANCES="$(gcloud_list_instances)"
  for old in $OLD_RELEASES
  do
    echo "$INSTANCES" | grep $old | grep 'RUNNING' | while read -r instance; do
      local name=$(echo "$instance" | awk '{ print $1 }')
      echo "- stopping $name"
      gcloud compute instances stop $name $ZONE
    done

    echo "$INSTANCES" | grep $old | grep 'TERMINATED' | while read -r instance; do
      local name=$(echo "$instance" | awk '{ print $1 }')
      local zone=$(echo "$instance" | awk '{ print $2 }')
      echo "- deleting terminated instance $name"
      gcloud_delete_instance "${name}" "${zone}"
    done
  done
}

gcloud_delete_addresses "${OLD_RELEASES}"
gcloud_delete_instances "${OLD_RELEASES}"
