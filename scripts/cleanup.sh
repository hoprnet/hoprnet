#!/usr/bin/env bash
# Cleanup old gcloud resources.

set -o errexit
set -o nounset
set -o pipefail

if [ -z "$GCLOUD_INCLUDED" ]; then
  source scripts/gcloud.sh
fi

if [ -z "$OLD_RELEASES" ]; then
  source scripts/environments.sh
fi

cleanup_ips() {
  local IPS
  IPS=$(gcloud compute addresses list)

  for x in $OLD_RELEASES
  do
    local IP
    IP=$(echo "$IPS" | grep "$x" | awk '{ print $2 }')
    if [ "$IP" ]; then
      local NAME
      NAME=$(echo "$IPS" | grep "$x" | awk '{ print $1 }')

      echo "- releasing - $NAME ($IP)"
      gcloud compute addresses delete "$NAME" "$REGION" --quiet
    fi
  done
}

cleanup_instances() {
  local INSTANCES
  INSTANCES=$(gcloud compute instances list)

  for old in $OLD_RELEASES
  do
    echo "$INSTANCES" | grep "$old" | grep 'RUNNING' | while read -r instance; do
      local name
      name=$(echo "$instance" | awk '{ print $1 }')

      echo "- stopping $name"
      gcloud compute instances stop "$name" "$ZONE"
    done

    echo "$INSTANCES" | grep "$old" | grep 'TERMINATED' | while read -r instance; do
      local name
      local zone

      name=$(echo "$instance" | awk '{ print $1 }')
      zone=$(echo "$instance" | awk '{ print $2 }')

      echo "- deleting terminated instance $name"
      gcloud compute instances delete "$name" --zone="$zone" --keep-disks=data --quiet
    done
  done
}

cleanup () {
  cleanup_ips
  cleanup_instances
}

# execute default cleanup if script is called directly and not sourced
if [ "$0" = "./scripts/cleanup.sh" ]; then
  cleanup
fi
