#!/bin/bash
set -e #u
shopt -s expand_aliases

# ------ GCloud utilities ------
#
# NB. functions here should not rely on any external env. variables, or functions
# not in this file, as this is intended for reuse in various scenarios.

ZONE="--zone=europe-west6-a"

alias gssh="gcloud compute ssh --ssh-flag='-t' $ZONE"

# $1 = VM name
gcloud_find_vm_with_name() {
  echo $(gcloud compute instances list | grep $1)
}

# $1 - VM name
gcloud_get_image_running_on_vm() {
  echo $(gcloud compute instances describe $1 $ZONE \
    --format='value[](metadata.items.gce-container-declaration)' \
    | grep image \
    | tr -s ' ' \
    | cut -f3 -d' ')
}

# $1 = vm name
# $2 = container-image
# $3 = disk name
# $4 = mount path
gcloud_update_container_with_image() {
  echo "Updating container on vm:$1 - $2 (disk: $3:$4)"
  gcloud compute instances update-container $1 $ZONE \
    --container-image=$2 --container-mount-disk name=$3,mount-path="$4"
  sleep 30s
}

# $1 - vm name 
# $2 - docker image
gcloud_stop() {
  echo "Stopping docker image:$2 on vm $1"
  gssh $1 -- "export DOCKER_IMAGE=$2 && docker stop \$(docker ps -q --filter ancestor=\$DOCKER_IMAGE)"
}
