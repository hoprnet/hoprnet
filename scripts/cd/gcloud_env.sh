#!/bin/bash
# @param {('master'|'**')} RELEASE_NAME - The name of the release to deploy
# @requires gcloud
set -e
[[ ! $(type -P "gcloud") ]] && { echo "gcloud is NOT in PATH, exiting." 1>&2; exit 1; }

if [[ -z "${RELEASE_NAME}" ]]; then
  ENV="master"
else
  ENV="${RELEASE_NAME}"
fi

if [[ $(gcloud compute instances list | grep ${ENV}-bootstrap) ]]; then
  echo "GCLOUD_ACTION_CONTAINER=update"
  echo "GCLOUD_VM_IMAGE=$(gcloud compute instances describe ${ENV}-bootstrap --zone=europe-west6-a --format='value[](metadata.items.gce-container-declaration)' | grep image | tr -s ' ' | cut -f3 -d' ')"
else
  echo "GCLOUD_ACTION_CONTAINER=create"
fi

echo "GCLOUD_VM_DISK=/mnt/disks/gce-containers-mounts/gce-persistent-disks/bs-${ENV}"
echo "GCLOUD_VM_NAME=${ENV}-bootstrap"