#!/usr/bin/env bash

# prevent execution of this script, only allow sourcing
$(return >/dev/null 2>&1)
test "$?" -eq "0" || { echo "This script should only be sourced."; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

# don't source this file twice
test -z "${GCLOUD_SOURCED:-}" && GCLOUD_SOURCED=1 || exit 0

# set log id and use shared log function for readable logs
declare HOPR_LOG_ID="gcloud"
declare mydir
mydir=$(dirname $(readlink -f $0))
source "${mydir}/utils.sh"

# ------ GCloud utilities ------
#
# NB. functions here should not rely on any external env. variables, or functions
# not in this file, as this is intended for reuse in various scenarios.

GCLOUD_ZONE="--zone=europe-west6-a"
GCLOUD_REGION="--region=europe-west6"

GCLOUD_MACHINE="--machine-type=e2-medium"
GCLOUD_META="--metadata=google-logging-enabled=true,google-monitoring-enabled=true,enable-oslogin=true --maintenance-policy=MIGRATE"
GCLOUD_TAGS="--tags=hopr-node,web-client,rest-client,portainer,healthcheck"
GCLOUD_BOOTDISK="--boot-disk-size=20GB --boot-disk-type=pd-standard"
GCLOUD_IMAGE="--image-family=cos-stable"

GCLOUD_DEFAULTS="$GLOUD_ZONE $GCLOUD_MACHINE $GCLOUD_META $GCLOUD_TAGS $GCLOUD_BOOTDISK $GCLOUD_IMAGE"

# NB: This is useless for getting an IP of a VM
# Get or create an IP address
# $1 = name
gcloud_get_address() {
  local ip=$(gcloud compute addresses describe $1 $GCLOUD_REGION 2>&1)
  # Google does not return an appropriate exit code :(
  if [ "$(echo "$ip" | grep 'ERROR')" ]; then
    log "No address, creating" 1>&2
    gcloud compute addresses create $1 $REGION
    local ip=$(gcloud compute addresses describe $1 $GCLOUD_REGION 2>&1)
  fi
  echo $ip | awk '{ print $2 }'
}

# Get external IP for running node or die
# $1 - name
gcloud_get_ip() {
  gcloud compute instances list | grep "$1" | awk '{ print $5 }'
}

# $1 = VM name
gcloud_find_vm_with_name() {
  gcloud compute instances list | grep "$1" | grep 'RUNNING'
}

# $1 = address name
gcloud_delete_address() {
  gcloud compute addresses delete "$1" $GCLOUD_REGION --quiet
}

# $1 = space separated list where each item is used as a filter
gcloud_delete_addresses() {
  local addresses="$(gcloud_list_addresses)"
  for filter in $1
  do
    local ip=$(echo "$addresses" | grep $filter | awk '{ print $2 }')
    if [ $ip ]; then
      local name=$(echo "$addresses" | grep $filter | awk '{ print $1 }')
      log "releasing - $name ($ip)"
      gcloud_delete_address "$name"
    fi
  done
}

gcloud_list_addresses() {
  gcloud compute addresses list
}

# $1 = instance name
# $2 = OPTIONAL: zone name
# $3 = OPTIONAL: delete disks? if set to true
gcloud_delete_instance() {
  local zone="${GCLOUD_ZONE}"

  if [ -z "${2:-}" ]; then
    zone="--zone=${2}"
  fi

  local keep_disks="--keep-disks=data"

  if [ -z "${3:-}" ]; then
    keep_disks=""
  fi

  gcloud compute instances delete "$1" "$zone" ${keep_disks} --quiet
}

gcloud_list_instances() {
  gcloud compute instances list
}

# $1 = space separated list where each item is used as a filter
gcloud_delete_instances() {
  local instances="$(gcloud_list_instances)"
  for filter in $1
  do
    echo "$instances" | grep $filter | grep 'RUNNING' | while read -r instance; do
      local name=$(echo "$instance" | awk '{ print $1 }')
      local zone=$(echo "$instance" | awk '{ print $2 }')
      log "stopping $name"
      gcloud compute instances stop "${name}" --zone="${zone}"
    done

    echo "$instances" | grep $filter | grep 'TERMINATED' | while read -r instance; do
      local name=$(echo "$instance" | awk '{ print $1 }')
      local zone=$(echo "$instance" | awk '{ print $2 }')
      log "deleting terminated instance $name"
      gcloud_delete_instance "${name}" "${zone}"
    done
  done
}

# $1 - VM name
# Warning, using `--format='value[](metadata*)` is an unsupported API by gcloud and can change any time.
# More information on https://cloud.google.com/compute/docs/storing-retrieving-metadata
gcloud_get_image_running_on_vm() {
  gcloud compute instances describe $1 $ZONE \
    --format='value[](metadata.items.gce-container-declaration)' \
    | grep image \
    | tr -s ' ' \
    | cut -f3 -d' '
}

# $1 = vm name
# $2 = container-image
# $3 = disk name
# $4 = mount path
gcloud_update_container_with_image() {
  log "Updating container on vm:$1 - $2 (disk: $3:$4)"
  gcloud compute instances update-container $1 $ZONE \
    --container-image=$2 --container-mount-disk name=$3,mount-path="$4"
  sleep 30s
}

# $1 - vm name
# $2 - docker image
gcloud_stop() {
  log "Stopping docker image:$2 on vm $1"
  gcloud compute ssh --ssh-flag='-t' $ZONE $1 -- \
    "export DOCKER_IMAGE=$2 && docker stop \$(docker ps -q --filter ancestor=\$DOCKER_IMAGE)"
}

# $1 - vm name
# $2 - docker image
gcloud_get_logs() {
  # Docker sucks and gives us warnings in stdout.
  local id=$(gcloud compute ssh $ZONE $1 --command "docker ps -q --filter ancestor='$2' | xargs docker inspect --format='{{.Id}}'" | grep -v 'warning')
  gcloud compute ssh $ZONE $1 --command "docker logs $id"
}

# $1 - node name used as part of the disk name as well
# $2 - docker image
# $3 - extra gcloud arguments
gcloud_create_container_instance() {
  gcloud compute instances create-with-container $1 $GCLOUD_DEFAULTS \
    --create-disk name=$(disk_name $1),size=10GB,type=pd-standard,mode=rw \
    --container-image=$2 \
    --container-restart-policy=always \
    ${3}
}

# $1 - path to folder containing a Dockerfile
# $2 - docker image name
# $3 - docker image version
# $4 - if set to true new image is tagged with latest
gcloud_build_image() {
  local tag="${2}:${3}"
  local latest_tag="${2}:latest"
  gcloud builds submit "${1}" --tag ${tag}

  if [ "${4:-}" = "true" ]; then
    gcloud container images add-tag ${tag} ${latest_tag}
  fi
}
