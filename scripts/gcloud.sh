#!/bin/bash
set -e #u

# ------ GCloud utilities ------
#
# NB. functions here should not rely on any external env. variables, or functions
# not in this file, as this is intended for reuse in various scenarios.

GCLOUD_INCLUDED=1 # So we can test for inclusion
ZONE="--zone=europe-west6-a"
REGION="--region=europe-west6"

GCLOUD_MACHINE="--machine-type=e2-medium"
GCLOUD_META="--metadata=google-logging-enabled=true --maintenance-policy=MIGRATE"
GCLOUD_TAGS="--tags=hopr-node,web-client,rest-client,portainer"
GCLOUD_BOOTDISK="--boot-disk-size=20GB --boot-disk-type=pd-standard"

GCLOUD_DEFAULTS="$ZONE $GCLOUD_MACHINE $GCLOUD_META $GCLOUD_TAGS $GCLOUD_BOOTDISK"

alias gssh="gcloud compute ssh --ssh-flag='-t' $ZONE"


# $1 = role (ie. bootstrap)
# Requires:
# - VERSION_MAJ_MIN
# - RELEASE_NAME
gcloud_vm_name() {
  # For example, 1-57-larnaca-bootstrap, prerelease-master-bootstrap
  echo "bootstrap-$(echo $VERSION_MAJ_MIN | sed 's/\./-/g')-$RELEASE_NAME"
}

# Get or create an IP address
# $1 = name
gcloud_get_address() { 
  local ip=$(gcloud compute addresses describe $1 $REGION 2>&1)
  # Google does not return an appropriate exit code :(
  if [ "$(echo "$ip" | grep 'ERROR')" ]; then
    echo "No address, creating" 1>&2
    gcloud compute addresses create $1 $REGION
    local ip=$(gcloud compute addresses describe $1 $REGION 2>&1)
  fi
  echo $ip | awk '{ print $2 }'
}

# $1 = VM name
gcloud_find_vm_with_name() {
  echo $(gcloud compute instances list | grep "$1" | grep 'RUNNING')
}

# $1 - VM name
# Warning, using `--format='value[](metadata*)` is an unsupported API by gcloud and can change any time.
# More information on https://cloud.google.com/compute/docs/storing-retrieving-metadata
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

# $1 - vm name
# $2 - docker image
gcloud_get_logs() {
  # Docker sucks and gives us warnings in stdout.
  local id=$(gcloud compute ssh $ZONE $1 --command "docker ps -q --filter ancestor='$2' | xargs docker inspect --format='{{.Id}}'" | grep -v 'warning')
  gcloud compute ssh $ZONE $1 --command "docker logs $id"
}
