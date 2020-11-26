#!/bin/bash
# @param {('master'|'**')} RELEASE_NAME - The name of the release to deploy
# @requires gcloud
set -e

if [[ -z "${RELEASE_NAME}" ]]; then
  ENV="master"
else
  ENV="${RELEASE_NAME}"
fi

if [[ $(gcloud compute instances list | grep ${ENV}-bootstrap) ]]; then
  echo "GCLOUD_UPDATE_CONTAINER=true"
  echo "GCLOUD_CREATE_CONTAINER=false"
else
  echo "GCLOUD_UPDATE_CONTAINER=false"
  echo "GCLOUD_CREATE_CONTAINER=true"
fi

echo "GCLOUD_VM_NAME=${ENV}-bootstrap"