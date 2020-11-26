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
  echo "GCLOUD_ACTION_CONTAINER=update"
else
  echo "GCLOUD_ACTION_CONTAINER=create"
fi

echo "GCLOUD_VM_NAME=${ENV}-bootstrap"